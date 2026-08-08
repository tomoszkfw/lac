use crate::matcher::{self, TargetKind::*};
use anyhow::{Result, anyhow};
use ignore::{WalkBuilder, WalkState};
use log::{debug, error, info};
use rayon::prelude::*;
use std::collections::HashSet;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;

pub enum DeletionTarget {
    File(PathBuf),
    Directory(PathBuf),
}

impl DeletionTarget {
    fn path(&self) -> &Path {
        match self {
            Self::File(p) | Self::Directory(p) => p,
        }
    }
    fn kind_str(&self) -> &'static str {
        match self {
            Self::File(_) => "file",
            Self::Directory(_) => "directory",
        }
    }
}

#[derive(Default)]
struct ThreadLocalState {
    tex_sources: Vec<(PathBuf, OsString)>,
    aux_candidates: Vec<(PathBuf, PathBuf, OsString)>,
    minted_dirs: Vec<PathBuf>,
    scanned_count: usize,
}

struct Worker {
    state: ThreadLocalState,
    sender: mpsc::Sender<ThreadLocalState>,
}

impl Drop for Worker {
    fn drop(&mut self) {
        let _ = self.sender.send(std::mem::take(&mut self.state));
    }
}

struct DiscoveryResult {
    tex_sources: HashSet<(PathBuf, OsString)>,
    aux_candidates: Vec<(PathBuf, PathBuf, OsString)>,
    minted_dirs: Vec<PathBuf>,
    scanned_count: usize,
}

fn discover_candidates(target: &Path, recursive: bool, no_ignore: bool) -> Result<DiscoveryResult> {
    let mut walker = WalkBuilder::new(target);
    walker
        .follow_links(false)
        .max_depth(if recursive { None } else { Some(1) });

    if no_ignore {
        walker
            .git_ignore(false)
            .git_exclude(false)
            .git_global(false)
            .ignore(false)
            .parents(false);
    }

    let (sender, receiver) = mpsc::channel();

    walker.build_parallel().run(|| {
        let mut worker = Worker {
            state: ThreadLocalState::default(),
            sender: sender.clone(),
        };

        Box::new(move |entry| {
            let Ok(dir_entry) = entry else {
                debug!("Skipping error");
                return WalkState::Continue;
            };

            worker.state.scanned_count += 1;
            let path = dir_entry.path();

            if let Some(kind) = matcher::classify(path, dir_entry.file_type()) {
                let parent = path.parent().unwrap_or(Path::new("")).to_path_buf();
                match kind {
                    MintedDir => {
                        worker.state.minted_dirs.push(path.to_path_buf());
                        return WalkState::Skip;
                    }
                    TexSource(s) => worker.state.tex_sources.push((parent, s)),
                    AuxCandidate(s) => {
                        worker
                            .state
                            .aux_candidates
                            .push((path.to_path_buf(), parent, s))
                    }
                }
            }
            WalkState::Continue
        })
    });
    drop(sender);

    let (mut tex_sources, mut aux_candidates, mut minted_dirs, mut scanned_count) =
        (HashSet::new(), Vec::new(), Vec::new(), 0);

    for mut state in receiver {
        tex_sources.extend(state.tex_sources);
        aux_candidates.append(&mut state.aux_candidates);
        minted_dirs.append(&mut state.minted_dirs);
        scanned_count += state.scanned_count;
    }

    Ok(DiscoveryResult {
        tex_sources,
        aux_candidates,
        minted_dirs,
        scanned_count,
    })
}

fn filter_targets(discovery: DiscoveryResult) -> Vec<DeletionTarget> {
    let mut targets: Vec<DeletionTarget> = discovery
        .minted_dirs
        .into_iter()
        .map(DeletionTarget::Directory)
        .collect();

    targets.par_extend(discovery.aux_candidates.into_par_iter().filter_map(
        |(path, parent, stem)| {
            discovery
                .tex_sources
                .contains(&(parent, stem))
                .then_some(DeletionTarget::File(path))
        },
    ));

    targets
}

struct ExecutionResult {
    removed: usize,
    failures: usize,
}

fn execute_deletion(targets: Vec<DeletionTarget>, dry_run: bool) -> ExecutionResult {
    let (removed, failures) = targets
        .into_par_iter()
        .map(|target| {
            let (path, kind_str) = (target.path(), target.kind_str());

            if dry_run {
                info!("Would remove {}: {}", kind_str, path.display());
                return (0, 0);
            }

            let result = match &target {
                DeletionTarget::Directory(p) => fs::remove_dir_all(p),
                DeletionTarget::File(p) => fs::remove_file(p),
            };

            if let Err(e) = result {
                error!("Failed to remove {}: {}", kind_str, e);
                (0, 1)
            } else {
                info!("Removed {}: {}", kind_str, path.display());
                (1, 0)
            }
        })
        .reduce(|| (0, 0), |a, b| (a.0 + b.0, a.1 + b.1));

    ExecutionResult { removed, failures }
}

pub fn run(target: &Path, recursive: bool, dry_run: bool, no_ignore: bool) -> Result<()> {
    let discovery = discover_candidates(target, recursive, no_ignore)?;
    let scanned_count = discovery.scanned_count;
    let execution = execute_deletion(filter_targets(discovery), dry_run);

    info!(
        "Scanned {} entries and removed {} entries.",
        scanned_count, execution.removed
    );

    if execution.failures == 0 {
        Ok(())
    } else {
        Err(anyhow!(
            "Cleanup completed with {} failures",
            execution.failures
        ))
    }
}
