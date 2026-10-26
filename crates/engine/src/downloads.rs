use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

use crate::paths::ShellPlatform;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ShellSource {
    Remotion,
    Google,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FfmpegFile {
    Ffmpeg,
    Ffprobe,
    License,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Pinned {
    pub url: String,
    pub sha256: String,
    pub size: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ShellDownload {
    pub platform: ShellPlatform,
    pub source: ShellSource,
    #[serde(flatten)]
    pub pinned: Pinned,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FfmpegDownload {
    pub target: String,
    pub file: FfmpegFile,
    #[serde(flatten)]
    pub pinned: Pinned,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Downloads {
    pub tested_version: String,
    pub ffmpeg_release: String,
    pub shells: Vec<ShellDownload>,
    pub ffmpeg: Vec<FfmpegDownload>,
}

static DOWNLOADS: LazyLock<Downloads> = LazyLock::new(|| {
    serde_json::from_str(include_str!("downloads.generated.json")).expect("pinned downloads parse")
});

pub fn downloads() -> &'static Downloads {
    &DOWNLOADS
}

pub const REMOTION_MIN_GLIBC: (u32, u32) = (2, 35);

#[cfg(all(target_os = "linux", target_env = "gnu"))]
pub fn glibc_version() -> Option<(u32, u32)> {
    let text = unsafe { std::ffi::CStr::from_ptr(libc::gnu_get_libc_version()) }
        .to_str()
        .ok()?;
    let (major, minor) = text.split_once('.')?;
    Some((major.parse().ok()?, minor.split('.').next()?.parse().ok()?))
}

#[cfg(not(all(target_os = "linux", target_env = "gnu")))]
pub fn glibc_version() -> Option<(u32, u32)> {
    None
}

pub fn shell_source(platform: ShellPlatform, glibc: Option<(u32, u32)>) -> ShellSource {
    match platform {
        ShellPlatform::MacArm64 => ShellSource::Remotion,
        ShellPlatform::Linux64 | ShellPlatform::LinuxArm64
            if glibc.is_some_and(|v| v >= REMOTION_MIN_GLIBC) =>
        {
            ShellSource::Remotion
        }
        _ => ShellSource::Google,
    }
}

pub fn shell_download(
    platform: ShellPlatform,
    source: ShellSource,
) -> Option<&'static ShellDownload> {
    downloads()
        .shells
        .iter()
        .find(|s| s.platform == platform && s.source == source)
}

pub fn ffmpeg_download(
    platform: ShellPlatform,
    file: FfmpegFile,
) -> Option<&'static FfmpegDownload> {
    downloads()
        .ffmpeg
        .iter()
        .find(|d| d.target == platform.ffmpeg_target() && d.file == file)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_remotions_build_where_remotion_uses_it() {
        assert_eq!(
            shell_source(ShellPlatform::MacArm64, None),
            ShellSource::Remotion
        );
        assert_eq!(
            shell_source(ShellPlatform::Linux64, Some((2, 39))),
            ShellSource::Remotion
        );
        assert_eq!(
            shell_source(ShellPlatform::Linux64, Some((2, 31))),
            ShellSource::Google
        );
        assert_eq!(
            shell_source(ShellPlatform::Linux64, None),
            ShellSource::Google
        );
        assert_eq!(
            shell_source(ShellPlatform::Win64, None),
            ShellSource::Google
        );
    }

    #[test]
    fn pins_every_download_the_installer_can_ask_for() {
        let d = downloads();
        assert!(!d.tested_version.is_empty());
        for platform in [
            ShellPlatform::Linux64,
            ShellPlatform::LinuxArm64,
            ShellPlatform::MacX64,
            ShellPlatform::MacArm64,
            ShellPlatform::Win64,
        ] {
            for glibc in [None, Some((2, 39))] {
                assert!(
                    shell_download(platform, shell_source(platform, glibc)).is_some()
                        || platform == ShellPlatform::LinuxArm64 && glibc.is_none()
                );
            }
            for file in [FfmpegFile::Ffmpeg, FfmpegFile::Ffprobe, FfmpegFile::License] {
                assert!(
                    ffmpeg_download(platform, file).is_some(),
                    "{platform:?} {file:?}"
                );
            }
        }
        for shell in &d.shells {
            assert_eq!(shell.pinned.sha256.len(), 64);
            assert!(shell.pinned.url.contains(&d.tested_version));
        }
    }
}
