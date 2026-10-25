use std::path::{Path, PathBuf};

use opsis_types::engine::LicenceEntry;

use crate::files::read_text_if_present;
use crate::paths::{CHROMIUM_CREDITS_FILE, ENGINE_LICENCES_FOLDER, ShellPlatform, engine_status};

pub fn shipped_credits_candidates(roots: &[PathBuf]) -> Vec<PathBuf> {
    roots
        .iter()
        .flat_map(|root| {
            [
                root.join(ENGINE_LICENCES_FOLDER)
                    .join(CHROMIUM_CREDITS_FILE),
                root.join("engine").join(CHROMIUM_CREDITS_FILE),
            ]
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Licences {
    Present,
    Copied,
    Missing,
    NoEngine,
}

pub fn complete_engine_licences(
    engine_dir: &Path,
    candidates: &[PathBuf],
    platform: ShellPlatform,
) -> std::io::Result<Licences> {
    if !engine_status(engine_dir, platform).complete {
        return Ok(Licences::NoEngine);
    }
    let target = engine_dir.join(CHROMIUM_CREDITS_FILE);
    if target.exists() {
        return Ok(Licences::Present);
    }
    let same = |c: &Path| std::path::absolute(c).ok() == std::path::absolute(&target).ok();
    let Some(source) = candidates.iter().find(|c| !same(c) && c.exists()) else {
        return Ok(Licences::Missing);
    };
    std::fs::copy(source, &target)?;
    Ok(Licences::Copied)
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
    use crate::paths::engine_paths;

    fn fake_engine(dir: &Path) {
        let p = engine_paths(dir, ShellPlatform::current().unwrap());
        for file in [&p.headless_shells[0], &p.ffmpeg, &p.ffprobe] {
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            std::fs::write(file, "").unwrap();
        }
    }

    #[test]
    fn copies_the_credits_that_ship_with_the_app_into_a_set_up_engine() {
        let root = tempfile::tempdir().unwrap();
        let engine = root.path().join("user").join("engine");
        let platform = ShellPlatform::current().unwrap();
        fake_engine(&engine);
        let shipped = shipped_credits_candidates(&[root.path().join("resources")]);
        std::fs::create_dir_all(shipped[0].parent().unwrap()).unwrap();
        std::fs::write(&shipped[0], "<html>credits</html>").unwrap();
        assert_eq!(
            complete_engine_licences(&engine, &shipped, platform).unwrap(),
            Licences::Copied
        );
        assert_eq!(
            std::fs::read_to_string(engine.join(CHROMIUM_CREDITS_FILE)).unwrap(),
            "<html>credits</html>"
        );
        assert_eq!(
            complete_engine_licences(&engine, &shipped, platform).unwrap(),
            Licences::Present
        );
    }

    #[test]
    fn does_nothing_without_an_engine_or_without_shipped_credits() {
        let root = tempfile::tempdir().unwrap();
        let engine = root.path().join("engine");
        let platform = ShellPlatform::current().unwrap();
        assert_eq!(
            complete_engine_licences(&engine, &[], platform).unwrap(),
            Licences::NoEngine
        );
        fake_engine(&engine);
        let candidates = shipped_credits_candidates(&[root.path().to_path_buf()]);
        assert_eq!(
            complete_engine_licences(&engine, &candidates, platform).unwrap(),
            Licences::Missing
        );
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
