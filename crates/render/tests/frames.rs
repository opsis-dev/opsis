use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use opsis_capture::film::{Capture, DriverOptions, FilmPage};
use opsis_compile::asset_ref::FILM_FOLDER;
use opsis_compile::clip::{ItemExtra, clip_to_spec, item_clip};
use opsis_compile::elements::{ElementKind, element_meta, element_node_type};
use opsis_compile::hosted::hosted;
use opsis_compile::node::{CLIP_PORT, Ctx};
use opsis_compile::nodes::media::Codec;
use opsis_compile::nodes::output::{Gl, StillFormat};
use opsis_compile::registry::node_def;
use opsis_compile::vars::Vars;
use opsis_engine::hash::sha256_hex;
use opsis_engine::http_files::{FileServer, folder_routes};
use opsis_engine::tools::EngineFiles;
use opsis_golden::home;
use opsis_media::codecs::FilmEncoding;
use opsis_media::ffmpeg::Ffmpeg;
use opsis_prepare::serve::{PreparedDirs, prepared_routes};
use opsis_render::frames::{
    FilmError, FilmRender, FilmRun, StillRender, render_film, render_still,
};
use opsis_types::timeline::{SpecAssets, TimelineSpec};
use opsis_types::values::{ClipValue, CompileValue, PortValues};
use opsis_types::workflow::{Workflow, WorkflowSettings};
use serde_json::{Map, Value, json};
use tokio_util::sync::CancellationToken;

const STILLS: [&str; 5] = [
    "kinetic-headline",
    "text-3d",
    "shapes-3d",
    "shader-grain-gradient",
    "grain-dissolve",
];
const VIDEOS: [&str; 2] = ["kinetic-headline", "shader-grain-gradient"];
const PNG_SIZE_AT: usize = 16;
const RENDERS: &str = "render/renders.json";
const WHOLE_HASH: usize = 64;

fn solid(id: &str, color: &str) -> CompileValue {
    let props = Map::from_iter([("color".to_owned(), json!(color))]);
    CompileValue::Clip(item_clip(id, "solid", props, 90, ItemExtra::default()))
}

fn compile_clip(kind: &str, inputs: &PortValues) -> ClipValue {
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
        inputs,
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

fn spec_for(key: &str) -> TimelineSpec {
    let meta = element_meta(key).unwrap();
    let (kind, inputs) = match meta.kind {
        ElementKind::Presentation => (
            format!("transition.{key}"),
            PortValues::from_iter([
                ("a".to_owned(), solid("a", "#334155")),
                ("b".to_owned(), solid("b", "#7c3aed")),
            ]),
        ),
        ElementKind::Wrapper => (
            element_node_type(key),
            PortValues::from_iter([("clip".to_owned(), solid("in", "#334155"))]),
        ),
        _ => (element_node_type(key), PortValues::new()),
    };
    clip_to_spec(&compile_clip(&kind, &inputs), &WorkflowSettings::default())
}

struct Setup {
    engine: EngineFiles,
    out: PathBuf,
    temp: PathBuf,
    server: FileServer,
}

impl Setup {
    async fn new() -> Self {
        let engine = EngineFiles::downloaded(&home().join(".config/Opsis/engine"));
        let out = home().join(".cache/opsis-film-test");
        let temp = out.join("tmp");
        std::fs::create_dir_all(&temp).unwrap();
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let bundle = repo.join("film-bundle");
        assert!(
            bundle.join("index.html").exists(),
            "build the film page first: bun run film:bundle"
        );
        let project = temp.join("project");
        std::fs::create_dir_all(&project).unwrap();
        let dirs = PreparedDirs {
            project: project.clone(),
            public: repo.join("public"),
        };
        let routes = prepared_routes(dirs).merge(folder_routes(FILM_FOLDER, &bundle));
        let server = FileServer::start_with(&project, routes).await.unwrap();
        Self {
            engine,
            out,
            temp,
            server,
        }
    }

    fn page_url(&self) -> String {
        format!("{}{FILM_FOLDER}/index.html", self.server.url)
    }

    fn spec(&self, key: &str) -> (Value, TimelineSpec) {
        let spec = hosted(&TimelineSpec {
            assets: Some(SpecAssets {
                project_base: Some(self.server.url.clone()),
            }),
            ..spec_for(key)
        });
        (serde_json::to_value(&spec).unwrap(), spec)
    }

    fn driver(&self) -> DriverOptions<'_> {
        self.driver_with(Capture::for_this_computer())
    }

    fn driver_with(&self, capture: Capture) -> DriverOptions<'_> {
        DriverOptions {
            executable: self.engine.browser().unwrap(),
            temp_dir: &self.temp,
            gl: Gl::Angle,
            capture,
        }
    }
}

