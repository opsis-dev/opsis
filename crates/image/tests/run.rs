use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use image::{DynamicImage, GenericImageView, Rgb, RgbImage, Rgba, RgbaImage};
use opsis_engine::tools::EngineFiles;
use opsis_image::executors::image_executors;
use opsis_media::executors::{Tools, media_executors};
use opsis_media::ffmpeg::Ffmpeg;
use opsis_runner::context::{Executors, RunEnv};
use opsis_runner::runner::{RunDeps, run_workflow};
use opsis_types::events::{RunEvent, RunRequest};
use opsis_types::settings::AppSettings;
use opsis_types::workflow::Workflow;
use serde_json::{Value, json};
use tokio_util::sync::CancellationToken;

fn home() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).expect("HOME")
}

fn node(id: &str, kind: &str, params: Value) -> Value {
    json!({ "id": id, "type": kind, "position": { "x": 0, "y": 0 }, "params": params })
}

fn edge(source: &str, out: &str, target: &str, port: &str) -> Value {
    json!({ "id": format!("{source}-{target}-{port}"), "source": source, "sourceHandle": out, "target": target, "targetHandle": port })
}

fn workflow() -> Workflow {
    let nodes = [
        node(
            "photo",
            "media.import",
            json!({ "path": "assets/photo.png" }),
        ),
        node("logo", "media.import", json!({ "path": "assets/logo.png" })),
        node(
            "crop",
            "image.crop",
            json!({ "mode": "margins", "top": 10, "bottom": 10 }),
        ),
        node("turn", "image.rotate", json!({ "turn": "right" })),
        node("size", "image.resize", json!({ "width": 60, "height": 0 })),
        node("look", "image.adjust", json!({ "contrast": 0.2 })),
        node("soft", "image.blur", json!({ "blur": 1.5, "sharpen": 0 })),
        node(
            "hide",
            "image.redact",
            json!({ "areas": [{ "x": 0, "y": 0, "width": 0.25, "height": 0.25 }], "style": "solid", "color": "#000000" }),
        ),
        node(
            "mark",
            "image.place",
            json!({ "x": 0.5, "y": 0.5, "size": 50 }),
        ),
        node(
            "save",
            "image.format",
            json!({ "format": "jpeg", "quality": 90 }),
        ),
    ];
    let edges = [
        edge("photo", "media", "crop", "image"),
        edge("crop", "image", "turn", "image"),
        edge("turn", "image", "size", "image"),
        edge("size", "image", "look", "image"),
        edge("look", "image", "soft", "image"),
        edge("soft", "image", "hide", "image"),
        edge("hide", "image", "mark", "image"),
        edge("logo", "media", "mark", "overlay"),
        edge("mark", "image", "save", "image"),
    ];
    serde_json::from_value(
        json!({ "version": 1, "id": "images", "name": "Images", "nodes": nodes, "edges": edges }),
    )
    .unwrap()
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the dev engine for reading the imported files"]
async fn runs_image_nodes_one_after_another_from_imported_files() {
    let engine = EngineFiles::downloaded(&home().join(".config/Opsis/engine"));
    let base = tempfile::tempdir().unwrap();
    let project = base.path().join("project");
    let temp = base.path().join("tmp");
    std::fs::create_dir_all(project.join("assets")).unwrap();
    std::fs::create_dir_all(&temp).unwrap();
    RgbImage::from_pixel(200, 120, Rgb([20, 90, 200]))
        .save(project.join("assets/photo.png"))
        .unwrap();
    RgbaImage::from_pixel(40, 40, Rgba([255, 255, 255, 255]))
        .save(project.join("assets/logo.png"))
        .unwrap();
    let tools: Tools = Arc::new(Ffmpeg::new(&engine).with_temp(&temp));
    let mut executors = Executors::default();
    executors.extend(media_executors(&tools)).unwrap();
    executors.extend(image_executors()).unwrap();
    let events = Arc::new(Mutex::new(Vec::new()));
    let seen = Arc::clone(&events);
    let deps = RunDeps {
        env: Arc::new(RunEnv {
            app_version: "test".into(),
            engine,
            public_dir: PathBuf::new(),
            temp_dir: temp,
            settings: AppSettings::default(),
        }),
        executors: Arc::new(executors),
        emit: Arc::new(move |e| seen.lock().unwrap().push(e)),
        cancel: CancellationToken::new(),
    };
    let request = RunRequest {
        project_dir: project.to_string_lossy().into_owned(),
        workflow: workflow(),
        target_node_ids: None,
        from_node_id: None,
    };
    run_workflow("images", request, deps).await;
    let events = events.lock().unwrap();
    let errors: Vec<String> = events
        .iter()
        .filter_map(|e| match e {
            RunEvent::NodeError {
                node_id, message, ..
            } => Some(format!("{node_id}: {message}")),
            _ => None,
        })
        .collect();
    assert!(errors.is_empty(), "{errors:#?}");
    let saved = walk(&project.join(".opsis"))
        .into_iter()
        .find(|p| p.extension().is_some_and(|e| e == "jpg"))
        .expect("a saved JPEG");
    let image: DynamicImage = image::open(&saved).unwrap();
    assert_eq!(image.dimensions(), (60, 120));
    let middle = image.get_pixel(30, 60).0;
    assert!(
        middle[0] > 200 && middle[2] > 200,
        "the logo sits in the middle: {middle:?}"
    );
    let corner = image.get_pixel(5, 5).0;
    assert!(
        corner[..3].iter().all(|&v| v < 20),
        "the corner is hidden: {corner:?}"
    );
}

fn walk(dir: &std::path::Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.is_dir() {
            found.extend(walk(&path));
        } else {
            found.push(path);
        }
    }
    found
}

