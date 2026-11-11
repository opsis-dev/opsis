use std::path::Path;
use std::sync::{Arc, Mutex};

use opsis_compile::asset_ref::project_rel_path;
use opsis_compile::terminal::preview::{PREVIEW_FILE, PREVIEW_FPS, preview_size};
use opsis_engine::tools::EngineFiles;
use opsis_golden::{dev_cache, edge, home, node, repo};
use opsis_media::executors::{Tools, media_executors};
use opsis_media::ffmpeg::Ffmpeg;
use opsis_probe::probe;
use opsis_render::executors::{RenderEngine, output_executors};
use opsis_render::preview::TakePreviews;
use opsis_runner::context::{Executors, RunEnv};
use opsis_runner::runner::{RunDeps, run_workflow};
use opsis_terminal::executors::{TerminalDeps, terminal_executors};
use opsis_types::events::{RunEvent, RunRequest};
use opsis_types::settings::AppSettings;
use opsis_types::values::{ArtifactKind, CompileValue, TakeSource};
use opsis_types::workflow::Workflow;
use serde_json::json;
use tokio_util::sync::CancellationToken;

fn workflow() -> Workflow {
    let nodes = [
        node(
            "import",
            "media.import",
            json!({ "path": "assets/clip.mp4" }),
        ),
        node(
            "trim",
            "media.trim",
            json!({ "startMs": 500, "endMs": 1500 }),
        ),
        node("rev", "media.reverse", json!({})),
        node("speed", "media.speed", json!({ "rate": 2 })),
        node("thumb", "media.thumbnail", json!({ "atMs": 500 })),
        node("look", "grade.colour", json!({ "contrast": 1.2 })),
        node("graded", "media.applyGrade", json!({})),
        node("joined", "media.concat", json!({})),
        node("title", "fx.title", json!({})),
        node(
            "click",
            "audio.sfx",
            json!({ "src": "app:sfx/mouse-click.wav", "atMs": 300 }),
        ),
        node("mixed", "audio.mix", json!({})),
        node("film", "output.render", json!({ "fileName": "film" })),
        node(
            "lossless-mp4",
            "output.render",
            json!({ "fileName": "lossless", "codec": "h264-lossless" }),
        ),
        node(
            "lossless-mkv",
            "output.render",
            json!({ "fileName": "lossless", "codec": "ffv1" }),
        ),
        node("still", "output.still", json!({ "fileName": "film" })),
        node("as-clip", "media.toClip", json!({})),
        node(
            "imported",
            "output.render",
            json!({ "fileName": "imported" }),
        ),
    ];
    let edges = [
        edge("import", "media", "trim", "media"),
        edge("trim", "media", "rev", "media"),
        edge("rev", "media", "speed", "media"),
        edge("import", "media", "thumb", "media"),
        edge("import", "media", "graded", "media"),
        edge("look", "grade", "graded", "grade"),
        edge("import", "media", "joined", "in1"),
        edge("rev", "media", "joined", "in2"),
        edge("title", "clip", "mixed", "clip"),
        edge("click", "audio", "mixed", "sounds"),
        edge("mixed", "clip", "film", "clip"),
        edge("mixed", "clip", "lossless-mp4", "clip"),
        edge("mixed", "clip", "lossless-mkv", "clip"),
        edge("title", "clip", "still", "clip"),
        edge("import", "media", "as-clip", "media"),
        edge("as-clip", "clip", "imported", "clip"),
    ];
    serde_json::from_value(
        json!({ "version": 1, "id": "e2e", "name": "End to end", "nodes": nodes, "edges": edges }),
    )
    .unwrap()
}

async fn make_clip(ffmpeg: &Path, file: &Path) {
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    let args = [
        "-y",
        "-f",
        "lavfi",
        "-i",
        "testsrc=duration=2:size=640x360:rate=30",
        "-f",
        "lavfi",
        "-i",
        "sine=frequency=440:duration=2",
        "-c:v",
        "libx264",
        "-pix_fmt",
        "yuv420p",
        "-c:a",
        "aac",
        "-shortest",
    ];
    let made = tokio::process::Command::new(ffmpeg)
        .args(args)
        .arg(file)
        .output()
        .await
        .unwrap();
    assert!(
        made.status.success(),
        "{}",
        String::from_utf8_lossy(&made.stderr)
    );
}

