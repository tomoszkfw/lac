use anyhow::{Context, Result};

pub fn init(verbose: bool) -> Result<()> {
    let level = if verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };

    env_logger::Builder::from_env(env_logger::Env::default())
        .filter_level(level)
        .try_init()
        .context("failed to initialize logger")?;

    Ok(())
}
