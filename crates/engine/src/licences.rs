use std::path::{Path, PathBuf};

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
}
