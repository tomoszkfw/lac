use crate::matcher;
use anyhow::{Result, anyhow};
use ignore::{WalkBuilder, WalkState};
use log::{error, info};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

#[derive(Default)]
struct CleanupCounters {
    scanned: AtomicUsize,
    would_remove: AtomicUsize,
    removed: AtomicUsize,
    failures: AtomicUsize,
}

pub fn run(target_path: &Path, recursive: bool, dry_run: bool, no_ignore: bool) -> Result<()> {
    let counters = Arc::new(CleanupCounters::default());
    let mut walker = WalkBuilder::new(target_path);

    walker.follow_links(false);

    if no_ignore {
        walker
            .git_ignore(false)
            .git_exclude(false)
            .git_global(false)
            .ignore(false)
            .parents(false);
    }

    if !recursive {
        walker.max_depth(Some(1));
    }

    walker.build_parallel().run(|| {
        let counters = Arc::clone(&counters);
        Box::new(move |entry| process_entry(entry, dry_run, &counters))
    });

    let scanned = counters.scanned.load(Relaxed);
    let would_remove = counters.would_remove.load(Relaxed);
    let removed = counters.removed.load(Relaxed);
    let failures = counters.failures.load(Relaxed);
    info!(
        "{}",
        summary_message(scanned, would_remove, removed, dry_run)
    );

    if failures != 0 {
        return Err(anyhow!("Cleanup completed with {failures} failures"));
    }
    Ok(())
}

fn summary_message(scanned: usize, would_remove: usize, removed: usize, dry_run: bool) -> String {
    if dry_run {
        format!("Scanned {scanned} entries and would remove {would_remove} entries.")
    } else {
        format!("Scanned {scanned} entries and removed {removed} entries.")
    }
}

fn process_entry(
    entry: Result<ignore::DirEntry, ignore::Error>,
    dry_run: bool,
    counters: &CleanupCounters,
) -> WalkState {
    let entry = match entry {
        Ok(entry) => entry,
        Err(err) => {
            counters.failures.fetch_add(1, Relaxed);
            error!("Failed to traverse directory entry: {err}");
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
        counters.would_remove.fetch_add(1, Relaxed);
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

#[cfg(test)]
mod tests {
    use super::{CleanupCounters, process_entry, run, summary_message};
    use ignore::{WalkBuilder, WalkState};
    use std::fs;
    use std::io;
    use std::path::PathBuf;
    use std::sync::atomic::Ordering::Relaxed;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn summary_reports_planned_deletions_in_dry_run() {
        assert_eq!(
            summary_message(12, 3, 0, true),
            "Scanned 12 entries and would remove 3 entries."
        );
    }

    #[test]
    fn summary_reports_successful_deletions_in_normal_mode() {
        assert_eq!(
            summary_message(12, 3, 2, false),
            "Scanned 12 entries and removed 2 entries."
        );
    }

    #[test]
    fn dry_run_counts_matching_files_without_removing_them() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after UNIX epoch")
            .as_nanos();
        let target_dir = std::env::temp_dir().join(format!(
            "lac-dry-run-summary-test-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&target_dir).expect("test directory should be created");

        let aux_path = target_dir.join("paper.aux");
        fs::write(&aux_path, "auxiliary data").expect("test file should be created");
        let entry = WalkBuilder::new(&target_dir)
            .build()
            .filter_map(Result::ok)
            .find(|entry| entry.path() == aux_path.as_path())
            .expect("walker should find the auxiliary file");

        let counters = CleanupCounters::default();
        let state = process_entry(Ok(entry), true, &counters);

        assert!(matches!(state, WalkState::Continue));
        assert_eq!(counters.scanned.load(Relaxed), 1);
        assert_eq!(counters.would_remove.load(Relaxed), 1);
        assert_eq!(counters.removed.load(Relaxed), 0);
        assert!(aux_path.is_file(), "dry-run must preserve the file");

        fs::remove_dir_all(&target_dir).expect("test directory should be cleaned up");
    }

    #[test]
    fn traversal_errors_are_counted_as_failures() {
        let counters = CleanupCounters::default();
        let err: ignore::Error = io::Error::other("synthetic traversal error").into();

        let state = process_entry(Err(err), false, &counters);

        assert!(matches!(state, WalkState::Continue));
        assert_eq!(counters.failures.load(Relaxed), 1);
        assert_eq!(counters.scanned.load(Relaxed), 0);
        assert_eq!(counters.removed.load(Relaxed), 0);
    }

    #[test]
    fn run_returns_an_error_when_traversal_fails() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after UNIX epoch")
            .as_nanos();
        let target_path: PathBuf = std::env::temp_dir().join(format!(
            "lac-nonexistent-traversal-test-{}-{nonce}",
            std::process::id()
        ));
        assert!(!target_path.exists());

        let result = run(&target_path, true, false, false);

        assert!(result.is_err());
    }
}
