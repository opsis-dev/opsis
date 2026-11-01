use std::path::Path;
use std::sync::{Arc, Mutex};

use opsis_engine::tools::EngineFiles;
use opsis_golden::{edge, home, node, repo};
use opsis_media::executors::{Tools, media_executors};
use opsis_media::ffmpeg::Ffmpeg;
use opsis_render::executors::{RenderEngine, output_executors};
use opsis_runner::context::{Executors, RunEnv};
use opsis_runner::runner::{RunDeps, run_workflow};
use opsis_types::events::{RunEvent, RunRequest};
use opsis_types::settings::AppSettings;
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
        node("still", "output.still", json!({ "fileName": "film" })),
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
        edge("title", "clip", "still", "clip"),
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
    let base = home().join(".cache/opsis-render-test");
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
    for file in ["film.mp4", "film-og.png"] {
        assert!(
            project.join(".opsis/out").join(file).exists(),
            "{file} wasn't exported"
        );
    }
}
