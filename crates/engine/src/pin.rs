use std::path::Path;

use sha2::{Digest, Sha256};
use tokio_util::sync::CancellationToken;

use crate::downloads::{Downloads, Pinned, ShellDownload};
use crate::ffmpeg::{FFMPEG_RELEASE, FFMPEG_TARGETS, FfmpegPins, FfmpegTarget};
use crate::install::{InstallError, fetch};
use crate::paths::ShellPlatform;
use crate::tools::file_sha256;

const SHELLS: [ShellPlatform; 5] = [
    ShellPlatform::Linux64,
    ShellPlatform::LinuxArm64,
    ShellPlatform::MacX64,
    ShellPlatform::MacArm64,
    ShellPlatform::Win64,
];

pub fn shell_url(version: &str, platform: ShellPlatform) -> String {
    format!(
        "https://storage.googleapis.com/chrome-for-testing-public/{version}/{id}/chrome-headless-shell-{id}.zip",
        id = platform.id()
    )
}

fn ffmpeg_release_url(file: &str) -> String {
    format!("https://github.com/eugeneware/ffmpeg-static/releases/download/{FFMPEG_RELEASE}/{file}")
}

pub fn ffmpeg_program_url(asset: &str) -> String {
    ffmpeg_release_url(&format!("ffmpeg-{asset}.gz"))
}

pub fn ffmpeg_licence_url(asset: &str) -> String {
    ffmpeg_release_url(&format!("{asset}.LICENSE"))
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
        std::fs::create_dir_all(cache)?;
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

pub async fn verified_download(
    client: &reqwest::Client,
    expected: &Pinned,
    cache: &Path,
) -> Result<std::path::PathBuf, InstallError> {
    let got = pinned(client, expected.url.clone(), cache).await?;
    let file = cache.join(cached_name(&expected.url));
    if got != *expected {
        std::fs::remove_file(&file)?;
        return Err(InstallError::Mismatch {
            url: expected.url.clone(),
        });
    }
    Ok(file)
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
    Ok(Downloads {
        tested_version: tested_version.to_owned(),
        shells,
    })
}

pub async fn pin_ffmpeg(cache: &Path) -> Result<FfmpegPins, InstallError> {
    let client = reqwest::Client::new();
    let mut targets = Vec::new();
    for (target, asset) in FFMPEG_TARGETS {
        targets.push(FfmpegTarget {
            target: target.to_owned(),
            asset: asset.to_owned(),
            program: pinned(&client, ffmpeg_program_url(asset), cache).await?,
            licence: pinned(&client, ffmpeg_licence_url(asset), cache).await?,
        });
    }
    Ok(FfmpegPins {
        release: FFMPEG_RELEASE.to_owned(),
        targets,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::downloads::downloads;
    use crate::ffmpeg::ffmpeg_pins;

    #[test]
    fn builds_the_addresses_that_are_pinned_today() {
        let d = downloads();
        for shell in &d.shells {
            assert_eq!(
                shell_url(&d.tested_version, shell.platform),
                shell.pinned.url
            );
        }
    }

    #[test]
    fn builds_the_ffmpeg_addresses_that_are_pinned_today() {
        let pins = ffmpeg_pins();
        assert_eq!(pins.release, FFMPEG_RELEASE);
        for (target, asset) in FFMPEG_TARGETS {
            let pinned = pins.target(target).expect("every target is pinned");
            assert_eq!(pinned.asset, asset);
            assert_eq!(pinned.program.url, ffmpeg_program_url(asset));
            assert_eq!(pinned.licence.url, ffmpeg_licence_url(asset));
        }
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
    fn writes_the_ffmpeg_pins_in_the_form_they_are_committed() {
        let json = serde_json::to_string_pretty(ffmpeg_pins()).unwrap();
        assert_eq!(format!("{json}\n"), include_str!("ffmpeg.generated.json"));
    }

    #[test]
    fn names_cached_files_like_the_address_they_came_from() {
        assert!(cached_name("https://a.b/x/chrome.zip?clear").ends_with("-chrome.zip"));
    }
}
