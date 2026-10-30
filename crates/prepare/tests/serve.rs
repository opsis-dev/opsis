use opsis_golden::repo;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use http_body_util::BodyExt;
use opsis_prepare::serve::{PreparedDirs, prepared_routes};
use tower::ServiceExt;

async fn get(dirs: &PreparedDirs, path: &str) -> (StatusCode, String, Vec<u8>) {
    let response = prepared_routes(dirs.clone())
        .oneshot(Request::get(path).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let kind = response
        .headers()
        .get(header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap().to_owned())
        .unwrap_or_default();
    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes()
        .to_vec();
    (status, kind, bytes)
}

#[tokio::test]
async fn prepares_once_keeps_the_file_and_refuses_what_it_cant_read() {
    let project = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(project.path().join("assets")).unwrap();
    std::fs::copy(
        repo().join("fixtures/prepare/models/draco.glb"),
        project.path().join("assets/draco.glb"),
    )
    .unwrap();
    std::fs::write(project.path().join("assets/broken.glb"), "not a model").unwrap();
    let dirs = PreparedDirs {
        project: project.path().to_path_buf(),
        public: repo().join("public"),
    };
    let (status, kind, bytes) = get(
        &dirs,
        "/.opsis/prepared/typeface/700/app/fonts/bundled/inter/inter-latin-wght-normal.woff2",
    )
    .await;
    assert_eq!(
        (status, kind.as_str()),
        (StatusCode::OK, "application/json")
    );
    let typeface: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(typeface["familyName"], "Inter");
    let (status, kind, bytes) = get(&dirs, "/.opsis/prepared/model/project/assets/draco.glb").await;
    assert_eq!(
        (status, kind.as_str(), &bytes[..4]),
        (StatusCode::OK, "model/gltf-binary", &b"glTF"[..])
    );
    let kept = std::fs::read_dir(project.path().join(".opsis/cache/prepared"))
        .unwrap()
        .count();
    assert_eq!(kept, 2);
    let (again, _, same) = get(&dirs, "/.opsis/prepared/model/project/assets/draco.glb").await;
    assert_eq!((again, same), (StatusCode::OK, bytes));
    let (status, _, _) = get(&dirs, "/.opsis/prepared/model/project/assets/broken.glb").await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let (status, _, _) = get(&dirs, "/.opsis/prepared/model/project/../secrets/x.glb").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _, _) = get(&dirs, "/.opsis/prepared/video/project/assets/a.mp4").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
