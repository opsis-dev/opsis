use std::path::{Path, PathBuf};

pub const FFMPEG_PROGRAM: &str = "opsis-ffmpeg";

pub const FFMPEG_LICENCE_FILE: &str = "LICENSE.ffmpeg.txt";

#[cfg(debug_assertions)]
const FFMPEG_ENV: &str = "OPSIS_FFMPEG";

#[derive(Debug, thiserror::Error)]
#[error("Opsis's video tools are missing. Install Opsis again.")]
pub struct FfmpegMissing;

pub fn bundled_ffmpeg() -> Result<PathBuf, FfmpegMissing> {
    #[cfg(debug_assertions)]
    if let Some(given) = std::env::var_os(FFMPEG_ENV) {
        return given_ffmpeg(PathBuf::from(given));
    }
    let exe = std::env::current_exe().map_err(|_| FfmpegMissing)?;
    ffmpeg_beside(&exe)
}

#[cfg(debug_assertions)]
fn given_ffmpeg(path: PathBuf) -> Result<PathBuf, FfmpegMissing> {
    if path.is_absolute() && path.is_file() {
        Ok(path)
    } else {
        Err(FfmpegMissing)
    }
}

fn ffmpeg_beside(exe: &Path) -> Result<PathBuf, FfmpegMissing> {
    let file = exe
        .parent()
        .ok_or(FfmpegMissing)?
        .join(format!("{FFMPEG_PROGRAM}{}", std::env::consts::EXE_SUFFIX));
    if file.is_file() {
        Ok(file)
    } else {
        Err(FfmpegMissing)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_program_beside_the_app_and_nowhere_else() {
        let dir = tempfile::tempdir().unwrap();
        let app = dir.path().join("opsis");
        assert!(ffmpeg_beside(&app).is_err());
        let program = dir
            .path()
            .join(format!("{FFMPEG_PROGRAM}{}", std::env::consts::EXE_SUFFIX));
        std::fs::write(&program, b"").unwrap();
        assert_eq!(ffmpeg_beside(&app).unwrap(), program);
    }

    #[test]
    fn takes_only_an_absolute_path_to_a_file_from_the_variable() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("ffmpeg");
        std::fs::write(&file, b"").unwrap();
        assert_eq!(given_ffmpeg(file.clone()).unwrap(), file);
        assert!(given_ffmpeg(PathBuf::from("ffmpeg")).is_err());
        assert!(given_ffmpeg(dir.path().to_path_buf()).is_err());
        assert!(given_ffmpeg(dir.path().join("none")).is_err());
    }
}
