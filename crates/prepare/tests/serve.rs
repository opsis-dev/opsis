use opsis_compile::asset_ref::PreparedRef;
use opsis_golden::repo;
use opsis_prepare::serve::{PreparedDirs, prepare};

fn made(dirs: &PreparedDirs, path: &str) -> Option<Vec<u8>> {
    let prepared = PreparedRef::from_path(path)?;
    prepare(dirs, &prepared).ok().map(|made| made.bytes)
}

#[test]
fn prepares_once_keeps_the_file_and_refuses_what_it_cant_read() {
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
    let bytes = made(
        &dirs,
        "typeface/700/app/fonts/bundled/inter/inter-latin-wght-normal.woff2",
    )
    .unwrap();
    let typeface: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(typeface["familyName"], "Inter");
    let model = made(&dirs, "model/project/assets/draco.glb").unwrap();
    assert_eq!(&model[..4], b"glTF");
    let kept = std::fs::read_dir(project.path().join(".opsis/cache/prepared"))
        .unwrap()
        .count();
    assert_eq!(kept, 2);
    assert_eq!(made(&dirs, "model/project/assets/draco.glb"), Some(model));
    assert_eq!(made(&dirs, "model/project/assets/broken.glb"), None);
    assert_eq!(made(&dirs, "model/project/../secrets/x.glb"), None);
    assert_eq!(made(&dirs, "video/project/assets/a.mp4"), None);
}