#[tokio::test]
#[ignore = "needs the dev engine and the film page (bun run film:bundle)"]
async fn runs_media_nodes_and_exports_a_film() {
    let engine = EngineFiles::downloaded(&home().join(".config/Opsis/engine"));
    let base = dev_cache("render-test");
    let project = base.join("project");
    let temp = base.join("tmp");
    if project.exists() {
        std::fs::remove_dir_all(&project).unwrap();
    }
    std::fs::create_dir_all(&temp).unwrap();
    make_clip(&opsis_golden::ffmpeg(), &project.join("assets/clip.mp4")).await;
    let tools: Tools = Arc::new(Ffmpeg::new(&engine).with_temp(&temp));
    let render = Arc::new(RenderEngine {
        tools: Arc::clone(&tools),
        film_bundle: repo().join("film-bundle"),
    });
    let mut executors = Executors::default();
    executors.extend(media_executors(&tools)).unwrap();
    executors.extend(output_executors(&render)).unwrap();
    let events = Arc::new(Mutex::new(Vec::new()));
    let seen = Arc::clone(&events);
    let deps = RunDeps {
        env: Arc::new(RunEnv {
            app_version: "test".into(),
            engine,
            public_dir: repo().join("public"),
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
    run_workflow("e2e", request, deps).await;
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
    assert!(
        matches!(events.last(), Some(RunEvent::RunDone { ok: true, .. })),
        "{:?}",
        events.last()
    );
    for file in ["film.mp4", "film-og.png", "imported.mp4"] {
        assert!(
            project.join(".opsis/out").join(file).exists(),
            "{file} wasn't exported"
        );
    }
    for (file, picture, sound) in [
        ("lossless.mp4", "h264", "alac"),
        ("lossless.mkv", "ffv1", "flac"),
    ] {
        let found =
            probe(&project.join(".opsis/out").join(file)).unwrap_or_else(|e| panic!("{file}: {e}"));
        let video = found.video.expect("a picture");
        assert_eq!(
            (video.codec.as_str(), found.audio.map(|a| a.codec)),
            (picture, Some(sound.to_owned())),
            "{file}"
        );
        assert!(
            matches!(video.pix_fmt.as_str(), "gbrp" | "bgr0"),
            "{file}: {}",
            video.pix_fmt
        );
    }
}

#[tokio::test]
#[ignore = "needs the dev engine and the film page (bun run film:bundle)"]
async fn draws_a_preview_of_every_terminal_take() {
    let engine = EngineFiles::downloaded(&home().join(".config/Opsis/engine"));
    let base = dev_cache("render-test/preview");
    let project = base.join("project");
    let temp = base.join("tmp");
    if project.exists() {
        std::fs::remove_dir_all(&project).unwrap();
    }
    std::fs::create_dir_all(&project).unwrap();
    std::fs::create_dir_all(&temp).unwrap();
    std::fs::copy(
        repo().join("crates/terminal/tests/casts/demo.cast"),
        project.join("demo.cast"),
    )
    .unwrap();
    let tools: Tools = Arc::new(Ffmpeg::new(&engine).with_temp(&temp));
    let render = Arc::new(RenderEngine {
        tools,
        film_bundle: repo().join("film-bundle"),
    });
    let mut executors = Executors::default();
    executors
        .extend(terminal_executors(&TerminalDeps {
            previews: Some(Arc::new(TakePreviews::new(render))),
            ..TerminalDeps::default()
        }))
        .unwrap();
    let events = Arc::new(Mutex::new(Vec::new()));
    let seen = Arc::clone(&events);
    let deps = RunDeps {
        env: Arc::new(RunEnv {
            app_version: "test".into(),
            engine,
            public_dir: repo().join("public"),
            temp_dir: temp,
            settings: AppSettings::default(),
        }),
        executors: Arc::new(executors),
        emit: Arc::new(move |e| seen.lock().unwrap().push(e)),
        cancel: CancellationToken::new(),
    };
    let workflow: Workflow = serde_json::from_value(json!({
        "version": 1, "id": "preview", "name": "Preview", "edges": [],
        "nodes": [node("cast", "terminal.castFile", json!({ "file": "project:demo.cast" }))],
    }))
    .unwrap();
    let request = RunRequest {
        project_dir: project.to_string_lossy().into_owned(),
        workflow,
        target_node_ids: None,
        from_node_id: None,
    };
    run_workflow("preview", request, deps).await;
    let events = events.lock().unwrap();
    let result = events
        .iter()
        .find_map(|e| match e {
            RunEvent::NodeDone {
                result: Some(result),
                ..
            } => Some(result),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no result: {events:#?}"));
    let Some(CompileValue::Recording(take)) = result.output("recording") else {
        panic!("no take: {result:?}");
    };
    let TakeSource::Terminal(source) = &take.source else {
        panic!("not a terminal take");
    };
    let preview = source.preview.as_deref().expect("a preview");
    let file = project.join(project_rel_path(preview).unwrap());
    assert!(
        result
            .artifacts
            .iter()
            .any(|a| a.kind == ArtifactKind::Video && a.path.ends_with(PREVIEW_FILE)),
        "{:?}",
        result.artifacts
    );
    let probed = probe(&file).unwrap();
    let video = probed.video.unwrap();
    let size = preview_size(take).unwrap();
    assert_eq!((video.width, video.height), (size.width, size.height));
    let length = f64::from(probed.duration_ms.unwrap());
    assert!(
        (length - take.duration_ms).abs() <= 1000.0 / f64::from(PREVIEW_FPS) * 2.0,
        "{length} ms against a {} ms take",
        take.duration_ms
    );
}
