use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use opsis_compile::clip::{ItemExtra, clip_to_spec, item_clip};
use opsis_compile::elements::{ElementKind, element_meta, element_node_type};
use opsis_compile::node::{CLIP_PORT, Ctx};
use opsis_compile::registry::node_def;
use opsis_compile::vars::Vars;
use opsis_engine::hash::sha256_hex;
use opsis_engine::http_files::FileServer;
use opsis_engine::paths::{ShellPlatform, engine_paths, find_headless_shell};
use opsis_prepare::serve::{PreparedDirs, prepared_routes};
use opsis_render::protocol::{COMPOSITION_ID, MediaJob, RenderTarget, SidecarCall, StillJob};
use opsis_render::sidecar::{Sidecar, SidecarCommand, SidecarError};
use opsis_types::timeline::{SpecAssets, TimelineSpec};
use opsis_types::values::{ClipValue, CompileValue, PortValues};
use opsis_types::workflow::{Workflow, WorkflowSettings};
use serde_json::{Map, Value, json};
use tokio_util::sync::CancellationToken;

const STILLS: [&str; 5] = [
    "core/kinetic-headline",
    "core/text-3d",
    "core/shapes-3d",
    "remocn/shader-grain-gradient",
    "remocn/grain-dissolve",
];
const VIDEOS: [&str; 2] = ["core/kinetic-headline", "remocn/shader-grain-gradient"];
const RENDERS: &str = "render/renders.json";
const WHOLE_HASH: usize = 64;

