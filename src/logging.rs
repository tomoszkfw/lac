use anyhow::{Context, Result};

pub fn init(verbose: bool) -> Result<()> {
    let default_level = if verbose { "debug" } else { "info" };

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(default_level))
        .try_init()
        .context("failed to initialize logger")?;

    Ok(())
}
