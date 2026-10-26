use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use opsis_compile::clip::clip_to_spec;
use opsis_compile::elements::element_node_type;
use opsis_compile::node::{CLIP_PORT, Ctx};
use opsis_compile::registry::node_def;
use opsis_compile::vars::Vars;
use opsis_engine::http_files::FileServer;
use opsis_engine::paths::{ShellPlatform, engine_paths, find_headless_shell};
use opsis_prepare::serve::{PreparedDirs, prepared_routes};
use opsis_render::protocol::{COMPOSITION_ID, RenderTarget, SidecarCall, StillJob};
use opsis_render::sidecar::{Sidecar, SidecarCommand};
use opsis_types::timeline::{SpecAssets, TimelineSpec};
use opsis_types::values::{CompileValue, PortValues};
use opsis_types::workflow::{Workflow, WorkflowSettings};
use serde_json::{Value, json};
use tokio_util::sync::CancellationToken;

const STILLS: &str = "prepare/stills";
const FRAME: u32 = 45;
const CHANNEL_SLACK: u8 = 24;
const MOSTLY_SAME: f64 = 0.995;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn sidecar_program() -> PathBuf {
    std::env::var_os("OPSIS_SIDECAR").map_or_else(|| PathBuf::from("bun"), PathBuf::from)
}

fn sidecar_args() -> Vec<OsString> {
    if std::env::var_os("OPSIS_SIDECAR").is_some() {
        Vec::new()
    } else {
        ["run", "apps/sidecar/src/main.ts"]
            .map(OsString::from)
            .to_vec()
    }
}

fn home() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).expect("HOME")
}

fn cases() -> Vec<(&'static str, &'static str, Value)> {
    vec![
        (
            "text-inter",
            "core/text-3d",
            json!({ "text": "Opsis 3D", "fontFamily": "Inter", "fontWeight": 700 }),
        ),
        (
            "text-serif-italic",
            "core/text-3d",
            json!({ "text": "Quiet craft", "fontFamily": "Instrument Serif", "fontWeight": 400, "italic": true }),
        ),
        (
            "text-mono-italic",
            "core/text-3d",
            json!({ "text": "fn main()", "fontFamily": "JetBrains Mono", "fontWeight": 700, "italic": true }),
        ),
        ("text-heading", "core/text-3d", json!({})),
        (
            "model-plain",
            "core/model-3d",
            json!({ "src": "project:assets/models/plain.glb" }),
        ),
        (
            "model-external",
            "core/model-3d",
            json!({ "src": "project:assets/models/external/external.gltf" }),
        ),
        (
            "model-draco",
            "core/model-3d",
            json!({ "src": "project:assets/models/draco.glb" }),
        ),
        (
            "model-meshopt",
            "core/model-3d",
            json!({ "src": "project:assets/models/meshopt.glb" }),
        ),
        ("logo-default", "core/logo-3d", json!({})),
        (
            "logo-svg",
            "core/logo-3d",
            json!({ "src": "project:assets/models/logo.svg", "motion": "still" }),
        ),
        (
            "logo-default-still",
            "core/logo-3d",
            json!({ "src": "project:assets/models/logo-mark.svg", "motion": "still" }),
        ),
        (
            "logo-image",
            "core/logo-3d",
            json!({ "src": "project:assets/models/external/checker.png" }),
        ),
        ("card-default", "core/card-3d", json!({})),
    ]
}

