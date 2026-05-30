use std::fs;
use std::path::Path;

// constants for matching
const SUFFIX_MATCH: [&str; 2] = [".run.xml", "-SAVE-ERROR"];
const EXT_MATCH: [&str; 15] = [
    "aux",
    "bbl",
    "bcf",
    "blg",
    "fdb_latexmk",
    "fls",
    "lof",
    "log",
    "lot",
    "nav",
    "out",
    "snm",
    "synctex(busy)",
    "toc",
    "vrb",
];

pub enum TargetKind {
    Directory,
    File,
}

pub fn is_minted_dir_name(name: &str) -> bool {
    name.starts_with("_minted")
}

pub fn is_latex_aux_name(name: &str, extension: Option<&str>) -> bool {
    if SUFFIX_MATCH.iter().any(|suffix| name.ends_with(suffix)) {
        return true;
    }

    extension.is_some_and(|ext| EXT_MATCH.contains(&ext))
}

pub fn classify(path: &Path, file_type: Option<fs::FileType>) -> Option<TargetKind> {
    let ft = file_type?;
    let name = path.file_name().and_then(|n| n.to_str())?;
    if ft.is_dir() && is_minted_dir_name(name) {
        return Some(TargetKind::Directory);
    }
    if ft.is_file() && is_latex_aux_name(name, path.extension().and_then(|e| e.to_str())) {
        return Some(TargetKind::File);
    }
    None
}
