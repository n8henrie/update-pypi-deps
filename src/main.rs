#![warn(clippy::pedantic)]

use tracing_subscriber::{self, EnvFilter};

use update_pypi_deps::{run, Result};

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("hyper=warn".parse()?)
                .add_directive("reqwest=warn".parse()?),
        )
        .init();

    run()
}
