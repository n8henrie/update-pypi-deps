#![warn(clippy::pedantic)]

use std::collections::HashMap;
use std::convert::Infallible;
use std::io::{self, Write};
use std::result;

use clap::Parser;
use serde::Deserialize;
use thiserror;
use tracing::{debug, info, trace, warn};

pub type Result<T> = result::Result<T, UpdatePypiDepsError>;

#[derive(thiserror::Error, Debug)]
pub enum UpdatePypiDepsError {
    #[error("parsing input as toml failed")]
    ParseTomlError(#[from] toml::de::Error),

    #[error("there was an IO error")]
    Io(#[from] io::Error),

    #[error("error parsing dependencies: {}", .0)]
    ParseDepsError(&'static str),

    #[error("unknown error")]
    Unknown,
}

impl From<Infallible> for UpdatePypiDepsError {
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
}

#[derive(Deserialize, Debug)]
struct PypiDeps(HashMap<String, Option<String>>);

impl TryFrom<&toml::Value> for PypiDeps {
    type Error = UpdatePypiDepsError;

    fn try_from(value: &toml::Value) -> Result<Self> {
        use UpdatePypiDepsError::ParseDepsError as PDE;
        let Some(vals) = value.as_array() else {
            return Err(PDE("value was not array"));
        };
        let hm: HashMap<_, _> = vals
            .into_iter()
            .map(|line| {
                let line = line.to_string();
                let trimmed = line.trim_matches('"');

                let pat = (|| {
                    // https://packaging.python.org/en/latest/specifications/version-specifiers/#id4
                    // Ordering is important as we return the first match
                    let pats = ["===", "~=", "==", "!=", "<=", ">=", "<", ">"];
                    for pat in pats {
                        if trimmed.contains(pat) {
                            return Some(pat);
                        }
                    }
                    None
                })();

                // No version, name only
                let Some(pat) = pat else {
                    return Ok((trimmed.to_owned(), None));
                };
                let mut splitter = trimmed.split(pat);

                let Some(name) = splitter.next().map(str::to_string) else {
                    return Err(PDE("line without valid dependency name"));
                };
                let version = splitter.next().map(str::to_string);
                Ok((name, version))
            })
            .collect::<Result<_>>()?;
        Ok(Self(hm))
    }
}

#[tracing::instrument]
#[tokio::main]
pub async fn run() -> Result<()> {
    use UpdatePypiDepsError::*;
    let args = Config::parse();

    let input = std::fs::read_to_string(args.input)?;
    let toml: toml::Table = input.parse()?;

    let deps: PypiDeps = toml
        .get("project")
        .and_then(|t| t.get("dependencies"))
        .ok_or_else(|| ParseDepsError("no such section: project.dependencies"))?
        .try_into()?;
    dbg!(&deps);

    let test_deps: PypiDeps = toml
        .get("project")
        .and_then(|t| t.get("optional-dependencies"))
        .and_then(|t| t.get("test"))
        .ok_or_else(|| ParseDepsError("no such section: project.optional-dependencies.test"))?
        .try_into()?;
    dbg!(&test_deps);

    let dev_deps: PypiDeps = toml
        .get("project")
        .and_then(|t| t.get("optional-dependencies"))
        .and_then(|t| t.get("dev"))
        .ok_or_else(|| ParseDepsError("no such section: project.optional-dependencies.dev"))?
        .try_into()?;
    dbg!(&dev_deps);
    Ok(())
}

#[cfg(test)]
mod tests {}
