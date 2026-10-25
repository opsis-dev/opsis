use std::net::{Ipv4Addr, SocketAddr};
use std::path::Path;

use axum::Router;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

pub fn file_routes(root: &Path) -> Router {
    Router::new()
        .fallback_service(ServeDir::new(root))
        .layer(CorsLayer::new().allow_origin(Any))
}

pub struct FileServer {
    pub url: String,
    stop: CancellationToken,
}

impl FileServer {
    pub async fn start(root: &Path) -> std::io::Result<Self> {
        let listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0))).await?;
        let url = format!("http://{}/", listener.local_addr()?);
        let stop = CancellationToken::new();
        let stopped = stop.clone();
        let routes = file_routes(root);
        tokio::spawn(async move {
            let serving =
                axum::serve(listener, routes).with_graceful_shutdown(stopped.cancelled_owned());
            if let Err(error) = serving.await {
                log::warn!("file server stopped: {error}");
            }
        });
        Ok(Self { url, stop })
    }
}

impl Drop for FileServer {
    fn drop(&mut self) {
        self.stop.cancel();
    }
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode, header};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use super::*;

    async fn get(
        root: &Path,
        path: &str,
        range: Option<&str>,
    ) -> (StatusCode, axum::http::HeaderMap, Vec<u8>) {
        let mut request = Request::get(path);
        if let Some(range) = range {
            request = request.header(header::RANGE, range);
        }
        let response = file_routes(root)
            .oneshot(request.body(Body::empty()).unwrap())
            .await
            .unwrap();
        let (parts, body) = response.into_parts();
        (
            parts.status,
            parts.headers,
            body.collect().await.unwrap().to_bytes().to_vec(),
        )
    }

    fn project() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("assets")).unwrap();
        std::fs::write(
            dir.path().join("assets/clip one.mp4"),
            (0u8..100).collect::<Vec<_>>(),
        )
        .unwrap();
        dir
    }

    #[tokio::test]
    async fn serves_whole_files_open_closed_and_suffix_ranges_to_any_page() {
        let dir = project();
        let (status, headers, body) = get(dir.path(), "/assets/clip%20one.mp4", None).await;
        assert_eq!((status, body.len()), (StatusCode::OK, 100));
        assert_eq!(headers[header::CONTENT_TYPE], "video/mp4");
        assert_eq!(headers[header::ACCEPT_RANGES], "bytes");
        assert_eq!(headers[header::ACCESS_CONTROL_ALLOW_ORIGIN], "*");
        for (range, from, to) in [
            ("bytes=0-", 0u8, 99u8),
            ("bytes=10-19", 10, 19),
            ("bytes=10-500", 10, 99),
            ("bytes=-30", 70, 99),
        ] {
            let (status, headers, body) =
                get(dir.path(), "/assets/clip%20one.mp4", Some(range)).await;
            assert_eq!(status, StatusCode::PARTIAL_CONTENT, "{range}");
            assert_eq!(body, (from..=to).collect::<Vec<_>>(), "{range}");
            assert_eq!(
                headers[header::CONTENT_RANGE],
                format!("bytes {from}-{to}/100")
            );
        }
    }

    #[tokio::test]
    async fn refuses_ranges_outside_the_file() {
        let dir = project();
        for range in ["bytes=100-", "bytes=50-10"] {
            let (status, headers, _) = get(dir.path(), "/assets/clip%20one.mp4", Some(range)).await;
            assert_eq!(status, StatusCode::RANGE_NOT_SATISFIABLE, "{range}");
            assert_eq!(headers[header::CONTENT_RANGE], "bytes */100");
        }
    }

    #[tokio::test]
    async fn serves_nothing_outside_the_folder_and_nothing_missing() {
        let dir = project();
        std::fs::write(dir.path().parent().unwrap().join("secret.txt"), "no").unwrap();
        for path in [
            "/../secret.txt",
            "/assets/../../secret.txt",
            "/%2e%2e/secret.txt",
            "/assets/gone.mp4",
            "/assets",
        ] {
            let (status, _, _) = get(dir.path(), path, None).await;
            assert!(
                status.is_client_error() || status.is_redirection(),
                "{path}: {status}"
            );
        }
    }

    #[tokio::test]
    async fn answers_on_a_local_port_until_dropped() {
        let dir = project();
        let server = FileServer::start(dir.path()).await.unwrap();
        assert!(server.url.starts_with("http://127.0.0.1:"));
        let bytes = reqwest::get(format!("{}assets/clip%20one.mp4", server.url))
            .await
            .unwrap()
            .bytes()
            .await
            .unwrap();
        assert_eq!(bytes.len(), 100);
    }
}
