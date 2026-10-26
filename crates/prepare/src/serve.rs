use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::Router;
use axum::extract::{Path as UrlPath, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use opsis_compile::asset_ref::{
    PREPARED_FOLDER, PreparedKind, PreparedRef, RefTarget, extension_of,
};
use opsis_engine::files::{atomic_write, file_identity};
use opsis_engine::hash::versioned_hash;

use crate::{PREPARED_VERSION, PrepareError, model, svg, texture, typeface};

const KEY_LENGTH: usize = 32;
const CACHE_FOLDER: &str = ".opsis/cache/prepared";

#[derive(Debug, Clone)]
pub struct PreparedDirs {
    pub project: PathBuf,
    pub public: PathBuf,
}

pub struct Prepared {
    pub bytes: Vec<u8>,
    pub content_type: &'static str,
}

fn source_file(dirs: &PreparedDirs, prepared: &PreparedRef) -> Option<PathBuf> {
    match &prepared.source {
        RefTarget::Project(rel) => Some(dirs.project.join(rel)),
        RefTarget::App(rel) => Some(dirs.public.join(rel)),
        RefTarget::External(_) | RefTarget::Absolute(_) => None,
    }
}

fn image_type(file: &str) -> &'static str {
    match extension_of(file).as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "avif" => "image/avif",
        "svg" => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

fn content_type(prepared: &PreparedRef, source: &Path) -> &'static str {
    match prepared.kind {
        PreparedKind::Typeface { .. } => "application/json",
        PreparedKind::Model => "model/gltf-binary",
        PreparedKind::Svg => "image/svg+xml",
        PreparedKind::Texture => image_type(&source.to_string_lossy()),
    }
}

fn make(prepared: &PreparedRef, source: &Path) -> Result<Vec<u8>, PrepareError> {
    let bytes = std::fs::read(source).map_err(|e| PrepareError::Detail(e.to_string()))?;
    match prepared.kind {
        PreparedKind::Typeface { weight } => {
            Ok(serde_json::to_vec(&typeface::typeface(&bytes, weight)?)
                .map_err(|e| PrepareError::Detail(e.to_string()))?)
        }
        PreparedKind::Model => model::model(&bytes, source.parent().unwrap_or(Path::new("."))),
        PreparedKind::Svg => svg::svg(&bytes),
        PreparedKind::Texture if extension_of(&source.to_string_lossy()) == "svg" => Ok(bytes),
        PreparedKind::Texture => texture::texture(&bytes),
    }
}

pub fn prepare(dirs: &PreparedDirs, prepared: &PreparedRef) -> Result<Prepared, PrepareError> {
    let source = source_file(dirs, prepared).ok_or(PrepareError::Unreadable)?;
    let identity = file_identity(&source).map_err(|e| PrepareError::Detail(e.to_string()))?;
    let key = versioned_hash(PREPARED_VERSION, &(prepared.path(), identity), KEY_LENGTH);
    let cached = dirs.project.join(CACHE_FOLDER).join(&key);
    let content_type = content_type(prepared, &source);
    if let Ok(bytes) = std::fs::read(&cached) {
        return Ok(Prepared {
            bytes,
            content_type,
        });
    }
    let bytes = make(prepared, &source)?;
    if let Some(folder) = cached.parent()
        && let Err(error) =
            std::fs::create_dir_all(folder).and_then(|()| atomic_write(&cached, &bytes))
    {
        log::warn!("keeping a prepared file: {error}");
    }
    Ok(Prepared {
        bytes,
        content_type,
    })
}

async fn serve(State(dirs): State<Arc<PreparedDirs>>, UrlPath(path): UrlPath<String>) -> Response {
    let Some(prepared) = PreparedRef::from_path(&path) else {
        return (StatusCode::NOT_FOUND, "No such prepared file").into_response();
    };
    let done = tokio::task::spawn_blocking(move || prepare(&dirs, &prepared)).await;
    match done {
        Ok(Ok(made)) => ([(header::CONTENT_TYPE, made.content_type)], made.bytes).into_response(),
        Ok(Err(error)) => {
            log::warn!("preparing {path}: {error}");
            (StatusCode::UNPROCESSABLE_ENTITY, error.to_string()).into_response()
        }
        Err(error) => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response(),
    }
}

pub fn prepared_routes(dirs: PreparedDirs) -> Router {
    Router::new()
        .route(&format!("/{PREPARED_FOLDER}/{{*path}}"), get(serve))
        .with_state(Arc::new(dirs))
}
