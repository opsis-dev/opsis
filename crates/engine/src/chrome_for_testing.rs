use std::path::{Path, PathBuf};

use serde::Deserialize;
use tokio_util::sync::CancellationToken;

use crate::install::{InstallError, fetch, make_executable};
use crate::paths::ShellPlatform;

const KNOWN_GOOD: &str =
    "https://googlechromelabs.github.io/chrome-for-testing/known-good-versions-with-downloads.json";

#[derive(Deserialize)]
struct KnownGood {
    versions: Vec<KnownVersion>,
}

#[derive(Deserialize)]
struct KnownVersion {
    version: String,
    downloads: KnownDownloads,
}

#[derive(Deserialize)]
struct KnownDownloads {
    #[serde(default)]
    chrome: Vec<PlatformDownload>,
}

#[derive(Deserialize)]
struct PlatformDownload {
    platform: String,
    url: String,
}

fn chrome_binary(platform: ShellPlatform) -> &'static str {
    match platform {
        ShellPlatform::Linux64 => "chrome-linux64/chrome",
        ShellPlatform::MacX64 => {
            "chrome-mac-x64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing"
        }
        ShellPlatform::MacArm64 => {
            "chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing"
        }
        ShellPlatform::Win64 => "chrome-win64/chrome.exe",
        ShellPlatform::LinuxArm64 => "chrome-linux-arm64/chrome",
    }
}

async fn chrome_url(
    client: &reqwest::Client,
    version: &str,
    platform: ShellPlatform,
) -> Result<String, InstallError> {
    let network = |source| InstallError::Network {
        url: KNOWN_GOOD.to_owned(),
        source,
    };
    let text = client
        .get(KNOWN_GOOD)
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(network)?
        .text()
        .await
        .map_err(network)?;
    let known: KnownGood = serde_json::from_str(&text).map_err(std::io::Error::other)?;
    known
        .versions
        .into_iter()
        .find(|v| v.version == version)
        .and_then(|v| {
            v.downloads
                .chrome
                .into_iter()
                .find(|d| d.platform == platform.id())
        })
        .map(|d| d.url)
        .ok_or(InstallError::NoShell)
}

pub async fn full_chrome(
    version: &str,
    platform: ShellPlatform,
    cache: &Path,
) -> Result<PathBuf, InstallError> {
    let binary = chrome_binary(platform);
    let dir = cache.join(version).join(platform.id());
    let chrome = dir.join(binary);
    if chrome.exists() {
        return Ok(chrome);
    }
    let client = reqwest::Client::new();
    let url = chrome_url(&client, version, platform).await?;
    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
    }
    let fetched = fetch(
        &client,
        &url,
        &dir,
        &CancellationToken::new(),
        &mut |_, _| {},
    )
    .await?;
    let unpack_into = dir.clone();
    tokio::task::spawn_blocking(move || -> Result<(), InstallError> {
        zip::ZipArchive::new(std::fs::File::open(&fetched.path)?)?.extract(&unpack_into)?;
        std::fs::remove_file(&fetched.path)?;
        Ok(())
    })
    .await
    .map_err(std::io::Error::other)??;
    make_executable(&chrome)?;
    Ok(chrome)
}
