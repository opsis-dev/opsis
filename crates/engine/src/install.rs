use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use futures_util::StreamExt;
use humansize::{FormatSizeOptions, WINDOWS, format_size};
use opsis_types::engine::{EngineInstallUpdate, EnginePart};
use opsis_types::numbers;
use sha2::{Digest, Sha256};
use tokio_util::sync::CancellationToken;

use crate::downloads::{
    FfmpegFile, Pinned, ffmpeg_download, glibc_version, shell_download, shell_source,
};
use crate::paths::{ShellPlatform, engine_paths};

#[derive(Debug, thiserror::Error)]
pub enum InstallError {
    #[error("Couldn't download {url}: {source}")]
    Network { url: String, source: reqwest::Error },
    #[error("Download failed ({status}): {url}")]
    Status { url: String, status: u16 },
    #[error("The download from {url} isn't the file Opsis expects. Try again later.")]
    Mismatch { url: String },
    #[error("The recording browser needs a newer system (glibc {}.{} or later).", .0.0, .0.1)]
    TooOld((u32, u32)),
    #[error("There's no recording browser for this computer.")]
    NoShell,
    #[error("Setup was stopped")]
    Cancelled,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Couldn't unpack the recording browser: {0}")]
    Zip(#[from] zip::result::ZipError),
}

const CALM: Duration = Duration::from_millis(500);

#[derive(Default)]
pub struct CalmProgress {
    step: String,
    shown: Option<i64>,
    last_at: Option<Instant>,
}

impl CalmProgress {
    pub fn should_report(&mut self, update: &EngineInstallUpdate, now: Instant) -> bool {
        let current = format!("{:?}|{}", update.part, update.step);
        let percent = update
            .progress
            .map(|p| numbers::truncate::<i64>((p * 100.0).floor()));
        let recent = self.last_at.is_some_and(|t| now.duration_since(t) < CALM);
        if current == self.step && percent == self.shown && recent {
            return false;
        }
        self.step = current;
        self.shown = percent;
        self.last_at = Some(now);
        true
    }
}

pub struct Progress<'a> {
    report: &'a mut (dyn FnMut(EngineInstallUpdate) + Send),
    calm: CalmProgress,
}

impl<'a> Progress<'a> {
    pub fn new(report: &'a mut (dyn FnMut(EngineInstallUpdate) + Send)) -> Self {
        Self {
            report,
            calm: CalmProgress::default(),
        }
    }

    fn send(&mut self, part: EnginePart, step: &str, progress: Option<f64>, message: String) {
        let update = EngineInstallUpdate {
            part,
            step: step.to_owned(),
            progress,
            message,
        };
        if self.calm.should_report(&update, Instant::now()) {
            (self.report)(update);
        }
    }
}

fn size_text(bytes: u64) -> String {
    format_size(
        bytes,
        FormatSizeOptions::from(WINDOWS)
            .decimal_places(1)
            .decimal_zeroes(1),
    )
}

fn downloaded_text(got: u64, total: Option<u64>) -> String {
    match total {
        Some(total) => format!("{} of {}", size_text(got), size_text(total)),
        None => size_text(got),
    }
}

pub struct Fetched {
    pub path: PathBuf,
    pub sha256: String,
    pub size: u64,
}

pub async fn fetch(
    client: &reqwest::Client,
    url: &str,
    into: &Path,
    cancel: &CancellationToken,
    on_bytes: &mut (dyn FnMut(u64, Option<u64>) + Send),
) -> Result<Fetched, InstallError> {
    let network = |source| InstallError::Network {
        url: url.to_owned(),
        source,
    };
    let response = client.get(url).send().await.map_err(network)?;
    if !response.status().is_success() {
        return Err(InstallError::Status {
            url: url.to_owned(),
            status: response.status().as_u16(),
        });
    }
    let total = response.content_length();
    std::fs::create_dir_all(into)?;
    let mut file = tempfile::Builder::new()
        .prefix(".download-")
        .tempfile_in(into)?;
    let mut hash = Sha256::new();
    let mut got = 0u64;
    let mut body = response.bytes_stream();
    loop {
        let chunk = tokio::select! {
            _ = cancel.cancelled() => return Err(InstallError::Cancelled),
            chunk = body.next() => chunk,
        };
        let Some(chunk) = chunk else { break };
        let chunk = chunk.map_err(network)?;
        hash.update(&chunk);
        file.write_all(&chunk)?;
        got += chunk.len() as u64;
        on_bytes(got, total);
    }
    let (_, path) = file.keep().map_err(|e| e.error)?;
    Ok(Fetched {
        path,
        sha256: hex::encode(hash.finalize()),
        size: got,
    })
}

