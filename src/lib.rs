use Error as Err;

use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt::Display;
use std::io::{self, Write};
use std::ops::{Deref, DerefMut};
use std::result;
use std::sync::Arc;

use clap::Parser;
use tokio::sync::Semaphore;
use tracing::{debug, warn};

mod pypi;

pub type Result<T> = result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    Join(#[from] tokio::task::JoinError),

    #[error("error parsing dependencies: {}", .0)]
    ParseDeps(&'static str),

    #[error(transparent)]
    ParseFilter(#[from] tracing_subscriber::filter::ParseError),

    #[error("parsing input as toml failed")]
    ParseToml(#[from] toml::de::Error),

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error("error serializing toml: {}", .0)]
    SerializeToml(&'static str),

    #[error("unknown error: {}", .0)]
    Unknown(String),
}

impl From<Infallible> for Error {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Config {
    /// File from which to parse dependencies
    #[arg(short, long, default_value = "pyproject.toml")]
    input: String,

    /// Number of `PyPI` requests to send in parallel
    #[arg(short, long, default_value = "10")]
    requests: usize,
}

#[derive(Clone, Debug)]
struct PypiDeps {
    dependencies: Dependencies,
    optional_dependencies: HashMap<String, Dependencies>,
}

impl TryFrom<TomlTable> for PypiDeps {
    type Error = Error;

    fn try_from(toml: TomlTable) -> Result<Self> {
        let project = toml
            .get("project")
            .ok_or_else(|| Err::ParseDeps("no such section: project"))?;
        let dependencies: Dependencies = project
            .get("dependencies")
            .ok_or_else(|| Err::ParseDeps("no such section: project.dependencies"))?
            .try_into()?;

        let optional_dependencies: HashMap<_, _> = project
            .get("optional-dependencies")
            .and_then(|v| v.as_table())
            .map_or(Ok(HashMap::new()), |t| {
                t.iter()
                    .map(|(k, v)| {
                        let deps: Dependencies = v.try_into()?;
                        Ok((k.clone(), deps))
                    })
                    .collect::<Result<_>>()
            })?;

        Ok(Self {
            dependencies,
            optional_dependencies,
        })
    }
}

#[derive(Clone, Debug)]
struct Dependencies(Vec<(String, Option<(String, String)>)>);

impl TryFrom<&toml::Value> for Dependencies {
    type Error = Error;

    fn try_from(value: &toml::Value) -> Result<Self> {
        use Err::ParseDeps;
        let Some(vals) = value.as_array() else {
            return Err(ParseDeps("value was not array"));
        };
        let vec: Vec<_> = vals
            .iter()
            .map(|line| {
                let line = line.to_string();
                let trimmed = line.trim_matches('"');
                debug!("processing line: {trimmed}");

                let pat = {
                    // https://packaging.python.org/en/latest/specifications/version-specifiers/#id4
                    // Ordering is important as we return the first match
                    let pats = ["===", "~=", "==", "!=", "<=", ">=", "<", ">"];
                    pats.into_iter().find(|&pat| trimmed.contains(pat))
                };

                // No version, name only
                let Some(pat) = pat else {
                    return Ok((trimmed.to_owned(), None));
                };
                let mut splitter = trimmed.split(pat);

                let Some(name) = splitter.next().map(str::trim).map(str::to_string) else {
                    return Err(ParseDeps("line without valid dependency name"));
                };
                let Some(version) = splitter.next().map(str::trim).map(str::to_string) else {
                    return Err(ParseDeps(
                        "dependency with constraint (e.g. `==`) but no version",
                    ));
                };
                Ok((name, Some((pat.to_owned(), version))))
            })
            .collect::<Result<_>>()?;
        Ok(Self(vec))
    }
}

async fn fetch_latest_versions(
    deps: &PypiDeps,
    concurrency: usize,
) -> Result<HashMap<String, String>> {
    let semaphore = Arc::new(Semaphore::new(concurrency));

    let mut handles = Vec::new();
    for (name, _) in &deps.dependencies.0 {
        let semaphore = semaphore.clone();
        let name = name.clone();

        let handle = tokio::spawn(async move {
            let permit = semaphore.acquire().await.unwrap();
            let version = pypi::find_latest(&name).await;
            drop(permit);
            (name, version)
        });
        handles.push(handle);
    }

    for opt_deps in deps.optional_dependencies.values() {
        for (name, _) in &opt_deps.0 {
            let semaphore = semaphore.clone();
            let name = name.clone();

            let handle = tokio::spawn(async move {
                let permit = semaphore.acquire().await.unwrap();
                let version = pypi::find_latest(&name).await;
                drop(permit);
                (name, version)
            });
            handles.push(handle);
        }
    }

    let mut latest_versions = HashMap::new();
    for handle in handles {
        match handle.await? {
            (name, Ok(version)) => {
                latest_versions.insert(name, version);
            }
            (name, Err(e)) => {
                warn!("unable to find latest version of {name} due to {e}");
                continue;
            }
        }
    }
    Ok(latest_versions)
}

fn update_versions(deps: &mut PypiDeps, latest_versions: &HashMap<String, String>) -> Result<()> {
    let errmsg = |name| {
        format!("dependency {name} should already be in map of latest dependencies but not found:\n{latest_versions:?}")
    };
    for (name, constraints) in &mut deps.dependencies.0 {
        let constraint = constraints
            .as_ref()
            .map_or("==".to_string(), |(c, _)| c.to_string());
        let latest = latest_versions
            .get(name)
            .ok_or_else(|| Error::Unknown(errmsg(name)))?;
        *constraints = Some((constraint, latest.clone()));
    }

    for opt_deps in deps.optional_dependencies.values_mut() {
        for (name, constraints) in &mut opt_deps.0 {
            let constraint = constraints
                .as_ref()
                .map_or("==".to_string(), |(c, _)| c.to_string());
            let latest = latest_versions
                .get(name)
                .ok_or_else(|| Error::Unknown(errmsg(name)))?;
            *constraints = Some((constraint, latest.clone()));
        }
    }
    Ok(())
}

#[derive(Clone)]
struct TomlTable(toml::Table);

impl Deref for TomlTable {
    type Target = toml::Table;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TomlTable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for PypiDeps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.dependencies.0.is_empty() {
            writeln!(f, "dependencies = [")?;
            for dep in &self.dependencies.0 {
                match dep {
                    (ref name, Some((constraint, version))) => {
                        writeln!(f, "    \"{name} {constraint} {version}\",")?;
                    }
                    (ref name, None) => {
                        writeln!(f, "    \"{name}\",")?;
                    }
                }
            }
            writeln!(f, "]")?;
        }

        if self.optional_dependencies.is_empty() {
            return Ok(());
        }

        for (opt_name, deps) in &self.optional_dependencies {
            writeln!(f, "\n{opt_name} = [")?;
            for dep in &deps.0 {
                match dep {
                    (ref name, Some((constraint, version))) => {
                        writeln!(f, "    \"{name} {constraint} {version}\",")?;
                    }
                    (ref name, None) => {
                        writeln!(f, "    \"{name}\",")?;
                    }
                }
            }
            writeln!(f, "]")?;
        }
        Ok(())
    }
}

#[tracing::instrument]
#[tokio::main]
pub async fn run() -> Result<()> {
    let config = Config::parse();

    let input = std::fs::read_to_string(config.input)?;
    let toml = TomlTable(input.parse()?);
    let mut deps: PypiDeps = toml.try_into()?;
    let latest_versions = fetch_latest_versions(&deps, config.requests).await?;

    update_versions(&mut deps, &latest_versions)?;

    write!(io::stdout(), "Newest versions:\n\n{deps}")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_deps() {
        let deps: PypiDeps = TomlTable(
            std::fs::read_to_string("tests/files/pyproject.toml")
                .unwrap()
                .parse()
                .unwrap(),
        )
        .try_into()
        .unwrap();

        assert!(deps.dependencies.0.contains(&(
            "cryptography".to_string(),
            Some(("~=".to_string(), "41.0".to_string()))
        )));

        let test_deps = deps.optional_dependencies.get("test").unwrap();
        assert!(test_deps.0.contains(&(
            "black".to_string(),
            Some(("==".to_string(), "22.12.0".to_string()))
        )));
    }
}
