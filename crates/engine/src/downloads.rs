use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

use crate::paths::ShellPlatform;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Pinned {
    pub url: String,
    pub sha256: String,
    pub size: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ShellDownload {
    pub platform: ShellPlatform,
    #[serde(flatten)]
    pub pinned: Pinned,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Downloads {
    pub tested_version: String,
    pub shells: Vec<ShellDownload>,
}

static DOWNLOADS: LazyLock<Downloads> = LazyLock::new(|| {
    serde_json::from_str(include_str!("downloads.generated.json")).expect("pinned downloads parse")
});

pub fn downloads() -> &'static Downloads {
    &DOWNLOADS
}

pub fn shell_download(platform: ShellPlatform) -> Option<&'static ShellDownload> {
    downloads().shells.iter().find(|s| s.platform == platform)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pins_every_download_the_installer_can_ask_for() {
        let d = downloads();
        assert_ne!(d.tested_version, "");
        for platform in [
            ShellPlatform::Linux64,
            ShellPlatform::LinuxArm64,
            ShellPlatform::MacX64,
            ShellPlatform::MacArm64,
            ShellPlatform::Win64,
        ] {
            assert!(shell_download(platform).is_some(), "{platform:?}");
        }
        for shell in &d.shells {
            assert_eq!(shell.pinned.sha256.len(), 64);
            assert!(shell.pinned.url.contains(&d.tested_version));
        }
    }

    #[test]
    fn reads_the_downloads_file_in_the_form_it_is_committed() {
        let json = serde_json::to_string_pretty(downloads()).unwrap();
        assert_eq!(
            format!("{json}\n"),
            include_str!("downloads.generated.json")
        );
    }
}