async fn run_two(kind: &str, params: Value, after: (u32, u32)) -> Result<DynamicImage, String> {
    use opsis_compile::vars::Vars;
    use opsis_runner::context::{ExecutorContext, ResultStore};
    let base = tempfile::tempdir().unwrap();
    let project = base.path().to_path_buf();
    std::fs::create_dir_all(project.join("assets")).unwrap();
    std::fs::create_dir_all(project.join(".opsis/cache/h")).unwrap();
    RgbImage::from_pixel(400, 200, Rgb([200, 30, 30]))
        .save(project.join("assets/before.png"))
        .unwrap();
    RgbImage::from_pixel(after.0, after.1, Rgb([30, 30, 200]))
        .save(project.join("assets/after.png"))
        .unwrap();
    let workflow: Workflow = serde_json::from_value(
        json!({ "version": 1, "id": "x", "name": "x", "nodes": [], "edges": [] }),
    )
    .unwrap();
    let workflow = Arc::new(workflow);
    let mut values = opsis_compile::registry::default_params(kind);
    values.extend(params.as_object().cloned().unwrap_or_default());
    let ctx = ExecutorContext {
        run_id: "run".into(),
        node: serde_json::from_value(node("compare", kind, Value::Object(values))).unwrap(),
        workflow: Arc::clone(&workflow),
        env: Arc::new(RunEnv {
            app_version: "test".into(),
            engine: EngineFiles::default(),
            public_dir: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../public"),
            temp_dir: project.clone(),
            settings: AppSettings::default(),
        }),
        project_dir: project.clone(),
        dir: project.join(".opsis/cache/h"),
        rel_dir: ".opsis/cache/h".into(),
        out_dir: project.join(".opsis/out"),
        hash: "h".into(),
        forced: false,
        vars: Arc::new(Vars::default()),
        cancel: CancellationToken::new(),
        emit: Arc::new(|_| {}),
        store: Arc::new(ResultStore::new(workflow)),
    };
    let inputs = serde_json::from_value(json!({
        "image": { "type": "media", "kind": "image", "src": "project:assets/before.png" },
        "after": { "type": "media", "kind": "image", "src": "project:assets/after.png" }
    }))
    .unwrap();
    let executors = image_executors();
    let (_, run) = executors.iter().find(|(known, _)| *known == kind).unwrap();
    let out = run
        .run(Arc::new(ctx), inputs)
        .await
        .map_err(|e| e.to_string())?;
    Ok(image::open(project.join(&out.artifacts[0].path)).unwrap())
}

#[tokio::test(flavor = "multi_thread")]
async fn labels_before_and_after_in_the_projects_main_font() {
    let out = run_two(
        "image.beforeAfter",
        json!({ "labelSize": 30, "dividerWidth": 0 }),
        (400, 200),
    )
    .await
    .unwrap();
    assert_eq!(out.dimensions(), (400, 200));
    let whites = (0..400)
        .flat_map(|x| (0..80).map(move |y| (x, y)))
        .filter(|&(x, y)| out.get_pixel(x, y).0[..3].iter().all(|&v| v > 200))
        .count();
    assert!(whites > 200, "the labels are drawn: {whites}");
    assert_eq!(out.get_pixel(100, 150).0, [200, 30, 30, 255]);
    assert_eq!(out.get_pixel(300, 150).0, [30, 30, 200, 255]);
    let missing = run_two(
        "image.beforeAfter",
        json!({ "font": "No Such Face" }),
        (400, 200),
    )
    .await
    .unwrap_err();
    assert_eq!(
        missing,
        "The font “No Such Face” isn't in this project. Pick another."
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn refuses_to_compare_pictures_of_different_sizes() {
    let refused = run_two("image.compare", json!({}), (400, 210))
        .await
        .unwrap_err();
    assert_eq!(
        refused,
        "The two pictures are different sizes (400 × 200 and 400 × 210). Compare two captures of the same size."
    );
    let marked = run_two("image.compare", json!({ "padding": 0 }), (400, 200))
        .await
        .unwrap();
    assert_eq!(
        marked.get_pixel(0, 0).0,
        [239, 68, 68, 255],
        "the whole picture changed"
    );
}