fn from_env(name: &str) -> PathBuf {
    std::env::var_os(name)
        .map(PathBuf::from)
        .unwrap_or_else(|| panic!("{name} isn't set"))
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

fn existing(path: PathBuf, what: &str) -> PathBuf {
    assert!(path.exists(), "{what} is missing: {}", path.display());
    path
}

fn solid(id: &str, color: &str) -> CompileValue {
    let props = Map::from_iter([("color".to_owned(), json!(color))]);
    CompileValue::Clip(item_clip(
        id,
        "core/solid",
        props,
        90.0,
        ItemExtra::default(),
    ))
}

fn compile_clip(kind: &str, inputs: PortValues) -> ClipValue {
    let def = node_def(kind).unwrap();
    let values = def.parse(&def.defaults(), None).unwrap();
    let workflow: Workflow = serde_json::from_value(
        json!({ "version": 1, "id": "wf", "name": "Test", "nodes": [], "edges": [] }),
    )
    .unwrap();
    let settings = WorkflowSettings::default();
    let vars = Vars::new();
    let ctx = Ctx {
        node_id: "n",
        kind,
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
        Some(CompileValue::Clip(clip)) => clip,
        other => panic!("no clip from {kind}: {other:?}"),
    }
}

fn spec_for(key: &str) -> (TimelineSpec, f64) {
    let meta = element_meta(key).unwrap();
    let (kind, inputs) = if meta.kind == ElementKind::Presentation {
        (
            format!("transition.{}", key.replacen('/', ".", 1)),
            PortValues::from_iter([
                ("a".to_owned(), solid("a", "#334155")),
                ("b".to_owned(), solid("b", "#7c3aed")),
            ]),
        )
    } else {
        (element_node_type(key), PortValues::new())
    };
    let clip = compile_clip(&kind, inputs);
    (
        clip_to_spec(&clip, &WorkflowSettings::default()),
        clip.duration_in_frames,
    )
}

struct Setup {
    sidecar: Sidecar,
    shell: PathBuf,
    bundle: PathBuf,
    out: PathBuf,
    assets: FileServer,
}

impl Setup {
    async fn new() -> Self {
        let cache = home().join(".cache");
        let engine = engine_paths(
            &home().join(".config/Opsis/engine"),
            ShellPlatform::current().unwrap(),
        );
        let shell = find_headless_shell(&engine)
            .expect("the dev engine's headless shell")
            .to_path_buf();
        let out = cache.join("opsis-render-test");
        let temp = out.join("tmp");
        std::fs::create_dir_all(&temp).unwrap();
        let env = ["TMPDIR", "TEMP", "TMP"]
            .map(|k| (OsString::from(k), temp.clone().into_os_string()))
            .to_vec();
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let project = temp.join("project");
        std::fs::create_dir_all(&project).unwrap();
        let dirs = PreparedDirs {
            project: project.clone(),
            public: repo.join("public"),
        };
        let assets = FileServer::start_with(&project, prepared_routes(dirs))
            .await
            .unwrap();
        let command = SidecarCommand {
            program: sidecar_program(),
            args: sidecar_args(),
            env,
            cwd: Some(repo),
        };
        Self {
            sidecar: Sidecar::new(command, Arc::new(|line| eprintln!("sidecar: {line}"))),
            shell,
            bundle: existing(from_env("OPSIS_VIDEO_BUNDLE"), "the video bundle"),
            out,
            assets,
        }
    }

    fn target(&self, spec: TimelineSpec) -> RenderTarget {
        RenderTarget {
            serve_url: self.bundle.to_string_lossy().into_owned(),
            browser_executable: self.shell.to_string_lossy().into_owned(),
            binaries_directory: std::env::var("OPSIS_COMPOSITOR").ok(),
            license_key: "free-license".into(),
            composition: COMPOSITION_ID.into(),
            spec: TimelineSpec {
                assets: Some(SpecAssets {
                    project_base: Some(self.assets.url.clone()),
                    versions: None,
                }),
                ..spec
            },
            gl: serde_json::from_value(json!("angle")).unwrap(),
        }
    }

    fn media(&self, key: &str, output: &Path) -> SidecarCall {
        SidecarCall::RenderMedia(MediaJob {
            target: self.target(spec_for(key).0),
            output: output.to_string_lossy().into_owned(),
            codec: serde_json::from_value(json!("h264")).unwrap(),
            crf: Some(18.0),
            scale: 1.0,
            muted: true,
        })
    }

    fn hash(&self, file: &str) -> Value {
        let made = std::fs::read(self.out.join(file)).unwrap_or_else(|e| panic!("{file}: {e}"));
        Value::from(sha256_hex(&made, WHOLE_HASH))
    }
}

fn file_name(key: &str, ext: &str) -> String {
    format!("{}.{ext}", key.replacen('/', "--", 1))
}

#[tokio::test]
#[ignore = "needs the dev engine and a video bundle named by OPSIS_VIDEO_BUNDLE"]
async fn renders_the_same_bytes_as_recorded() {
    let setup = Setup::new().await;
    let never = CancellationToken::new();
    let mut hashes = Map::new();
    for key in STILLS {
        let (spec, frames) = spec_for(key);
        let file = file_name(key, "png");
        let job = StillJob {
            target: setup.target(spec),
            frame: (frames / 2.0).floor(),
            output: setup.out.join(&file).to_string_lossy().into_owned(),
            format: serde_json::from_value(json!("png")).unwrap(),
            scale: None,
            jpeg_quality: None,
        };
        setup
            .sidecar
            .call(SidecarCall::RenderStill(job), &never, None)
            .await
            .unwrap();
        hashes.insert(file.clone(), setup.hash(&file));
    }
    for key in VIDEOS {
        let file = file_name(key, "mp4");
        let progressed = std::sync::Mutex::new(0.0);
        let report = |p: f64| *progressed.lock().unwrap() = p;
        setup
            .sidecar
            .call(
                setup.media(key, &setup.out.join(&file)),
                &never,
                Some(&report),
            )
            .await
            .unwrap();
        assert_eq!(*progressed.lock().unwrap(), 1.0);
        hashes.insert(file.clone(), setup.hash(&file));
    }
    opsis_golden::check(RENDERS, &Value::Object(hashes));
}

#[tokio::test]
#[ignore = "needs the dev engine and a video bundle named by OPSIS_VIDEO_BUNDLE"]
async fn stops_a_render_soon_after_it_is_cancelled() {
    let setup = Setup::new().await;
    let cancel = CancellationToken::new();
    let stop = cancel.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(1500)).await;
        stop.cancel();
    });
    let started = Instant::now();
    let rendered = setup
        .sidecar
        .call(
            setup.media("core/text-3d", &setup.out.join("cancelled.mp4")),
            &cancel,
            None,
        )
        .await;
    assert!(
        matches!(rendered, Err(SidecarError::Cancelled)),
        "{rendered:?}"
    );
    assert!(
        started.elapsed() < Duration::from_secs(4),
        "took {:?}",
        started.elapsed()
    );
    setup.sidecar.ping().await.unwrap();
}
