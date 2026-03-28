mod cli;
mod matcher;
use anyhow::{Result, anyhow};
use clap::Parser;
use std::fs;
use walkdir::WalkDir;

fn main() -> Result<()> {
    let options = cli::Options::parse();
    let target_path = cli::Options::get_target_path(&options)?;

    if options.dry_run {
        println!("Running in dry-run mode. No files will be deleted.");
    }

    let walker = if options.recursive {
        WalkDir::new(&target_path)
    } else {
        WalkDir::new(&target_path).max_depth(1)
    }
    .follow_links(false)
    .into_iter();

    let mut scanned: usize = 0;
    let mut removed: usize = 0;
    let mut failure: usize = 0;

    for entry in walker.filter_map(|e| e.ok()) {
        let path = entry.path();
        let file_type = entry.file_type();

        scanned += 1;

        if file_type.is_dir() {
            if let Some(file_name) = path.file_name().and_then(|s| s.to_str())
                && file_name.starts_with("_minted")
            {
                if options.dry_run {
                    println!("Would remove directory: {}", path.display());
                } else {
                    match fs::remove_dir_all(path) {
                        Ok(()) => {
                            removed += 1;
                            println!("Removed directory: {}", path.display());
                        }
                        Err(err) => {
                            failure += 1;
                            eprintln!("Failed to remove directory {}: {}", path.display(), err);
                        }
                    }
                }
            }
        } else if file_type.is_file() && matcher::is_latex_aux(path) {
            if options.dry_run {
                println!("Would remove: {}", path.display());
            } else {
                match fs::remove_file(path) {
                    Ok(()) => {
                        removed += 1;
                    }
                    Err(err) => {
                        failure += 1;
                        eprintln!("Failed to remove file {}: {}", path.display(), err);
                    }
                }
            }
        }
    }

    println!("Scanned {} files and Removed {} files.", scanned, removed);

    if failure != 0 {
        return Err(anyhow!("Cleanup completed with {} failure(s)", failure));
    }

    Ok(())
}
