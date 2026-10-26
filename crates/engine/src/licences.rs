use std::path::{Path, PathBuf};

use opsis_types::engine::LicenceEntry;

use crate::files::read_text_if_present;
use crate::paths::{CHROMIUM_CREDITS_FILE, ENGINE_LICENCES_FOLDER};

pub fn shipped_credits(roots: &[PathBuf]) -> Option<PathBuf> {
    roots
        .iter()
        .map(|root| {
            root.join(ENGINE_LICENCES_FOLDER)
                .join(CHROMIUM_CREDITS_FILE)
        })
        .find(|file| file.is_file())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LicenceKind {
    Text,
    File,
}

#[derive(Debug, Clone)]
pub struct LicenceSource {
    pub id: &'static str,
    pub title: &'static str,
    pub summary: &'static str,
    pub kind: LicenceKind,
    pub path: PathBuf,
}

fn entry_for(source: &LicenceSource) -> std::io::Result<Option<LicenceEntry>> {
    let entry = |text: Option<String>, file: Option<String>| LicenceEntry {
        id: source.id.to_owned(),
        title: source.title.to_owned(),
        summary: source.summary.to_owned(),
        text,
        file,
    };
    Ok(match source.kind {
        LicenceKind::File => source
            .path
            .is_file()
            .then(|| entry(None, Some(source.path.to_string_lossy().into_owned()))),
        LicenceKind::Text => read_text_if_present(&source.path)?
            .filter(|t| !t.is_empty())
            .map(|t| entry(Some(t), None)),
    })
}

pub fn list_licences(sources: &[LicenceSource]) -> std::io::Result<Vec<LicenceEntry>> {
    sources
        .iter()
        .map(entry_for)
        .filter_map(Result::transpose)
        .collect()
}

pub fn is_licence_file(sources: &[LicenceSource], file: &Path) -> bool {
    sources
        .iter()
        .any(|s| s.kind == LicenceKind::File && s.path == file)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn finds_the_credits_that_ship_with_the_app() {
        let root = tempfile::tempdir().unwrap();
        let roots = [root.path().join("resources"), root.path().to_path_buf()];
        assert_eq!(shipped_credits(&roots), None);
        let file = root
            .path()
            .join(ENGINE_LICENCES_FOLDER)
            .join(CHROMIUM_CREDITS_FILE);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, "<html>credits</html>").unwrap();
        assert_eq!(shipped_credits(&roots), Some(file));
    }

    #[test]
    fn lists_the_licences_that_are_there_and_opens_only_licence_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("LICENSE"), "Apache").unwrap();
        std::fs::write(dir.path().join("credits.html"), "<p>").unwrap();
        let source = |id, kind, file: &str| LicenceSource {
            id,
            title: "T",
            summary: "S",
            kind,
            path: dir.path().join(file),
        };
        let sources = [
            source("opsis", LicenceKind::Text, "LICENSE"),
            source("gone", LicenceKind::Text, "NOTICES.md"),
            source("chromium", LicenceKind::File, "credits.html"),
            source("ffmpeg", LicenceKind::File, "ffmpeg/LICENSE.txt"),
        ];
        let listed = list_licences(&sources).unwrap();
        assert_eq!(
            listed.iter().map(|e| e.id.as_str()).collect::<Vec<_>>(),
            ["opsis", "chromium"]
        );
        assert_eq!(listed[0].text.as_deref(), Some("Apache"));
        assert_eq!(
            listed[1].file.as_deref(),
            Some(dir.path().join("credits.html").to_string_lossy().as_ref())
        );
        assert!(is_licence_file(&sources, &dir.path().join("credits.html")));
        assert!(!is_licence_file(&sources, &dir.path().join("LICENSE")));
    }
}
