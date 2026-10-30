use std::sync::Arc;

use axum::Router;
use axum::extract::{Path as UrlPath, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use opsis_compile::asset_ref::FOOTAGE_FOLDER;

use crate::FootageError;
use crate::footage::Footage;

async fn serve(
    State(footage): State<Arc<Footage>>,
    UrlPath((id, time)): UrlPath<(String, String)>,
) -> Response {
    let Ok(time_us) = time.trim_end_matches(".jpg").parse::<i64>() else {
        return (
            StatusCode::BAD_REQUEST,
            "A frame's time is whole microseconds.",
        )
            .into_response();
    };
    let done = tokio::task::spawn_blocking(move || footage.picture(&id, time_us)).await;
    match done {
        Ok(Ok(bytes)) => (
            [
                (header::CONTENT_TYPE, "image/jpeg"),
                (header::CACHE_CONTROL, "no-store"),
            ],
            bytes.as_ref().clone(),
        )
            .into_response(),
        Ok(Err(error @ FootageError::Unknown(_))) => {
            (StatusCode::NOT_FOUND, error.to_string()).into_response()
        }
        Ok(Err(error)) => {
            log::warn!("footage frame at {time_us} µs: {error}");
            (StatusCode::UNPROCESSABLE_ENTITY, error.to_string()).into_response()
        }
        Err(error) => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response(),
    }
}

pub fn footage_routes(footage: Arc<Footage>) -> Router {
    Router::new()
        .route(&format!("/{FOOTAGE_FOLDER}/{{id}}/{{time}}"), get(serve))
        .with_state(footage)
}
