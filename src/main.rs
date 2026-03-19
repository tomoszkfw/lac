mod cli;
use anyhow::{Context, Result, anyhow};
use clap::Parser;
use std::path::Path;
use std::{env, fs};
use walkdir::WalkDir;

// constants for matching
const SUFFIX_MATCH: &[&str] = &[".run.xml", "-SAVE-ERROR"];
const EXT_MATCH: &[&str] = &[
    "aux",
    "bbl",
    "log",
    "out",
    "toc",
    "lof",
    "lot",
    "fls",
    "fdb_latexmk",
    "blg",
    "bcf",
];

fn is_latex_aux(path: &Path) -> bool {
    let name = match path.file_name().and_then(|s| s.to_str()) {
        Some(n) => n,
        None => return false,
    };

    if SUFFIX_MATCH.iter().any(|s| name.ends_with(s)) {
        return true;
    }

    let ext = match path.extension().and_then(|s| s.to_str()) {
        Some(e) => e,
        None => return false,
    };

    EXT_MATCH.contains(&ext)
}

fn main() -> Result<()> {
    let options = cli::Options::parse();
    let target_path = match options.target_dir {
        Some(path) => path,
        None => env::current_dir().context("Failed to get current directory")?,
    };

    if !target_path.exists() {
        return Err(anyhow!(
            "Target path '{}' does not exist",
            target_path.display()
        ));
    }

    if !target_path.is_dir() {
        return Err(anyhow!(
            "Target path '{}' is not a directory",
            target_path.display()
        ));
    }

    if options.dry_run {
        println!("INFO: Running in dry-run mode. No files will be deleted.");
    }

    let mut walker = if options.recursive {
        WalkDir::new(&target_path).follow_links(false).into_iter()
    } else {
        WalkDir::new(&target_path)
            .max_depth(1)
            .follow_links(false)
            .into_iter()
    };

    let mut scanned: usize = 0;
    let mut matched: usize = 0;
    let mut removed: usize = 0;
    let mut failures: Vec<String> = Vec::new();

    while let Some(entry_result) = walker.next() {
        let entry = entry_result.context("Failed to read a directory entry while traversing")?;
        let path = entry.path();
        let file_type = entry.file_type();

        scanned += 1;

        if file_type.is_symlink() {
            continue;
        }

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
        } else if file_type.is_file() && is_latex_aux(path) {
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
        "Summary: scanned={}, matched={}, removed={}, failed={}",
        scanned,
        matched,
        removed,
        failures.len()
    );

    if !failures.is_empty() {
        eprintln!("Encountered {} failure(s):", failures.len());
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
