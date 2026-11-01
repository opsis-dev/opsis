use std::path::{Path, PathBuf};

use opsis_capture::film::{Capture, DriverOptions, FilmPage};
use opsis_compile::asset_ref::FILM_FOLDER;
use opsis_compile::clip::clip_to_spec;
use opsis_compile::compile::compile_workflow;
use opsis_compile::elements::overlay::captions::CAPTIONS_ELEMENT;
use opsis_compile::elements::{ElementKind, ElementMeta, element_node_type, elements};
use opsis_compile::hosted::hosted;
use opsis_compile::node::CLIP_PORT;
use opsis_compile::nodes::output::{Gl, StillFormat};
use opsis_compile::nodes::text::{CAPTIONS_FROM_SCRIPT_TYPE, CAPTIONS_PORT, SHOW_CAPTIONS_TYPE};
use opsis_engine::http_files::{FileServer, folder_routes};
use opsis_engine::paths::{ShellPlatform, engine_paths, find_headless_shell};
use opsis_prepare::serve::{PreparedDirs, prepared_routes};
use opsis_render::frames::{FilmError, StillRender, render_still};
use opsis_types::numbers::whole;
use opsis_types::timeline::{SpecAssets, TimelineSpec};
use opsis_types::values::CompileValue;
use opsis_types::workflow::{Workflow, WorkflowSettings};
use serde_json::{Value, json};

const STILL_WIDTH: f64 = 480.0;
const JPEG_QUALITY: f64 = 82.0;
const SOLID_FRAMES: u32 = 90;
const ELEMENT_NODE: &str = "element";

fn home() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .expect("a home folder")
}

fn user_data() -> PathBuf {
    if let Some(dir) = std::env::var_os("OPSIS_USER_DATA") {
        return PathBuf::from(dir);
    }
    let config = if cfg!(target_os = "macos") {
        home().join("Library/Application Support")
    } else if cfg!(windows) {
        std::env::var_os("APPDATA").map_or_else(|| home().join("AppData/Roaming"), PathBuf::from)
    } else {
        std::env::var_os("XDG_CONFIG_HOME").map_or_else(|| home().join(".config"), PathBuf::from)
    };
    config.join("Opsis")
}

fn node(id: &str, kind: &str, params: Value) -> Value {
    let mut node = json!({ "id": id, "type": kind, "position": { "x": 0, "y": 0 } });
    node["params"] = params;
    node
}

fn edge(source: &str, target: &str, target_handle: &str) -> Value {
    port_edge(source, CLIP_PORT, target, target_handle)
}

fn port_edge(source: &str, source_handle: &str, target: &str, target_handle: &str) -> Value {
    json!({ "id": format!("{source}->{target}.{target_handle}"), "source": source, "sourceHandle": source_handle, "target": target, "targetHandle": target_handle })
}

fn solid(id: &str, color: &str) -> Value {
    node(
        id,
        "fx.solid",
        json!({ "color": color, "clipDuration": SOLID_FRAMES }),
    )
}

struct Host {
    shell: PathBuf,
    temp: PathBuf,
    project_base: String,
}

fn workflow_for(meta: &ElementMeta) -> Workflow {
    let kind = if meta.kind == ElementKind::Presentation {
        format!("transition.{}", meta.key)
    } else if meta.key == CAPTIONS_ELEMENT {
        SHOW_CAPTIONS_TYPE.to_owned()
    } else {
        element_node_type(&meta.key)
    };
    let (inputs, links): (Vec<Value>, Vec<Value>) = match meta.kind {
        _ if meta.key == CAPTIONS_ELEMENT => (
            vec![node("script", CAPTIONS_FROM_SCRIPT_TYPE, json!({}))],
            vec![port_edge(
                "script",
                CAPTIONS_PORT,
                ELEMENT_NODE,
                CAPTIONS_PORT,
            )],
        ),
        ElementKind::Presentation => (
            vec![solid("a", "#334155"), solid("b", "#7c3aed")],
            vec![edge("a", ELEMENT_NODE, "a"), edge("b", ELEMENT_NODE, "b")],
        ),
        ElementKind::Wrapper => (
            vec![solid("in", "#334155")],
            vec![edge("in", ELEMENT_NODE, "clip")],
        ),
        ElementKind::Sequencer => (
            vec![solid("in1", "#334155"), solid("in2", "#7c3aed")],
            vec![
                edge("in1", ELEMENT_NODE, "in1"),
                edge("in2", ELEMENT_NODE, "in2"),
            ],
        ),
        ElementKind::Component => (Vec::new(), Vec::new()),
    };
    let mut nodes = inputs;
    nodes.push(node(ELEMENT_NODE, &kind, json!({})));
    serde_json::from_value(json!({ "version": 1, "id": "elements", "name": "Elements", "nodes": nodes, "edges": links, "markers": [] })).expect("a workflow")
}

