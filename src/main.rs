use tracing::level_filters::LevelFilter;
use tracing_subscriber::{self, EnvFilter};

use update_pypi_deps::{run, Result};

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::WARN.into())
                .from_env_lossy()
                .add_directive("hyper=warn".parse()?)
                .add_directive("reqwest=warn".parse()?),
        )
        .init();

    run()
}
