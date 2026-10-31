use std::path::{Path, PathBuf};

use opsis_types::engine::ENGINE_MISSING_MESSAGE;
use serde::{Deserialize, Serialize};

pub const CHROMIUM_CREDITS_FILE: &str = "LICENSES.chromium.html";
pub const ENGINE_LICENCES_FOLDER: &str = "engine-licences";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShellPlatform {
    #[serde(rename = "linux64")]
    Linux64,
    #[serde(rename = "linux-arm64")]
    LinuxArm64,
    #[serde(rename = "mac-x64")]
    MacX64,
    #[serde(rename = "mac-arm64")]
    MacArm64,
    #[serde(rename = "win64")]
    Win64,
}

#[derive(Debug, thiserror::Error)]
#[error("Unsupported platform {os}/{arch}")]
pub struct UnsupportedPlatform {
    pub os: String,
    pub arch: String,
}

impl ShellPlatform {
    pub fn for_os(os: &str, arch: &str) -> Result<Self, UnsupportedPlatform> {
        let arm = arch == "aarch64" || arch == "arm64";
        match os {
            "linux" => Ok(if arm { Self::LinuxArm64 } else { Self::Linux64 }),
            "macos" | "darwin" => Ok(if arm { Self::MacArm64 } else { Self::MacX64 }),
            "windows" | "win32" if !arm => Ok(Self::Win64),
            _ => Err(UnsupportedPlatform {
                os: os.to_owned(),
                arch: arch.to_owned(),
            }),
        }
    }

    pub fn current() -> Result<Self, UnsupportedPlatform> {
        Self::for_os(std::env::consts::OS, std::env::consts::ARCH)
    }

    pub fn id(self) -> &'static str {
        match self {
            Self::Linux64 => "linux64",
            Self::LinuxArm64 => "linux-arm64",
            Self::MacX64 => "mac-x64",
            Self::MacArm64 => "mac-arm64",
            Self::Win64 => "win64",
        }
    }

    pub fn windows(self) -> bool {
        self == Self::Win64
    }

    pub fn ffmpeg_target(self) -> &'static str {
        match self {
            Self::Linux64 => "linux-x64",
            Self::LinuxArm64 => "linux-arm64",
            Self::MacX64 => "darwin-x64",
            Self::MacArm64 => "darwin-arm64",
            Self::Win64 => "win32-x64",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnginePaths {
    pub dir: PathBuf,
    pub download_dir: PathBuf,
    pub shell_root: PathBuf,
    pub shell_folder: PathBuf,
    pub headless_shell: PathBuf,
    pub ffmpeg: PathBuf,
    pub ffmpeg_license: PathBuf,
}

pub fn engine_paths(dir: &Path, platform: ShellPlatform) -> EnginePaths {
    let shell_root = dir.join("chrome-headless-shell");
    let shell_folder = shell_root
        .join(platform.id())
        .join(format!("chrome-headless-shell-{}", platform.id()));
    let exe = if platform.windows() { ".exe" } else { "" };
    EnginePaths {
        dir: dir.to_path_buf(),
        download_dir: dir.join(".download"),
        headless_shell: shell_folder.join(format!("chrome-headless-shell{exe}")),
        shell_root,
        shell_folder,
        ffmpeg: dir.join("ffmpeg").join(format!("ffmpeg{exe}")),
        ffmpeg_license: dir.join("ffmpeg").join("LICENSE.txt"),
    }
}

pub fn find_headless_shell(paths: &EnginePaths) -> Option<&Path> {
    paths
        .headless_shell
        .exists()
        .then_some(paths.headless_shell.as_path())
}

#[derive(Debug, thiserror::Error)]
#[error("{}", ENGINE_MISSING_MESSAGE)]
pub struct EngineMissing;

pub fn is_downloaded(paths: &EnginePaths) -> bool {
    find_headless_shell(paths).is_some() && paths.ffmpeg.is_file()
}

pub fn downloaded_browser(dir: &Path) -> Option<PathBuf> {
    let paths = engine_paths(dir, ShellPlatform::current().ok()?);
    find_headless_shell(&paths).map(Path::to_path_buf)
}

pub fn download_dir(user_data: &Path) -> PathBuf {
    user_data.join("engine")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::downloads::{FfmpegFile, ffmpeg_download};

    #[test]
    fn every_platform_it_accepts_can_download_its_engine() {
        let mut accepted = Vec::new();
        for os in ["linux", "macos", "darwin", "windows", "win32", "freebsd"] {
            for arch in ["x86_64", "aarch64", "arm64"] {
                let Ok(platform) = ShellPlatform::for_os(os, arch) else {
                    continue;
                };
                let arm = arch != "x86_64";
                assert_eq!(platform.id().ends_with("arm64"), arm, "{os} {arch}");
                assert_eq!(platform.windows(), os.starts_with("win"), "{os} {arch}");
                assert!(
                    ffmpeg_download(platform, FfmpegFile::Ffmpeg).is_some(),
                    "{os} {arch}"
                );
                accepted.push(platform);
            }
        }
        accepted.sort_by_key(|p| p.id());
        accepted.dedup();
        assert_eq!(accepted.len(), 5);
        assert!(ShellPlatform::for_os("windows", "aarch64").is_err());
        assert!(ShellPlatform::current().is_ok());
    }

    #[test]
    fn keeps_the_shell_in_its_own_folder() {
        let p = engine_paths(Path::new("/engine"), ShellPlatform::Win64);
        assert_eq!(
            p.headless_shell.to_string_lossy().replace('\\', "/"),
            "/engine/chrome-headless-shell/win64/chrome-headless-shell-win64/chrome-headless-shell.exe"
        );
        assert_eq!(
            p.ffmpeg.to_string_lossy().replace('\\', "/"),
            "/engine/ffmpeg/ffmpeg.exe"
        );
    }
}
