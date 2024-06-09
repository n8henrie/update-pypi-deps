use crate::Result;

use tracing::debug;

mod response;
use response::PypiResponse;

pub(crate) async fn find_latest(name: &str) -> Result<String> {
    let url = format!("https://pypi.org/pypi/{}/json", name);
    debug!("sending request to {}", url);
    let package: PypiResponse = reqwest::get(&url).await?.json().await?;
    Ok(package.info.version)
}

#[cfg(test)]
mod tests;
