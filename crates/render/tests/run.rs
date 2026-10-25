use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use opsis_engine::paths::{ShellPlatform, engine_paths};
use opsis_media::executors::{Tools, media_executors};
use opsis_media::ffmpeg::Ffmpeg;
use opsis_project::files::workflow_text;
use opsis_render::executors::{RenderEngine, output_executors};
use opsis_render::export_project::ExportSources;
use opsis_render::sidecar::{Sidecar, SidecarCommand};
use opsis_runner::context::{Executors, RunEnv};
use opsis_runner::runner::{RunDeps, run_workflow};
use opsis_types::events::{RunEvent, RunRequest};
use opsis_types::settings::AppSettings;
use opsis_types::workflow::{WORKFLOW_FILE, Workflow};
use serde_json::{Value, json};
use tokio_util::sync::CancellationToken;

fn home() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).expect("HOME")
}

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
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
        node("probe", "media.probe", json!({})),
        node("look", "grade.colour", json!({ "contrast": 1.2 })),
        node("graded", "media.applyGrade", json!({})),
        node("joined", "media.concat", json!({})),
        node("title", "fx.core.title", json!({})),
        node(
            "click",
            "audio.sfx",
            json!({ "src": "app:sfx/mouse-click.wav", "atMs": 300 }),
        ),
        node("mixed", "audio.mix", json!({})),
        node("film", "output.render", json!({ "fileName": "film" })),
        node("still", "output.still", json!({ "fileName": "film" })),
        node("code", "output.exportProject", json!({})),
    ];
    let edges = [
        edge("import", "media", "trim", "media"),
        edge("trim", "media", "rev", "media"),
        edge("rev", "media", "speed", "media"),
        edge("import", "media", "thumb", "media"),
        edge("import", "media", "probe", "media"),
        edge("import", "media", "graded", "media"),
        edge("look", "grade", "graded", "grade"),
        edge("import", "media", "joined", "in1"),
        edge("rev", "media", "joined", "in2"),
        edge("title", "clip", "mixed", "clip"),
        edge("click", "audio", "mixed", "sounds"),
        edge("mixed", "clip", "film", "clip"),
        edge("title", "clip", "still", "clip"),
        edge("mixed", "clip", "code", "clip"),
    ];
    serde_json::from_value(
        json!({ "version": 1, "id": "e2e", "name": "End to end", "nodes": nodes, "edges": edges }),
    )
    .unwrap()
}

async fn clip_in(ffmpeg: &Path, base: &Path, project: &Path) {
    let clip = base.join("clip.mp4");
    if !clip.exists() {
        make_clip(ffmpeg, &clip).await;
    }
    let file = project.join("assets/clip.mp4");
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::copy(&clip, &file).unwrap();
    let modified = std::fs::metadata(&clip).unwrap().modified().unwrap();
    std::fs::File::options()
        .write(true)
        .open(&file)
        .unwrap()
        .set_modified(modified)
        .unwrap();
}

async fn make_clip(ffmpeg: &Path, file: &Path) {
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
#[ignore = "needs the dev engine and the video bundle in ~/.cache/opsis-bun-test"]
async fn runs_media_nodes_and_exports_through_the_sidecar() {
    let engine_dir = home().join(".config/Opsis/engine");
    let paths = engine_paths(&engine_dir, ShellPlatform::current().unwrap());
    let base = home().join(".cache/opsis-render-test");
    let project = base.join("project");
    let temp = base.join("tmp");
    if project.exists() {
        std::fs::remove_dir_all(&project).unwrap();
    }
    std::fs::create_dir_all(&temp).unwrap();
    clip_in(&paths.ffmpeg, &base, &project).await;
    std::fs::write(project.join(WORKFLOW_FILE), workflow_text(&workflow())).unwrap();
    let tools: Tools = Arc::new(Ffmpeg::new(&paths).with_temp(&temp));
    let env_vars = ["TMPDIR", "TEMP", "TMP"]
        .map(|k| (OsString::from(k), temp.clone().into_os_string()))
        .to_vec();
    let command = SidecarCommand {
        program: "bun".into(),
        args: ["run", "apps/sidecar/src/main.ts"]
            .map(OsString::from)
            .to_vec(),
        env: env_vars,
        cwd: Some(repo()),
    };
    let render = Arc::new(RenderEngine {
        sidecar: Arc::new(Sidecar::new(
            command,
            Arc::new(|line| eprintln!("sidecar: {line}")),
        )),
        tools: Arc::clone(&tools),
        serve_url: std::env::var("OPSIS_VIDEO_BUNDLE").unwrap_or_else(|_| {
            home()
                .join(".cache/opsis-bun-test/bundle")
                .to_string_lossy()
                .into_owned()
        }),
        binaries_directory: None,
        sources: ExportSources {
            root: repo(),
            packaged: false,
        },
    });
    let mut executors = Executors::default();
    executors.extend(media_executors(&tools)).unwrap();
    executors.extend(output_executors(&render)).unwrap();
    let events = Arc::new(Mutex::new(Vec::new()));
    let seen = Arc::clone(&events);
    let deps = RunDeps {
        env: Arc::new(RunEnv {
            app_version: "test".into(),
            engine_dir,
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
    for file in [
        "film.mp4",
        "film-og.png",
        "video-code/package.json",
        "video-code/src/timeline.json",
        "video-code/remotion.config.ts",
    ] {
        assert!(
            project.join(".opsis/out").join(file).exists(),
            "{file} wasn't exported"
        );
    }
}
