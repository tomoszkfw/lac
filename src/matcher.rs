use std::path::Path;

// constants for matching
const SUFFIX_MATCH: &[&str] = &[".run.xml", "-SAVE-ERROR"];
const EXT_MATCH: &[&str] = &[
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

pub fn is_latex_aux(path: &Path) -> bool {
    let name = match path.file_name().and_then(|s| s.to_str()) {
        Some(n) => n,
        None => return false,
    };

    if SUFFIX_MATCH.iter().any(|s| name.ends_with(s)) {
        return true;
    }

    path.extension()
        .and_then(|s| s.to_str())
        .is_some_and(|ext| EXT_MATCH.contains(&ext))
}