#[derive(Debug, thiserror::Error)]
enum StillError {
    #[error("{0}")]
    Compile(String),
    #[error("no clip")]
    NoClip,
    #[error(transparent)]
    Render(#[from] FilmError),
}

async fn still_for(meta: &ElementMeta, at: f64, host: &Host, out: &Path) -> Result<(), StillError> {
    let settings = WorkflowSettings::default();
    let compiled = compile_workflow(&workflow_for(meta), None);
    if let Some(error) = compiled.errors.get(ELEMENT_NODE) {
        return Err(StillError::Compile(error.clone()));
    }
    let Some(CompileValue::Clip(clip)) = compiled
        .values
        .get(ELEMENT_NODE)
        .and_then(|v| v.get(CLIP_PORT))
    else {
        return Err(StillError::NoClip);
    };
    let spec = hosted(&TimelineSpec {
        assets: Some(SpecAssets {
            project_base: Some(host.project_base.clone()),
        }),
        ..clip_to_spec(clip, &settings)
    });
    let json = serde_json::to_value(&spec).expect("a spec as JSON");
    let url = format!("{}{FILM_FOLDER}/index.html", host.project_base);
    render_still(&StillRender {
        driver: DriverOptions {
            executable: &host.shell,
            temp_dir: &host.temp,
            gl: Gl::Angle,
            capture: Capture::for_this_computer(),
        },
        page: FilmPage {
            url: &url,
            spec: &json,
            width: spec.width,
            height: spec.height,
            scale: (STILL_WIDTH / f64::from(spec.width)).min(1.0),
        },
        frame: whole((f64::from(clip.duration_in_frames - 1) * at).floor()),
        format: StillFormat::Jpeg,
        jpeg_quality: Some(JPEG_QUALITY),
        output: &out.join(format!("{}.jpg", meta.key)),
    })
    .await?;
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut args = std::env::args().skip(1);
    let only = args.next().unwrap_or_default();
    let at: f64 = args.next().and_then(|a| a.parse().ok()).unwrap_or(0.5);
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let bundle = repo.join("film-bundle");
    assert!(
        bundle.join("index.html").exists(),
        "There's no film page. Run `bun run film:bundle` first."
    );
    let engine = engine_paths(
        &user_data().join("engine"),
        ShellPlatform::current().expect("a supported platform"),
    );
    let shell = find_headless_shell(&engine)
        .expect("No engine in the app data. Set it up from Settings → Engine first.")
        .to_path_buf();
    let out = home().join(".cache/opsis-element-stills");
    let temp = out.join("tmp");
    let project = temp.join("project");
    std::fs::create_dir_all(&project).expect("the output folder");
    let dirs = PreparedDirs {
        project: project.clone(),
        public: repo.join("public"),
    };
    let routes = prepared_routes(dirs).merge(folder_routes(FILM_FOLDER, &bundle));
    let server = FileServer::start_with(&project, routes)
        .await
        .expect("a file server for the film page and prepared files");
    let host = Host {
        shell,
        temp: temp.clone(),
        project_base: server.url.clone(),
    };
    let chosen: Vec<&ElementMeta> = elements()
        .iter()
        .filter(|m| m.key.contains(&only))
        .collect();
    let mut failures = Vec::new();
    for meta in &chosen {
        match still_for(meta, at, &host, &out).await {
            Ok(()) => println!("ok   {}", meta.key),
            Err(error) => {
                println!("FAIL {}", meta.key);
                failures.push(format!("{}: {error}", meta.key));
            }
        }
    }
    println!(
        "\n{} of {} rendered into {}",
        chosen.len() - failures.len(),
        chosen.len(),
        out.display()
    );
    if let Err(error) = std::fs::remove_dir_all(&temp) {
        eprintln!("couldn't remove {}: {error}", temp.display());
    }
    if !failures.is_empty() {
        eprintln!("{}", failures.join("\n"));
        std::process::exit(1);
    }
}
