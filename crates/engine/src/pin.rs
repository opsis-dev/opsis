use std::path::Path;

use sha2::{Digest, Sha256};
use tokio_util::sync::CancellationToken;

use crate::downloads::{Downloads, FfmpegDownload, FfmpegFile, Pinned, ShellDownload};
use crate::install::{InstallError, fetch};
use crate::paths::ShellPlatform;
use crate::tools::file_sha256;

pub const FFMPEG_RELEASE: &str = "b6.1.1";

const FFMPEG_TARGETS: [&str; 5] = [
    "linux-x64",
    "linux-arm64",
    "darwin-x64",
    "darwin-arm64",
    "win32-x64",
];

const SHELLS: [ShellPlatform; 5] = [
    ShellPlatform::Linux64,
    ShellPlatform::LinuxArm64,
    ShellPlatform::MacX64,
    ShellPlatform::MacArm64,
    ShellPlatform::Win64,
];

const FFMPEG_FILES: [FfmpegFile; 2] = [FfmpegFile::Ffmpeg, FfmpegFile::License];

pub fn shell_url(version: &str, platform: ShellPlatform) -> String {
    format!(
        "https://storage.googleapis.com/chrome-for-testing-public/{version}/{id}/chrome-headless-shell-{id}.zip",
        id = platform.id()
    )
}

pub fn ffmpeg_url(target: &str, file: FfmpegFile) -> String {
    let base =
        format!("https://github.com/eugeneware/ffmpeg-static/releases/download/{FFMPEG_RELEASE}");
    match file {
        FfmpegFile::Ffmpeg => format!("{base}/ffmpeg-{target}.gz"),
        FfmpegFile::License => format!("{base}/{target}.LICENSE"),
    }
}

fn cached_name(url: &str) -> String {
    let key = &hex::encode(Sha256::digest(url.as_bytes()))[..16];
    let base = url
        .split('?')
        .next()
        .and_then(|u| u.rsplit('/').next())
        .unwrap_or("download");
    format!("{key}-{base}")
}

async fn pinned(
    client: &reqwest::Client,
    url: String,
    cache: &Path,
) -> Result<Pinned, InstallError> {
    let file = cache.join(cached_name(&url));
    if !file.exists() {
        eprintln!("downloading {url}");
        let fetched = fetch(
            client,
            &url,
            cache,
            &CancellationToken::new(),
            &mut |_, _| {},
        )
        .await?;
        std::fs::rename(&fetched.path, &file)?;
    }
    Ok(Pinned {
        sha256: file_sha256(&file)?,
        size: std::fs::metadata(&file)?.len(),
        url,
    })
}

pub async fn pin_downloads(tested_version: &str, cache: &Path) -> Result<Downloads, InstallError> {
    let client = reqwest::Client::new();
    let mut shells = Vec::new();
    for platform in SHELLS {
        let url = shell_url(tested_version, platform);
        shells.push(ShellDownload {
            platform,
            pinned: pinned(&client, url, cache).await?,
        });
    }
    let mut ffmpeg = Vec::new();
    for target in FFMPEG_TARGETS {
        for file in FFMPEG_FILES {
            ffmpeg.push(FfmpegDownload {
                target: target.to_owned(),
                file,
                pinned: pinned(&client, ffmpeg_url(target, file), cache).await?,
            });
        }
    }
    Ok(Downloads {
        tested_version: tested_version.to_owned(),
        ffmpeg_release: FFMPEG_RELEASE.to_owned(),
        shells,
        ffmpeg,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::downloads::downloads;

    #[test]
    fn builds_the_addresses_that_are_pinned_today() {
        let d = downloads();
        for shell in &d.shells {
            assert_eq!(
                shell_url(&d.tested_version, shell.platform),
                shell.pinned.url
            );
        }
        for file in &d.ffmpeg {
            assert_eq!(ffmpeg_url(&file.target, file.file), file.pinned.url);
        }
        assert_eq!(d.ffmpeg_release, FFMPEG_RELEASE);
    }

    #[test]
    fn writes_the_downloads_file_in_the_form_it_is_committed() {
        let json = serde_json::to_string_pretty(downloads()).unwrap();
        assert_eq!(
            format!("{json}\n"),
            include_str!("downloads.generated.json")
        );
    }

    #[test]
    fn names_cached_files_like_the_address_they_came_from() {
        assert!(cached_name("https://a.b/x/chrome.zip?clear").ends_with("-chrome.zip"));
    }
}
