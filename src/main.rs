mod cleanup;
mod cli;
mod logging;
mod matcher;
use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let options = cli::Options::parse();
    logging::init(options.verbose)?;
    let target_path = cli::Options::get_target_path(&options)?;
    cleanup::run(
        &target_path,
        options.recursive,
        options.dry_run,
        options.no_ignore,
    )?;

    Ok(())
}