async fn fetch_verified(
    client: &reqwest::Client,
    pinned: &Pinned,
    into: &Path,
    cancel: &CancellationToken,
    on_bytes: &mut (dyn FnMut(u64, Option<u64>) + Send),
) -> Result<PathBuf, InstallError> {
    let fetched = fetch(client, &pinned.url, into, cancel, on_bytes).await?;
    if fetched.size != pinned.size || fetched.sha256 != pinned.sha256 {
        std::fs::remove_file(&fetched.path)?;
        return Err(InstallError::Mismatch {
            url: pinned.url.clone(),
        });
    }
    Ok(fetched.path)
}

#[cfg(unix)]
pub fn make_executable(file: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(file, std::fs::Permissions::from_mode(0o755))
}

#[cfg(not(unix))]
pub fn make_executable(_file: &Path) -> std::io::Result<()> {
    Ok(())
}

fn gunzip_into(archive: &Path, dest: &Path) -> std::io::Result<()> {
    let mut decoder = flate2::read::GzDecoder::new(std::fs::File::open(archive)?);
    let dir = dest.parent().unwrap_or(Path::new("."));
    let mut out = tempfile::Builder::new()
        .prefix(".partial-")
        .tempfile_in(dir)?;
    std::io::copy(&mut decoder, &mut out)?;
    out.persist(dest).map_err(|e| e.error)?;
    std::fs::remove_file(archive)
}

pub async fn install_ffmpeg(
    client: &reqwest::Client,
    dir: &Path,
    platform: ShellPlatform,
    progress: &mut Progress<'_>,
    cancel: &CancellationToken,
) -> Result<(), InstallError> {
    let p = engine_paths(dir, platform);
    let files = [
        (FfmpegFile::Ffmpeg, &p.ffmpeg),
        (FfmpegFile::Ffprobe, &p.ffprobe),
        (FfmpegFile::License, &p.ffmpeg_license),
    ];
    let folder = p.ffmpeg.parent().unwrap_or(dir).to_path_buf();
    for (i, (file, dest)) in files.iter().enumerate() {
        let pinned = &ffmpeg_download(platform, *file)
            .ok_or(InstallError::NoShell)?
            .pinned;
        let step = format!("download-{}", i + 1);
        let mut on_bytes = |got: u64, total: Option<u64>| {
            progress.send(
                EnginePart::Ffmpeg,
                &step,
                total.map(|t| numbers::float_wide(got) / numbers::float_wide(t)),
                format!(
                    "Downloading the video tools ({} of {}) · {}",
                    i + 1,
                    files.len(),
                    downloaded_text(got, total)
                ),
            );
        };
        let fetched = fetch_verified(client, pinned, &folder, cancel, &mut on_bytes).await?;
        if *file == FfmpegFile::License {
            std::fs::rename(&fetched, dest)?;
        } else {
            let target = (*dest).clone();
            tokio::task::spawn_blocking(move || gunzip_into(&fetched, &target))
                .await
                .map_err(std::io::Error::other)??;
            make_executable(dest)?;
        }
    }
    progress.send(
        EnginePart::Ffmpeg,
        "ready",
        Some(1.0),
        "Video tools ready".to_owned(),
    );
    Ok(())
}

fn unpack_shell(
    archive: &Path,
    staging: &Path,
    platform: ShellPlatform,
) -> Result<(), InstallError> {
    if staging.exists() {
        std::fs::remove_dir_all(staging)?;
    }
    zip::ZipArchive::new(std::fs::File::open(archive)?)?.extract(staging)?;
    std::fs::remove_file(archive)?;
    let target = staging.join(format!("chrome-headless-shell-{}", platform.id()));
    if !target.exists() {
        let Some(only) = std::fs::read_dir(staging)?
            .filter_map(Result::ok)
            .find(|e| e.path().is_dir())
        else {
            return Err(InstallError::NoShell);
        };
        std::fs::rename(only.path(), &target)?;
    }
    let chrome = target.join("chrome");
    if chrome.exists() {
        std::fs::rename(&chrome, target.join("chrome-headless-shell"))?;
    }
    for name in [
        "headless_shell",
        "chrome-headless-shell",
        "chrome-headless-shell.exe",
    ] {
        let file = target.join(name);
        if file.exists() {
            make_executable(&file)?;
        }
    }
    Ok(())
}

