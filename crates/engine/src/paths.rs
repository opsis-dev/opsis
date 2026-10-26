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
    pub headless_shells: [PathBuf; 2],
    pub ffmpeg: PathBuf,
    pub ffprobe: PathBuf,
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
        headless_shells: [
            shell_folder.join("headless_shell"),
            shell_folder.join(format!("chrome-headless-shell{exe}")),
        ],
        shell_root,
        shell_folder,
        ffmpeg: dir.join("ffmpeg").join(format!("ffmpeg{exe}")),
        ffprobe: dir.join("ffmpeg").join(format!("ffprobe{exe}")),
        ffmpeg_license: dir.join("ffmpeg").join("LICENSE.txt"),
    }
}

pub fn find_headless_shell(paths: &EnginePaths) -> Option<&Path> {
    paths
        .headless_shells
        .iter()
        .find(|f| f.exists())
        .map(PathBuf::as_path)
}

#[derive(Debug, thiserror::Error)]
#[error("{}", ENGINE_MISSING_MESSAGE)]
pub struct EngineMissing;

pub fn is_downloaded(paths: &EnginePaths) -> bool {
    find_headless_shell(paths).is_some() && paths.ffmpeg.is_file() && paths.ffprobe.is_file()
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

    #[test]
    fn maps_platforms_to_the_download_platforms() {
        assert_eq!(
            ShellPlatform::for_os("linux", "x86_64").unwrap(),
            ShellPlatform::Linux64
        );
        assert_eq!(
            ShellPlatform::for_os("linux", "aarch64").unwrap(),
            ShellPlatform::LinuxArm64
        );
        assert_eq!(
            ShellPlatform::for_os("macos", "aarch64").unwrap(),
            ShellPlatform::MacArm64
        );
        assert_eq!(
            ShellPlatform::for_os("windows", "x86_64").unwrap(),
            ShellPlatform::Win64
        );
        assert!(ShellPlatform::for_os("windows", "aarch64").is_err());
    }

    #[test]
    fn keeps_the_shell_in_its_own_folder_under_either_builds_name() {
        let p = engine_paths(Path::new("/engine"), ShellPlatform::Win64);
        let shown: Vec<String> = p
            .headless_shells
            .iter()
            .map(|f| f.to_string_lossy().replace('\\', "/"))
            .collect();
        assert_eq!(
            shown,
            [
                "/engine/chrome-headless-shell/win64/chrome-headless-shell-win64/headless_shell",
                "/engine/chrome-headless-shell/win64/chrome-headless-shell-win64/chrome-headless-shell.exe",
            ]
        );
        assert_eq!(
            p.ffprobe.to_string_lossy().replace('\\', "/"),
            "/engine/ffmpeg/ffprobe.exe"
        );
    }
}