fn h264() -> FilmEncoding {
    FilmEncoding {
        codec: Codec::H264,
        fps: 0.0,
        png_frames: false,
        crf: Some(18.0),
        x264_preset: None,
        pixel_format: None,
        prores: None,
        gif_loops: None,
    }
}

fn png_size(bytes: &[u8]) -> (u32, u32) {
    let word = |at: usize| u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap());
    (word(PNG_SIZE_AT), word(PNG_SIZE_AT + 4))
}

#[tokio::test]
#[ignore = "needs the dev engine and the film page (bun run film:bundle)"]
async fn draws_each_element_still_at_the_film_size() {
    let setup = Setup::new().await;
    for key in STILLS {
        let (json, spec) = setup.spec(key);
        let output = setup.out.join(format!("{key}.png"));
        render_still(&StillRender {
            driver: setup.driver(),
            page: FilmPage {
                url: &setup.page_url(),
                spec: &json,
                width: spec.width,
                height: spec.height,
                scale: 1.0,
            },
            frame: spec.duration_in_frames / 2,
            format: StillFormat::Png,
            jpeg_quality: None,
            output: &output,
        })
        .await
        .unwrap_or_else(|e| panic!("{key}: {e}"));
        let bytes = std::fs::read(&output).unwrap();
        assert_eq!(png_size(&bytes), (spec.width, spec.height), "{key}");
    }
}

#[tokio::test]
#[ignore = "needs the dev engine and the film page (bun run film:bundle)"]
async fn draws_the_same_still_with_a_screenshot_per_frame() {
    let setup = Setup::new().await;
    let (json, spec) = setup.spec("kinetic-headline");
    let mut drawn = Vec::new();
    for capture in [Capture::BeginFrame, Capture::Screenshot] {
        let output = setup.out.join(format!("{capture:?}.png"));
        render_still(&StillRender {
            driver: setup.driver_with(capture),
            page: FilmPage {
                url: &setup.page_url(),
                spec: &json,
                width: spec.width,
                height: spec.height,
                scale: 1.0,
            },
            frame: spec.duration_in_frames / 2,
            format: StillFormat::Png,
            jpeg_quality: None,
            output: &output,
        })
        .await
        .unwrap_or_else(|e| panic!("{capture:?}: {e}"));
        drawn.push(std::fs::read(&output).unwrap());
    }
    assert_eq!(png_size(&drawn[1]), (spec.width, spec.height));
    assert_eq!(drawn[0], drawn[1]);
}

