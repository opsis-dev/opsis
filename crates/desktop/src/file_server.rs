use std::net::{Ipv4Addr, SocketAddr, TcpListener};
use std::path::PathBuf;

use axum::Router;
use axum::extract::{Request, State};
use axum::http::{StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use opsis_compile::asset_ref::{RefTarget, join_url, library_rel_path, ref_target};
use opsis_engine::http_files::file_routes;
use opsis_session::state::AppState;
use percent_encoding::percent_decode_str;
use tauri::AppHandle;
use tower::ServiceExt;

use crate::session_of;

pub struct FileRoot(String);

fn requested_ref(path: &str, token: &str) -> Option<String> {
    let encoded = path
        .strip_prefix('/')?
        .strip_prefix(token)?
        .strip_prefix('/')?;
    let decoded = percent_decode_str(encoded).decode_utf8().ok()?;
    (!decoded.is_empty()).then(|| decoded.into_owned())
}

fn location(app: &AppHandle, file: &str) -> Option<(PathBuf, String)> {
    let session = session_of(app);
    let state = session.state::<AppState>();
    if let Some(rel) = library_rel_path(file) {
        return Some((state.paths.fonts_dir(), rel.to_owned()));
    }
    match ref_target(file) {
        RefTarget::Project(rel) => Some((state.editor().dir()?, rel)),
        RefTarget::App(rel) => Some((state.paths.public_dir.clone(), rel)),
        RefTarget::External(_) | RefTarget::Absolute(_) => None,
    }
}

async fn serve(State((app, token)): State<(AppHandle, String)>, mut request: Request) -> Response {
    let Some(file) = requested_ref(request.uri().path(), &token) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Some((dir, rel)) = location(&app, &file) else {
        return (StatusCode::NOT_FOUND, "Not a file Opsis serves").into_response();
    };
    let Ok(uri) = join_url("/", &rel).parse::<Uri>() else {
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
    use super::requested_ref;

    #[test]
    fn serves_only_refs_under_its_token() {
        assert_eq!(
            requested_ref("/abc/project%3Aassets%2Fclip%20one.mp4", "abc"),
            Some("project:assets/clip one.mp4".into())
        );
        assert_eq!(requested_ref("/abd/project%3Aa.mp4", "abc"), None);
        assert_eq!(requested_ref("/abcd/project%3Aa.mp4", "abc"), None);
        assert_eq!(requested_ref("/project%3Aa.mp4", "abc"), None);
        assert_eq!(requested_ref("/abc/", "abc"), None);
    }
}
