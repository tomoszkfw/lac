use crate::matcher;
use anyhow::{Result, anyhow};
use ignore::{WalkBuilder, WalkState};
use log::{debug, error, info};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

#[derive(Default)]
struct CleanupCounters {
    scanned: AtomicUsize,
    removed: AtomicUsize,
    failures: AtomicUsize,
}

pub fn run(target_path: &Path, recursive: bool, dry_run: bool) -> Result<()> {
    let counters = Arc::new(CleanupCounters::default());
    let mut walker = WalkBuilder::new(target_path);
    walker.follow_links(false);

    if !recursive {
        walker.max_depth(Some(1));
    }

    if recursive {
        let counters = Arc::clone(&counters);
        walker.build_parallel().run(move || {
            let counters = Arc::clone(&counters);
            Box::new(move |entry| process_entry(entry, dry_run, &counters))
        });
    } else {
        for entry in walker.build() {
            let _ = process_entry(entry, dry_run, &counters);
        }
    }

    let scanned = counters.scanned.load(Relaxed);
    let removed = counters.removed.load(Relaxed);
    let failures = counters.failures.load(Relaxed);
    info!("Scanned {scanned} entries and removed {removed} entries.");

    if failures != 0 {
        return Err(anyhow!("Cleanup completed with {failures} failures"));
    }
    Ok(())
}

fn process_entry(
    entry: Result<ignore::DirEntry, ignore::Error>,
    dry_run: bool,
    counters: &CleanupCounters,
) -> WalkState {
    let entry = match entry {
        Ok(entry) => entry,
        Err(err) => {
            debug!("Skipping traversal error: {err}");
            return WalkState::Continue;
        }
    };

    counters.scanned.fetch_add(1, Relaxed);
    let Some(kind) = matcher::classify(entry.path(), entry.file_type()) else {
        return WalkState::Continue;
    };

    let is_dir = matches!(kind, matcher::TargetKind::Directory);
    let kind = if is_dir { "directory" } else { "file" };
    if dry_run {
        info!("Would remove {kind}: {}", entry.path().display());
    } else {
        let result = if is_dir {
            fs::remove_dir_all(entry.path())
        } else {
            fs::remove_file(entry.path())
        };
        match result {
            Ok(()) => {
                counters.removed.fetch_add(1, Relaxed);
                info!("Removed {kind}: {}", entry.path().display());
            }
            Err(err) => {
                counters.failures.fetch_add(1, Relaxed);
                error!("Failed to remove {kind} {}: {err}", entry.path().display());
            }
        }
    }

    if is_dir {
        WalkState::Skip
    } else {
        WalkState::Continue
    }
}
