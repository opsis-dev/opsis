use std::net::{Ipv4Addr, SocketAddr, TcpListener};
use std::path::PathBuf;

use axum::Router;
use axum::extract::{Request, State};
use axum::http::{StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use opsis_engine::http_files::file_routes;
use tauri::{AppHandle, Manager};
use tower::ServiceExt;

use crate::state::AppState;

pub struct FileRoot(String);

fn root_dir(app: &AppHandle, root: &str) -> Option<PathBuf> {
    let state = app.state::<AppState>();
    match root {
        "project" => state.editor().dir(),
        "fonts" => Some(state.paths.fonts_dir()),
        "app" => Some(state.paths.public_dir.clone()),
        _ => None,
    }
}

fn split_path<'a>(path: &'a str, token: &str) -> Option<(&'a str, &'a str)> {
    path.strip_prefix('/')?
        .strip_prefix(token)?
        .strip_prefix('/')?
        .split_once('/')
}

async fn serve(State((app, token)): State<(AppHandle, String)>, mut request: Request) -> Response {
    let path = request.uri().path().to_owned();
    let Some((root, file)) = split_path(&path, &token) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Some(dir) = root_dir(&app, root) else {
        return (StatusCode::NOT_FOUND, "No project open").into_response();
    };
    let Ok(uri) = format!("/{file}").parse::<Uri>() else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    *request.uri_mut() = uri;
    match file_routes(&dir).oneshot(request).await {
        Ok(response) => response,
        Err(error) => match error {},
    }
}

pub fn start(app: &AppHandle) -> std::io::Result<FileRoot> {
    let listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))?;
    listener.set_nonblocking(true)?;
    let token = uuid::Uuid::new_v4().simple().to_string();
    let root = format!("http://{}/{token}/", listener.local_addr()?);
    let routes = Router::new()
        .fallback(serve)
        .with_state((app.clone(), token));
    tauri::async_runtime::spawn(async move {
        let serving =
            async { axum::serve(tokio::net::TcpListener::from_std(listener)?, routes).await };
        if let Err(error) = serving.await {
            log::warn!("the file server stopped: {error}");
        }
    });
    Ok(FileRoot(root))
}

#[tauri::command]
#[specta::specta]
pub fn file_root(root: tauri::State<'_, FileRoot>) -> String {
    root.0.clone()
}

#[cfg(test)]
mod tests {
    use super::split_path;

    #[test]
    fn serves_only_paths_under_its_token() {
        assert_eq!(
            split_path("/abc/project/assets/clip%20one.mp4", "abc"),
            Some(("project", "assets/clip%20one.mp4"))
        );
        assert_eq!(split_path("/abd/project/a.mp4", "abc"), None);
        assert_eq!(split_path("/abcd/project/a.mp4", "abc"), None);
        assert_eq!(split_path("/project/a.mp4", "abc"), None);
        assert_eq!(split_path("/abc/project", "abc"), None);
    }
}