pub async fn install_headless_shell(
    client: &reqwest::Client,
    dir: &Path,
    platform: ShellPlatform,
    progress: &mut Progress<'_>,
    cancel: &CancellationToken,
) -> Result<(), InstallError> {
    let p = engine_paths(dir, platform);
    let glibc = glibc_version();
    let source = shell_source(platform, glibc);
    let download = shell_download(platform, source).ok_or(match platform {
        ShellPlatform::LinuxArm64 => InstallError::TooOld(crate::downloads::REMOTION_MIN_GLIBC),
        _ => InstallError::NoShell,
    })?;
    progress.send(
        EnginePart::HeadlessShell,
        "download",
        Some(0.0),
        "Downloading the recording browser · starting".to_owned(),
    );
    let mut on_bytes = |got: u64, total: Option<u64>| {
        progress.send(
            EnginePart::HeadlessShell,
            "download",
            total.map(|t| numbers::float_wide(got) / numbers::float_wide(t)),
            format!(
                "Downloading the recording browser · {}",
                downloaded_text(got, total)
            ),
        );
    };
    let archive = fetch_verified(
        client,
        &download.pinned,
        &p.download_dir,
        cancel,
        &mut on_bytes,
    )
    .await?;
    let staging = p.download_dir.join(platform.id());
    let unpacked = staging.clone();
    tokio::task::spawn_blocking(move || unpack_shell(&archive, &unpacked, platform))
        .await
        .map_err(std::io::Error::other)??;
    if p.shell_root.exists() {
        std::fs::remove_dir_all(&p.shell_root)?;
    }
    std::fs::create_dir_all(&p.shell_root)?;
    std::fs::rename(&staging, p.shell_root.join(platform.id()))?;
    std::fs::write(
        p.shell_root.join("VERSION"),
        &crate::downloads::downloads().tested_version,
    )?;
    std::fs::remove_dir_all(&p.download_dir)?;
    progress.send(
        EnginePart::HeadlessShell,
        "ready",
        Some(1.0),
        "Recording browser ready".to_owned(),
    );
    Ok(())
}

pub async fn install_engine(
    dir: &Path,
    platform: ShellPlatform,
    report: &mut (dyn FnMut(EngineInstallUpdate) + Send),
    cancel: &CancellationToken,
) -> Result<(), InstallError> {
    let client = reqwest::Client::new();
    let mut progress = Progress::new(report);
    install_ffmpeg(&client, dir, platform, &mut progress, cancel).await?;
    install_headless_shell(&client, dir, platform, &mut progress, cancel).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn update(part: EnginePart, step: &str, progress: f64) -> EngineInstallUpdate {
        EngineInstallUpdate {
            part,
            step: step.to_owned(),
            progress: Some(progress),
            message: String::new(),
        }
    }

    #[test]
    fn reports_only_when_the_shown_percentage_or_part_changes_or_every_half_second() {
        let mut calm = CalmProgress::default();
        let start = Instant::now();
        let at = |ms: u64| start + Duration::from_millis(ms);
        let seen: Vec<Option<f64>> = [
            (update(EnginePart::Ffmpeg, "download-1", 0.101), 0),
            (update(EnginePart::Ffmpeg, "download-1", 0.104), 0),
            (update(EnginePart::Ffmpeg, "download-1", 0.109), 0),
            (update(EnginePart::Ffmpeg, "download-1", 0.109), 600),
            (update(EnginePart::Ffmpeg, "download-1", 0.12), 600),
            (update(EnginePart::Ffmpeg, "download-2", 0.12), 600),
            (update(EnginePart::HeadlessShell, "download", 0.12), 600),
        ]
        .into_iter()
        .filter(|(u, ms)| calm.should_report(u, at(*ms)))
        .map(|(u, _)| u.progress)
        .collect();
        assert_eq!(
            seen,
            [Some(0.101), Some(0.109), Some(0.12), Some(0.12), Some(0.12)]
        );
    }

    #[test]
    fn writes_sizes_in_binary_units() {
        assert_eq!(
            downloaded_text(3 * 1024 * 1024, Some(29 * 1024 * 1024)),
            "3.0 MB of 29.0 MB"
        );
        assert_eq!(downloaded_text(1536, None), "1.5 kB");
    }
}