#[tokio::test]
#[ignore = "needs the dev engine and the film page (bun run film:bundle)"]
async fn encodes_every_frame_of_a_film_and_reports_progress_to_the_end() {
    let setup = Setup::new().await;
    let tools = Ffmpeg::new(&setup.engine);
    let never = CancellationToken::new();
    for key in VIDEOS {
        let (json, spec) = setup.spec(key);
        let output = setup.out.join(format!("{key}.mp4"));
        let progressed = Mutex::new(0.0);
        let notes = Mutex::new(Vec::new());
        render_film(
            &FilmRender {
                driver: setup.driver(),
                page: FilmPage {
                    url: &setup.page_url(),
                    spec: &json,
                    width: spec.width,
                    height: spec.height,
                    scale: 1.0,
                },
                encoding: h264(),
                every_nth_frame: 1,
                output: &output,
            },
            &FilmRun {
                tools: &tools,
                cancel: &never,
                progress: &|p| *progressed.lock().unwrap() = p,
                note: &|n| notes.lock().unwrap().push(n),
            },
        )
        .await
        .unwrap_or_else(|e| panic!("{key}: {e}"));
        assert_eq!(*progressed.lock().unwrap(), 1.0, "{key}");
        eprintln!("{key}: {:?}", notes.lock().unwrap());
        let probed = opsis_probe::probe(&output).unwrap();
        let video = probed.video.expect("a picture");
        assert_eq!((video.width, video.height), (spec.width, spec.height));
        let expected_ms = f64::from(spec.duration_in_frames) * 1000.0 / f64::from(spec.fps);
        let duration = f64::from(probed.duration_ms.unwrap());
        assert!(
            (duration - expected_ms).abs() <= 1000.0 / f64::from(spec.fps),
            "{key}: {duration} ms against {expected_ms} ms"
        );
    }
}

#[tokio::test]
#[ignore = "needs the dev engine and the film page (bun run film:bundle)"]
async fn stops_a_render_soon_after_it_is_cancelled() {
    let setup = Setup::new().await;
    let tools = Ffmpeg::new(&setup.engine);
    let cancel = CancellationToken::new();
    let stop = cancel.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(1500)).await;
        stop.cancel();
    });
    let (json, spec) = setup.spec("text-3d");
    let output = setup.out.join("cancelled.mp4");
    let started = Instant::now();
    let rendered = render_film(
        &FilmRender {
            driver: setup.driver(),
            page: FilmPage {
                url: &setup.page_url(),
                spec: &json,
                width: spec.width,
                height: spec.height,
                scale: 1.0,
            },
            encoding: h264(),
            every_nth_frame: 1,
            output: &output,
        },
        &FilmRun {
            tools: &tools,
            cancel: &cancel,
            progress: &|_| {},
            note: &|_| {},
        },
    )
    .await;
    assert!(
        matches!(rendered, Err(FilmError::Cancelled)),
        "{rendered:?}"
    );
    assert!(started.elapsed() < Duration::from_secs(10));
}

#[tokio::test]
#[ignore = "needs the dev engine and the film page (bun run film:bundle)"]
async fn renders_the_same_bytes_as_recorded() {
    let setup = Setup::new().await;
    let tools = Ffmpeg::new(&setup.engine);
    let never = CancellationToken::new();
    let mut hashes = Map::new();
    for key in STILLS {
        let (json, spec) = setup.spec(key);
        let file = format!("{key}.png");
        render_still(&StillRender {
            driver: setup.driver(),
            page: FilmPage {
                url: &setup.page_url(),
                spec: &json,
                width: spec.width,
                height: spec.height,
                scale: 1.0,
            },
            frame: spec.duration_in_frames / 2,
            format: StillFormat::Png,
            jpeg_quality: None,
            output: &setup.out.join(&file),
        })
        .await
        .unwrap_or_else(|e| panic!("{key}: {e}"));
        let made = std::fs::read(setup.out.join(&file)).unwrap();
        hashes.insert(file, Value::from(sha256_hex(&made, WHOLE_HASH)));
    }
    for key in VIDEOS {
        let (json, spec) = setup.spec(key);
        let file = format!("{key}.mp4");
        render_film(
            &FilmRender {
                driver: setup.driver(),
                page: FilmPage {
                    url: &setup.page_url(),
                    spec: &json,
                    width: spec.width,
                    height: spec.height,
                    scale: 1.0,
                },
                encoding: h264(),
                every_nth_frame: 1,
                output: &setup.out.join(&file),
            },
            &FilmRun {
                tools: &tools,
                cancel: &never,
                progress: &|_| {},
                note: &|_| {},
            },
        )
        .await
        .unwrap_or_else(|e| panic!("{key}: {e}"));
        let made = std::fs::read(setup.out.join(&file)).unwrap();
        hashes.insert(file, Value::from(sha256_hex(&made, WHOLE_HASH)));
    }
    opsis_golden::check(RENDERS, &Value::Object(hashes));
}
