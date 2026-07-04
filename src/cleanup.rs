use crate::matcher::{self, TargetKind::*};
use anyhow::{Result, anyhow};
use ignore::{WalkBuilder, WalkState};
use log::{debug, error, info};
use rayon::prelude::*;
use std::collections::HashSet;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
use std::sync::mpsc;

#[derive(Default)]
struct Counters {
    scanned: AtomicUsize,
    removed: AtomicUsize,
    failures: AtomicUsize,
}

#[derive(Default)]
struct State {
    tex: Vec<(PathBuf, OsString)>,
    aux: Vec<(PathBuf, PathBuf, OsString)>,
    dirs: Vec<PathBuf>,
}

struct Worker(State, mpsc::Sender<State>, Arc<Counters>);

impl Drop for Worker {
    fn drop(&mut self) {
        let _ = self.1.send(std::mem::take(&mut self.0));
    }
}

pub fn run(target: &Path, rec: bool, dry: bool, no_ign: bool) -> Result<()> {
    let counters = Arc::new(Counters::default());
    let mut walker = WalkBuilder::new(target);
    walker
        .follow_links(false)
        .max_depth(if rec { None } else { Some(1) });
    if no_ign {
        walker
            .git_ignore(false)
            .git_exclude(false)
            .git_global(false)
            .ignore(false)
            .parents(false);
    }

    let (tx, rx) = mpsc::channel();
    walker.build_parallel().run(|| {
        let mut w = Worker(State::default(), tx.clone(), counters.clone());
        Box::new(move |e| {
            let Ok(e) = e else {
                debug!("Skipping error");
                return WalkState::Continue;
            };
            w.2.scanned.fetch_add(1, Relaxed);

            let path = e.path();
            if let Some(kind) = matcher::classify(path, e.file_type()) {
                let parent = path.parent().unwrap_or(Path::new("")).to_path_buf();
                return match kind {
                    MintedDir => {
                        w.0.dirs.push(path.to_path_buf());
                        WalkState::Skip
                    }
                    TexSource(s) => {
                        w.0.tex.push((parent, s));
                        WalkState::Continue
                    }
                    AuxCandidate(s) => {
                        w.0.aux.push((path.to_path_buf(), parent, s));
                        WalkState::Continue
                    }
                };
            }
            WalkState::Continue
        })
    });
    drop(tx);

    let (mut tex_set, mut all_aux, mut targets) = (HashSet::new(), Vec::new(), Vec::new());
    for mut s in rx {
        tex_set.extend(s.tex);
        all_aux.append(&mut s.aux);
        targets.extend(s.dirs.into_iter().map(|p| (p, true)));
    }

    targets.par_extend(
        all_aux.into_par_iter().filter_map(|(p, parent, stem)| {
            tex_set.contains(&(parent, stem)).then_some((p, false))
        }),
    );

    targets.into_par_iter().for_each(|(p, is_dir)| {
        let kind = if is_dir { "directory" } else { "file" };
        if dry {
            info!("Would remove {kind}: {}", p.display());
        } else {
            match if is_dir {
                fs::remove_dir_all(&p)
            } else {
                fs::remove_file(&p)
            } {
                Ok(_) => {
                    counters.removed.fetch_add(1, Relaxed);
                    info!("Removed {kind}: {}", p.display());
                }
                Err(e) => {
                    counters.failures.fetch_add(1, Relaxed);
                    error!("Failed to remove {kind}: {e}");
                }
            }
        }
    });

    let (s, r, f) = (
        counters.scanned.load(Relaxed),
        counters.removed.load(Relaxed),
        counters.failures.load(Relaxed),
    );
    info!("Scanned {s} entries and removed {r} entries.");
    (f == 0)
        .then_some(())
        .ok_or_else(|| anyhow!("Cleanup completed with {f} failures"))
}
