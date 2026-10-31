use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

use crate::downloads::Pinned;
use crate::install::{InstallError, make_executable};
use crate::pin::verified_download;

pub const FFMPEG_RELEASE: &str = "b6.1.1";

pub const FFMPEG_TARGETS: [(&str, &str); 5] = [
    ("x86_64-unknown-linux-gnu", "linux-x64"),
    ("aarch64-unknown-linux-gnu", "linux-arm64"),
    ("x86_64-apple-darwin", "darwin-x64"),
    ("aarch64-apple-darwin", "darwin-arm64"),
    ("x86_64-pc-windows-msvc", "win32-x64"),
];

pub const FFMPEG_PROGRAM: &str = "opsis-ffmpeg";

pub const FFMPEG_LICENCE_FILE: &str = "LICENSE.ffmpeg.txt";

#[cfg(debug_assertions)]
const FFMPEG_ENV: &str = "OPSIS_FFMPEG";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FfmpegTarget {
    pub target: String,
    pub asset: String,
    pub program: Pinned,
    pub licence: Pinned,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FfmpegPins {
    pub release: String,
    pub targets: Vec<FfmpegTarget>,
}

impl FfmpegPins {
    pub fn target(&self, target: &str) -> Option<&FfmpegTarget> {
        self.targets.iter().find(|t| t.target == target)
    }
}

static PINS: LazyLock<FfmpegPins> = LazyLock::new(|| {
    serde_json::from_str(include_str!("ffmpeg.generated.json")).expect("pinned FFmpeg parses")
});

pub fn ffmpeg_pins() -> &'static FfmpegPins {
    &PINS
}

pub fn sidecar_file_name(target: &str) -> String {
    let exe = if target.contains("-windows-") {
        ".exe"
    } else {
        ""
    };
    format!("{FFMPEG_PROGRAM}-{target}{exe}")
}

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StagedFfmpeg {
    pub ffmpeg: PathBuf,
    pub licence: PathBuf,
}

fn gunzip(archive: &Path, dest: &Path) -> std::io::Result<()> {
    let mut decoder = flate2::read::GzDecoder::new(std::fs::File::open(archive)?);
    let dir = dest.parent().unwrap_or(Path::new("."));
    let mut out = tempfile::Builder::new()
        .prefix(".partial-")
        .tempfile_in(dir)?;
    std::io::copy(&mut decoder, &mut out)?;
    out.persist(dest).map_err(|e| e.error)?;
    Ok(())
}

pub async fn stage_ffmpeg(
    target: &str,
    cache: &Path,
    binaries: &Path,
    licence: &Path,
) -> Result<StagedFfmpeg, InstallError> {
    let pins = ffmpeg_pins()
        .target(target)
        .ok_or_else(|| InstallError::NoFfmpeg(target.to_owned()))?;
    let client = reqwest::Client::new();
    let archive = verified_download(&client, &pins.program, cache).await?;
    let licence_text = verified_download(&client, &pins.licence, cache).await?;
    std::fs::create_dir_all(binaries)?;
    let ffmpeg = binaries.join(sidecar_file_name(target));
    gunzip(&archive, &ffmpeg)?;
    make_executable(&ffmpeg)?;
    if let Some(folder) = licence.parent() {
        std::fs::create_dir_all(folder)?;
    }
    std::fs::write(licence, std::fs::read(&licence_text)?)?;
    Ok(StagedFfmpeg {
        ffmpeg,
        licence: licence.to_path_buf(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_the_sidecar_the_way_tauri_looks_for_it() {
        assert_eq!(
            sidecar_file_name("x86_64-unknown-linux-gnu"),
            "opsis-ffmpeg-x86_64-unknown-linux-gnu"
        );
        assert_eq!(
            sidecar_file_name("x86_64-pc-windows-msvc"),
            "opsis-ffmpeg-x86_64-pc-windows-msvc.exe"
        );
    }

    #[test]
    fn pins_a_program_and_its_licence_for_every_target() {
        let pins = ffmpeg_pins();
        assert_eq!(pins.targets.len(), FFMPEG_TARGETS.len());
        for (target, _) in FFMPEG_TARGETS {
            let pinned = pins.target(target).expect("pinned");
            for file in [&pinned.program, &pinned.licence] {
                assert_eq!(file.sha256.len(), 64);
                assert!(file.url.contains(&pins.release));
            }
        }
    }

    #[test]
    fn the_source_notice_links_the_pinned_release() {
        let notices = include_str!("../../../THIRD_PARTY_NOTICES.md");
        let release = format!(
            "https://github.com/eugeneware/ffmpeg-static/releases/tag/{}",
            ffmpeg_pins().release
        );
        assert!(notices.contains(&release), "{release}");
    }

    #[test]
    fn finds_the_program_beside_the_app_and_nowhere_else() {
        let dir = tempfile::tempdir().unwrap();
        let app = dir.path().join("opsis-desktop");
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