fn spec_for(key: &str, params: &Value) -> TimelineSpec {
    let kind = element_node_type(key);
    let def = node_def(&kind).unwrap();
    let mut merged = def.defaults();
    if let Some(from) = params.as_object() {
        merged.extend(from.clone());
    }
    let values = def.parse(&merged, None).unwrap();
    let workflow: Workflow = serde_json::from_value(
        json!({ "version": 1, "id": "wf", "name": "Test", "nodes": [], "edges": [] }),
    )
    .unwrap();
    let settings = WorkflowSettings::default();
    let (inputs, vars) = (PortValues::new(), Vars::new());
    let ctx = Ctx {
        node_id: "n",
        kind: &kind,
        label: None,
        params: (),
        values: &values,
        inputs: &inputs,
        settings: &settings,
        result: None,
        vars: &vars,
        workflow: &workflow,
    };
    match def
        .run_compile(&ctx)
        .unwrap()
        .unwrap()
        .shift_remove(CLIP_PORT)
    {
        Some(CompileValue::Clip(clip)) => clip_to_spec(&clip, &settings),
        other => panic!("no clip from {kind}: {other:?}"),
    }
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn share_alike(made: &Path, baseline: &Path) -> Result<(), String> {
    let (a, b) = (
        image::open(made).map_err(|e| e.to_string())?.to_rgba8(),
        image::open(baseline).map_err(|e| e.to_string())?.to_rgba8(),
    );
    if a.dimensions() != b.dimensions() {
        return Err(format!(
            "size {:?} against {:?}",
            a.dimensions(),
            b.dimensions()
        ));
    }
    let close = a
        .pixels()
        .zip(b.pixels())
        .filter(|(p, q)| {
            p.0.iter()
                .zip(q.0.iter())
                .all(|(x, y)| x.abs_diff(*y) <= CHANNEL_SLACK)
        })
        .count();
    let share = close as f64 / f64::from(a.width() * a.height());
    if share >= MOSTLY_SAME {
        Ok(())
    } else {
        Err(format!("only {:.2}% of pixels match", share * 100.0))
    }
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the dev engine and the video bundle; OPSIS_STILLS_OUT names where stills go"]
async fn draws_3d_elements_as_recorded() {
    let out = std::env::var_os("OPSIS_STILLS_OUT")
        .map_or_else(|| home().join(".cache/opsis-three-stills"), PathBuf::from);
    let temp = out.join("tmp");
    let project = temp.join("project");
    copy_dir(
        &opsis_golden::path("prepare/models"),
        &project.join("assets/models"),
    );
    let dirs = PreparedDirs {
        project: project.clone(),
        public: repo().join("public"),
    };
    let server = FileServer::start_with(&project, prepared_routes(dirs))
        .await
        .unwrap();
    let engine = engine_paths(
        &home().join(".config/Opsis/engine"),
        ShellPlatform::current().unwrap(),
    );
    let shell = find_headless_shell(&engine).unwrap().to_path_buf();
    let env = ["TMPDIR", "TEMP", "TMP"]
        .map(|k| (OsString::from(k), temp.clone().into_os_string()))
        .to_vec();
    let command = SidecarCommand {
        program: sidecar_program(),
        args: sidecar_args(),
        env,
        cwd: Some(repo()),
    };
    let sidecar = Sidecar::new(command, Arc::new(|line| eprintln!("sidecar: {line}")));
    let never = CancellationToken::new();
    let mut failures = Vec::new();
    for (name, key, params) in cases() {
        let spec = TimelineSpec {
            assets: Some(SpecAssets {
                project_base: Some(server.url.clone()),
                versions: None,
            }),
            ..spec_for(key, &params)
        };
        let file = out.join(format!("{name}.png"));
        let job = StillJob {
            target: RenderTarget {
                serve_url: repo()
                    .join("remotion-bundle")
                    .to_string_lossy()
                    .into_owned(),
                browser_executable: shell.to_string_lossy().into_owned(),
                binaries_directory: std::env::var("OPSIS_COMPOSITOR").ok(),
                license_key: "free-license".into(),
                composition: COMPOSITION_ID.into(),
                spec,
                gl: serde_json::from_value(json!("angle")).unwrap(),
            },
            frame: FRAME,
            output: file.to_string_lossy().into_owned(),
            format: serde_json::from_value(json!("png")).unwrap(),
            scale: Some(0.5),
            jpeg_quality: None,
        };
        if let Err(error) = sidecar
            .call(SidecarCall::RenderStill(job), &never, None)
            .await
        {
            failures.push(format!("{name}: {error}"));
            continue;
        }
        let recorded = opsis_golden::path(&format!("{STILLS}/{name}.png"));
        if opsis_golden::writing() {
            std::fs::copy(&file, &recorded).unwrap_or_else(|e| panic!("{name}: {e}"));
        } else if let Err(error) = share_alike(&file, &recorded) {
            failures.push(format!("{name}: {error}"));
        }
    }
    std::fs::remove_dir_all(&temp).ok();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
