mod cli;
mod matcher;
use anyhow::{Context, Result, anyhow};
use clap::Parser;
use std::{env, fs};
use walkdir::WalkDir;

fn main() -> Result<()> {
    let options = cli::Options::parse();
    let target_path = options
        .target_dir
        .unwrap_or(env::current_dir().context("Failed to get current working directory")?);

    if !target_path.exists() || !target_path.is_dir() {
        return Err(anyhow!(
            "ERROR: Target path '{}' does not exist or is not a directory.",
            target_path.display()
        ));
    }

    if options.dry_run {
        println!("INFO: Running in dry-run mode. No files will be deleted.");
    }

    let mut walker = if options.recursive {
        WalkDir::new(&target_path)
    } else {
        WalkDir::new(&target_path).max_depth(1)
    }
    .follow_links(false)
    .into_iter();

    let mut scanned: usize = 0;
    let mut matched: usize = 0;
    let mut removed: usize = 0;
    let mut failures: Vec<String> = Vec::new();

    while let Some(entry_result) = walker.next() {
        let entry = entry_result.context("Failed to read a directory entry while traversing")?;
        let path = entry.path();
        let file_type = entry.file_type();

        scanned += 1;

        if file_type.is_dir() {
            if path.file_name().and_then(|s| s.to_str()) == Some("_minted") {
                matched += 1;
                if options.dry_run {
                    println!("Would remove directory: {}", path.display());
                } else {
                    match fs::remove_dir_all(path) {
                        Ok(()) => {
                            removed += 1;
                            println!("Removed directory: {}", path.display());
                        }
                        Err(err) => {
                            failures.push(format!(
                                "Failed to remove directory {}: {}",
                                path.display(),
                                err
                            ));
                        }
                    }
                }
                walker.skip_current_dir();
                continue;
            }
        } else if file_type.is_file() && matcher::is_latex_aux(path) {
            matched += 1;
            if options.dry_run {
                println!("Would remove: {}", path.display());
            } else {
                match fs::remove_file(path) {
                    Ok(()) => {
                        removed += 1;
                        println!("Removed: {}", path.display());
                    }
                    Err(err) => {
                        failures.push(format!("Failed to remove file {}: {}", path.display(), err));
                    }
                }
            }
        }
    }

    println!(
        "Summary: Scanned {} files, and found {} files matched. Removed {} files.",
        scanned, matched, removed
    );

    if !failures.is_empty() {
        eprintln!("Summary: Encountered {} failure(s):", failures.len());
        for failure in &failures {
            eprintln!("  - {}", failure);
        }
        return Err(anyhow!(
            "Cleanup completed with {} failure(s)",
            failures.len()
        ));
    }

    Ok(())
}
