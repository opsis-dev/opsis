use std::fmt;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use futures_util::StreamExt;
use opsis_types::engine::EngineInstallUpdate;
use opsis_types::numbers;
use sha2::{Digest, Sha256};
use tokio_util::sync::CancellationToken;

use crate::downloads::{Pinned, shell_download};
use crate::paths::{ShellPlatform, engine_paths};

#[derive(Debug, thiserror::Error)]
pub enum InstallError {
    #[error("Couldn't download {url}: {source}")]
    Network { url: String, source: reqwest::Error },
    #[error("Download failed ({status}): {url}")]
    Status { url: String, status: u16 },
    #[error("The download from {url} isn't the file Opsis expects. Try again later.")]
    Mismatch { url: String },
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

#[derive(Debug, Default)]
pub struct CalmProgress {
    step: String,
    shown: Option<i64>,
    last_at: Option<Instant>,
}

impl CalmProgress {
    pub fn should_report(&mut self, update: &EngineInstallUpdate, now: Instant) -> bool {
        let current = update.step.clone();
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

impl fmt::Debug for Progress<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Progress")
            .field("calm", &self.calm)
            .finish_non_exhaustive()
    }
}

impl<'a> Progress<'a> {
    pub fn new(report: &'a mut (dyn FnMut(EngineInstallUpdate) + Send)) -> Self {
        Self {
            report,
            calm: CalmProgress::default(),
        }
    }

    fn send(&mut self, step: &str, progress: Option<f64>, message: String) {
        let update = EngineInstallUpdate {
            step: step.to_owned(),
            progress,
            message,
        };
        if self.calm.should_report(&update, Instant::now()) {
            (self.report)(update);
        }
    }
}

fn downloaded_text(got: u64, total: Option<u64>) -> String {
    match total {
        Some(total) => format!(
            "{} of {}",
            numbers::byte_size(got),
            numbers::byte_size(total)
        ),
        None => numbers::byte_size(got),
    }
}

#[derive(Debug)]
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
            () = cancel.cancelled() => return Err(InstallError::Cancelled),
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
    let exe = if platform.windows() { ".exe" } else { "" };
    let shell = target.join(format!("chrome-headless-shell{exe}"));
    if !shell.exists() {
        return Err(InstallError::NoShell);
    }
    Ok(make_executable(&shell)?)
}

pub async fn install_headless_shell(
    client: &reqwest::Client,
    dir: &Path,
    platform: ShellPlatform,
    progress: &mut Progress<'_>,
    cancel: &CancellationToken,
) -> Result<(), InstallError> {
    let p = engine_paths(dir, platform);
    let download = shell_download(platform).ok_or(InstallError::NoShell)?;
    progress.send(
        "download",
        Some(0.0),
        "Downloading the recording browser · starting".to_owned(),
    );
    let mut on_bytes = |got: u64, total: Option<u64>| {
        progress.send(
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
    progress.send("ready", Some(1.0), "Recording browser ready".to_owned());
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
    install_headless_shell(&client, dir, platform, &mut progress, cancel).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn update(step: &str, progress: f64) -> EngineInstallUpdate {
        EngineInstallUpdate {
            step: step.to_owned(),
            progress: Some(progress),
            message: String::new(),
        }
    }

    #[test]
    fn reports_only_when_the_shown_percentage_or_step_changes_or_every_half_second() {
        let mut calm = CalmProgress::default();
        let start = Instant::now();
        let at = |ms: u64| start + Duration::from_millis(ms);
        let seen: Vec<Option<f64>> = [
            (update("download", 0.101), 0),
            (update("download", 0.104), 0),
            (update("download", 0.109), 0),
            (update("download", 0.109), 600),
            (update("download", 0.12), 600),
            (update("unpack", 0.12), 600),
            (update("ready", 0.12), 600),
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
    fn says_how_much_of_the_download_has_arrived() {
        assert_eq!(
            downloaded_text(3 * 1024 * 1024 + 400_000, Some(29 * 1024 * 1024)),
            "3.4 MiB of 29 MiB"
        );
        assert_eq!(downloaded_text(1536, None), "1.5 KiB");
    }
}
