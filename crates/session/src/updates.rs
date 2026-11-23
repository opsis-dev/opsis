use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

use opsis_types::numbers;
use opsis_update::failed::{FailedUpdate, failed_file, record, take};
use opsis_update::fetch::{Available, Downloaded, client, public_key};
use opsis_update::installer::{Installer, installed_from};
use tokio_util::sync::CancellationToken;

use crate::Session;
use crate::events::{Emit, UpdateChanged};
use crate::state::AppState;

const DOWNLOAD_TASK: &str = "An update is downloading.";
const UPDATES_FOLDER: &str = "updates";

#[derive(Debug, Clone, Default)]
pub struct UpdateStatus {
    pub checking: bool,
    pub checked: bool,
    pub available: Option<Available>,
    pub downloading: bool,
    pub progress: Option<f64>,
    pub downloaded: Option<Downloaded>,
    pub error: String,
}

#[derive(Debug, Default)]
pub struct Updates {
    status: Mutex<UpdateStatus>,
    restart: Mutex<bool>,
    cancel: Mutex<Option<CancellationToken>>,
}

impl Updates {
    fn status(&self) -> MutexGuard<'_, UpdateStatus> {
        self.status.lock().expect("update status")
    }

    pub fn snapshot(&self) -> UpdateStatus {
        self.status().clone()
    }

    fn take_downloaded(&self) -> Option<Downloaded> {
        self.status().downloaded.take()
    }
}

pub fn installer() -> Option<Installer> {
    installed_from()
}

fn folder(app: &Session) -> PathBuf {
    app.state::<AppState>().paths.user_data.join(UPDATES_FOLDER)
}

fn changed(app: &Session) {
    if let Err(error) = UpdateChanged.emit(app) {
        log::warn!("sending the update status: {error}");
    }
}

fn set(app: &Session, change: impl FnOnce(&mut UpdateStatus)) {
    change(&mut app.state::<Updates>().status());
    changed(app);
}

pub fn begin(app: &Session) {
    match installer() {
        Some(installer) => log::info!("installed from {}", installer.label()),
        None => log::info!("not installed from a release package"),
    }
    if let Some(failed) = take(&failed_file(&app.state::<AppState>().paths.user_data)) {
        let message = failed.message();
        log::warn!("{message}");
        app.platform().notify("The update didn't install", &message);
        set(app, |status| status.error = message);
    }
    let leftovers = folder(app);
    crate::runtime::spawn(async move {
        if tokio::fs::try_exists(&leftovers).await.unwrap_or(false)
            && let Err(error) = tokio::fs::remove_dir_all(&leftovers).await
        {
            log::warn!("removing the update files left from before: {error}");
        }
    });
    let wanted = app.state::<AppState>().settings().updates.check_on_start;
    if wanted && installer().is_some() {
        let app = app.clone();
        crate::runtime::spawn(async move { check(app).await });
    }
}

pub async fn check(app: Session) {
    let Some(installer) = installer() else {
        return;
    };
    {
        let mut status = app.state::<Updates>().status();
        if status.checking || status.downloading {
            return;
        }
        status.checking = true;
        status.error.clear();
    }
    changed(&app);
    let found = match client() {
        Ok(client) => opsis_update::fetch::check(&client, installer, crate::APP_VERSION).await,
        Err(error) => Err(error),
    };
    set(&app, |status| {
        status.checking = false;
        status.checked = true;
        match found {
            Ok(available) => status.available = available,
            Err(error) => {
                log::warn!("checking for updates: {error:?}");
                status.error = error.to_string();
            }
        }
    });
}

fn shown_progress(got: u64, total: Option<u64>) -> Option<f64> {
    total
        .filter(|total| *total > 0)
        .map(|total| (numbers::float_wide(got) / numbers::float_wide(total)).min(1.0))
}

fn same_percent(a: Option<f64>, b: Option<f64>) -> bool {
    let percent = |p: Option<f64>| p.map(|p| numbers::truncate::<i64>((p * 100.0).floor()));
    percent(a) == percent(b)
}

pub async fn download(app: Session) {
    let Some(installer) = installer().filter(|i| i.updates_itself()) else {
        return;
    };
    let offered = {
        let mut status = app.state::<Updates>().status();
        let offered = status.available.clone().filter(|_| !status.downloading);
        let Some(offered) = offered else {
            return;
        };
        let Some(file) = offered.file.clone() else {
            return;
        };
        status.downloading = true;
        status.progress = Some(0.0);
        status.error.clear();
        (offered.version, file)
    };
    changed(&app);
    let (version, file) = offered;
    let cancel = CancellationToken::new();
    *app.state::<Updates>().cancel.lock().expect("update cancel") = Some(cancel.clone());
    let state = app.state::<AppState>();
    state.tasks.begin(DOWNLOAD_TASK);
    let into = folder(&app).join(&version);
    let reporter = app.clone();
    let mut on_bytes = move |got: u64, total: Option<u64>| {
        let progress = shown_progress(got, total);
        let updates = reporter.state::<Updates>();
        let moved = !same_percent(updates.status().progress, progress);
        if moved {
            set(&reporter, |status| status.progress = progress);
        }
    };
    let result = match (client(), public_key()) {
        (Ok(client), Ok(key)) => {
            opsis_update::fetch::download(&client, &file, &key, &into, &cancel, &mut on_bytes).await
        }
        (Err(error), _) | (_, Err(error)) => Err(error),
    };
    state.tasks.end(DOWNLOAD_TASK);
    app.state::<Updates>()
        .cancel
        .lock()
        .expect("update cancel")
        .take();
    set(&app, |status| {
        status.downloading = false;
        status.progress = None;
        match result {
            Ok(path) => {
                status.downloaded = Some(Downloaded {
                    installer,
                    version,
                    path,
                });
            }
            Err(error) => {
                log::warn!("downloading the update: {error:?}");
                status.error = error.to_string();
            }
        }
    });
}

pub fn cancel_download(app: &Session) {
    if let Some(cancel) = app
        .state::<Updates>()
        .cancel
        .lock()
        .expect("update cancel")
        .as_ref()
    {
        cancel.cancel();
    }
}

pub fn want_restart(app: &Session, restart: bool) {
    *app.state::<Updates>()
        .restart
        .lock()
        .expect("update restart") = restart;
}

pub fn finish(app: &Session) {
    let Some(update) = app.state::<Updates>().take_downloaded() else {
        return;
    };
    let reopen = *app
        .state::<Updates>()
        .restart
        .lock()
        .expect("update restart");
    log::info!(
        "installing version {} from {}",
        update.version,
        update.path.display()
    );
    let failed = failed_file(&app.state::<AppState>().paths.user_data);
    if let Err(error) = opsis_update::install::install(&update, reopen, &failed) {
        log::error!("installing the update: {error:?}");
        let noted = record(
            &failed,
            &FailedUpdate {
                version: update.version.clone(),
                reason: error.to_string(),
            },
        );
        if let Err(error) = noted {
            log::warn!("noting the failed update: {error}");
        }
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        #[test]
        fn progress_stays_between_nothing_and_all(got in 0u64..10_000_000, total in 1u64..10_000_000) {
            let shown = shown_progress(got, Some(total)).unwrap();
            prop_assert!((0.0..=1.0).contains(&shown));
        }
    }

    #[test]
    fn an_unknown_size_shows_no_progress() {
        assert_eq!(shown_progress(10, None), None);
        assert_eq!(shown_progress(10, Some(0)), None);
    }
}
