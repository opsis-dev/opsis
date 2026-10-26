use std::io::ErrorKind;
use std::sync::{Arc, Mutex};

use opsis_types::shell::ApiServerStatus;
use tokio_util::sync::CancellationToken;

use crate::address::{API_SOCKET_ENV, ApiAddress};
use crate::routes::{ApiHost, routes};

const IN_USE: &str = "Another copy of Opsis is already using this address.";

struct Running {
    stop: CancellationToken,
}

pub struct ApiServer {
    address: ApiAddress,
    running: Mutex<Option<Running>>,
    status: Mutex<ApiServerStatus>,
}

#[cfg(unix)]
fn listen_error(error: &std::io::Error, address: &str) -> String {
    let folder = std::path::Path::new(address)
        .parent()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();
    match error.kind() {
        ErrorKind::NotFound => format!(
            "The folder {folder} doesn't exist. Create it, or set {API_SOCKET_ENV} to a path in a folder that does, and restart Opsis."
        ),
        ErrorKind::PermissionDenied => format!(
            "Opsis can't create files in {folder}. Set {API_SOCKET_ENV} to a path in a folder you own, and restart Opsis."
        ),
        ErrorKind::AddrInUse => format!(
            "A file that isn't Opsis's API is already at {address}. Move it, or set {API_SOCKET_ENV} to another path, and restart Opsis."
        ),
        _ => error.to_string(),
    }
}

#[cfg(unix)]
async fn serve(
    address: &str,
    host: Arc<dyn ApiHost>,
    stop: CancellationToken,
) -> Result<(), String> {
    use std::os::unix::fs::{FileTypeExt, PermissionsExt};

    if tokio::net::UnixStream::connect(address).await.is_ok() {
        return Err(IN_USE.to_owned());
    }
    if std::fs::symlink_metadata(address).is_ok_and(|m| m.file_type().is_socket()) {
        std::fs::remove_file(address).map_err(|e| listen_error(&e, address))?;
    }
    let listener =
        tokio::net::UnixListener::bind(address).map_err(|e| listen_error(&e, address))?;
    std::fs::set_permissions(address, std::fs::Permissions::from_mode(0o600))
        .map_err(|e| listen_error(&e, address))?;
    let path = address.to_owned();
    tokio::spawn(async move {
        let served = axum::serve(listener, routes(host))
            .with_graceful_shutdown(stop.cancelled_owned())
            .await;
        if let Err(error) = served {
            log::warn!("the local API stopped: {error}");
        }
        if std::fs::symlink_metadata(&path).is_ok_and(|m| m.file_type().is_socket())
            && let Err(error) = std::fs::remove_file(&path)
        {
            log::warn!("removing the API socket: {error}");
        }
    });
    Ok(())
}

#[cfg(windows)]
async fn serve(
    address: &str,
    host: Arc<dyn ApiHost>,
    stop: CancellationToken,
) -> Result<(), String> {
    use hyper_util::rt::{TokioExecutor, TokioIo};
    use hyper_util::server::conn::auto::Builder;
    use hyper_util::service::TowerToHyperService;
    use tokio::net::windows::named_pipe::ServerOptions;

    let mut server = ServerOptions::new()
        .first_pipe_instance(true)
        .create(address)
        .map_err(|e| {
            if e.kind() == ErrorKind::PermissionDenied {
                IN_USE.to_owned()
            } else {
                e.to_string()
            }
        })?;
    let address = address.to_owned();
    let app = routes(host);
    tokio::spawn(async move {
        loop {
            tokio::select! {
                () = stop.cancelled() => return,
                connected = server.connect() => {
                    if let Err(error) = connected {
                        log::warn!("the local API pipe: {error}");
                        return;
                    }
                }
            }
            let client = server;
            server = match ServerOptions::new().create(&address) {
                Ok(next) => next,
                Err(error) => {
                    log::warn!("the local API pipe: {error}");
                    return;
                }
            };
            let service = TowerToHyperService::new(app.clone());
            tokio::spawn(async move {
                if let Err(error) = Builder::new(TokioExecutor::new())
                    .serve_connection(TokioIo::new(client), service)
                    .await
                {
                    log::debug!("an API connection ended: {error}");
                }
            });
        }
    });
    Ok(())
}

impl ApiServer {
    pub fn new(address: ApiAddress) -> Self {
        let status = ApiServerStatus {
            enabled: false,
            listening: false,
            address: address.address.clone(),
            error: None,
        };
        Self {
            address,
            running: Mutex::new(None),
            status: Mutex::new(status),
        }
    }

    pub fn status(&self) -> ApiServerStatus {
        self.status.lock().expect("API status").clone()
    }

    fn set_status(&self, listening: bool, enabled: bool, error: Option<String>) -> ApiServerStatus {
        let status = ApiServerStatus {
            enabled,
            listening,
            address: self.address.address.clone(),
            error,
        };
        *self.status.lock().expect("API status") = status.clone();
        status
    }

    pub fn stop(&self) {
        if let Some(running) = self.running.lock().expect("API server").take() {
            running.stop.cancel();
        }
    }

    pub async fn set_enabled(&self, enabled: bool, host: Arc<dyn ApiHost>) -> ApiServerStatus {
        if !enabled {
            self.stop();
            return self.set_status(false, false, None);
        }
        if self.running.lock().expect("API server").is_some() {
            return self.status();
        }
        if let Some(error) = &self.address.error {
            return self.set_status(false, true, Some(error.clone()));
        }
        let stop = CancellationToken::new();
        match serve(&self.address.address, host, stop.clone()).await {
            Ok(()) => {
                *self.running.lock().expect("API server") = Some(Running { stop });
                self.set_status(true, true, None)
            }
            Err(error) => self.set_status(false, true, Some(error)),
        }
    }
}
