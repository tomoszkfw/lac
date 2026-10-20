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

#[cfg(test)]
mod tests {
    use super::{is_latex_aux_name, is_minted_dir_name};

    #[test]
    fn matches_known_filename_suffixes() {
        assert!(is_latex_aux_name("paper.run.xml", None));

        // bibliography-related files
        assert!(is_latex_aux_name(
            "chapter.bbl-SAVE-ERROR",
            Some("bbl-SAVE-ERROR")
        ));
        assert!(is_latex_aux_name(
            "chapter.bcf-SAVE-ERROR",
            Some("bcf-SAVE-ERROR")
        ));
    }

    #[test]
    fn matches_every_configured_extension() {
        let extensions = [
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

        for extension in extensions {
            let name = format!("paper.{extension}");
            assert!(
                is_latex_aux_name(&name, Some(extension)),
                "expected {name} to be recognized as an auxiliary file"
            );
        }
    }

    #[test]
    fn does_not_match_unrelated_files() {
        let cases = [
            ("paper.tex", Some("tex")),
            ("references.bib", Some("bib")),
            ("paper.pdf", Some("pdf")),
            ("paper.aux.tex", Some("tex")),
            ("README", None),
            ("paper.AUX", Some("AUX")),
        ];

        for (name, extension) in cases {
            assert!(
                !is_latex_aux_name(name, extension),
                "did not expect {name} to match"
            );
        }
    }

    #[test]
    fn recognizes_minted_directory_prefix() {
        assert!(is_minted_dir_name("_minted-paper"));
        assert!(is_minted_dir_name("_minted"));
    }

    #[test]
    fn rejects_unrelated_directory_names() {
        assert!(!is_minted_dir_name("minted"));
        assert!(!is_minted_dir_name("paper_minted"));
        assert!(!is_minted_dir_name("paper"));
    }
}
