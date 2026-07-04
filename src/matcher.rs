use TargetKind::*;
use std::{ffi::OsString, fs::FileType, path::Path};

pub enum TargetKind {
    MintedDir,
    TexSource(OsString),
    AuxCandidate(OsString),
}

pub fn classify(path: &Path, ft: Option<FileType>) -> Option<TargetKind> {
    let name = path.file_name()?.to_str()?;

    if ft?.is_dir() {
        return name.starts_with("_minted").then_some(MintedDir);
    }

    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        if ext == "tex" {
            return Some(TexSource(path.file_stem()?.to_os_string()));
        }
        if matches!(
            ext,
            "aux"
                | "bbl"
                | "bcf"
                | "blg"
                | "fdb_latexmk"
                | "fls"
                | "lof"
                | "log"
                | "lot"
                | "nav"
                | "out"
                | "snm"
                | "synctex(busy)"
                | "toc"
                | "vrb"
        ) {
            return Some(AuxCandidate(path.file_stem()?.to_os_string()));
        }
    }

    name.strip_suffix(".run.xml")
        .or_else(|| name.strip_suffix("-SAVE-ERROR"))
        .map(|b| AuxCandidate(OsString::from(b)))
}
