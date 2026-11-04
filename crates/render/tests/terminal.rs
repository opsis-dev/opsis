use std::path::Path;
use std::sync::Arc;

use opsis_capture::film::{Capture, DriverOptions, FilmPage};
use opsis_capture::terminal_paint::TerminalPainter;
use opsis_compile::asset_ref::FILM_FOLDER;
use opsis_compile::clip::clip_to_spec;
use opsis_compile::footage::footage_uses;
use opsis_compile::hosted::hosted;
use opsis_compile::node::{CLIP_PORT, Ctx};
use opsis_compile::nodes::output::{Gl, StillFormat};
use opsis_compile::registry::node_def;
use opsis_compile::terminal::{default_look, grid, screen_size, terminal_fonts, theme_named};
use opsis_compile::vars::Vars;
use opsis_engine::hash::sha256_hex;
use opsis_engine::http_files::{FileServer, folder_routes};
use opsis_engine::tools::EngineFiles;
use opsis_footage::decoder::FootageTools;
use opsis_footage::footage::Footage;
use opsis_footage::serve::footage_routes;
use opsis_footage::terminal::TerminalFootage;
use opsis_golden::home;
use opsis_render::frames::{StillRender, render_still};
use opsis_terminal::cast::read_path;
use opsis_terminal::frames::write;
use opsis_terminal::prepare::{PrepareInput, prepare};
use opsis_terminal::timeline::Timeline;
use opsis_types::numbers;
use opsis_types::numbers::millis;
use opsis_types::timeline::{SpecAssets, TimelineSpec};
use opsis_types::values::{CompileValue, PortValues};
use opsis_types::workflow::{Workflow, WorkflowSettings};
use serde_json::{Map, Value, json};

const STILLS: &str = "terminal/stills.json";
const WHOLE_HASH: usize = 64;
const BACKGROUND: [u8; 3] = [0x17, 0x17, 0x17];
const LIT_DIFFERENCE: u8 = 64;
const MIN_LIT: f64 = 0.002;
const MAX_LIT: f64 = 0.5;

fn terminal_clip(length_ms: f64) -> opsis_types::values::ClipValue {
    let take = json!({
        "type": "recording",
        "log": { "viewport": { "width": 80, "height": 20 }, "dpr": 1, "t0": 0, "durationMs": length_ms, "actions": [] },
        "durationMs": length_ms, "nodeId": "rec", "edits": [],
        "source": { "kind": "terminal", "cast": "project:take.cast", "cols": 80, "rows": 20 }
    });
    let style = json!({ "type": "recording-style", "entries": [
        { "nodeId": "look", "type": "terminal.look", "params": { "theme": "Catppuccin Mocha", "fontSize": 20, "lineHeight": 1.2, "letterSpacing": 0, "padding": 24, "cursor": "program", "cursorBlink": true } }
    ] });
    let inputs: PortValues =
        serde_json::from_value(json!({ "recording": take, "style": style })).unwrap();
    let def = node_def("camera.take").unwrap();
    let values = def.parse(&def.defaults(), None).unwrap();
    let workflow: Workflow = serde_json::from_value(
        json!({ "version": 1, "id": "wf", "name": "Test", "nodes": [], "edges": [] }),
    )
    .unwrap();
    let settings = WorkflowSettings::default();
    let vars = Vars::new();
    let ctx = Ctx {
        node_id: "take",
        kind: "camera.take",
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
        other => panic!("no clip: {other:?}"),
    }
}

#[tokio::test]
#[ignore = "needs the dev engine and the film page (bun run film:bundle)"]
async fn draws_a_terminal_take_through_the_frame_driver_as_recorded() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let out = home().join(".cache/opsis-film-test/terminal");
    let project = out.join("project");
    std::fs::create_dir_all(&project).unwrap();
    let cast_file = opsis_golden::path("terminal/take.cast");
    std::fs::copy(&cast_file, project.join("take.cast")).unwrap();
    let cast = read_path(&cast_file).unwrap();
    let length = Timeline::from_events(&cast.events).length();
    let clip = terminal_clip(millis(length));
    let spec = clip_to_spec(&clip, &WorkflowSettings::default());
    let uses = footage_uses(&spec.tracks);
    let terminals = uses.iter().map(|u| {
        (
            u.id.clone(),
            TerminalFootage {
                cast: project.join("take.cast"),
                theme: u
                    .terminal
                    .clone()
                    .unwrap_or_else(|| theme_named("Default").unwrap()),
                masks: Vec::new(),
                fps: spec.fps,
                grade: None,
            },
        )
    });
    let footage = Footage::new(
        FootageTools {
            ffmpeg: opsis_golden::ffmpeg(),
            temp: out.clone(),
        },
        Vec::new(),
    )
    .with_terminals(terminals);
    let routes = footage_routes(Arc::new(footage))
        .merge(folder_routes(FILM_FOLDER, &repo.join("film-bundle")));
    let server = FileServer::start_with(&project, routes).await.unwrap();
    let spec = hosted(&TimelineSpec {
        assets: Some(SpecAssets {
            project_base: Some(server.url.clone()),
        }),
        ..spec
    });
    let json = serde_json::to_value(&spec).unwrap();
    let engine = EngineFiles::downloaded(&home().join(".config/Opsis/engine"));
    let mut hashes = Map::new();
    for (name, frame) in [
        ("middle.png", spec.duration_in_frames / 2),
        ("last.png", spec.duration_in_frames - 1),
    ] {
        let output = out.join(name);
        render_still(&StillRender {
            driver: DriverOptions {
                executable: engine.browser().unwrap(),
                temp_dir: &out,
                gl: Gl::Angle,
                capture: Capture::for_this_computer(),
            },
            page: FilmPage {
                url: &format!("{}{FILM_FOLDER}/index.html", server.url),
                spec: &json,
                width: spec.width,
                height: spec.height,
                scale: 1.0,
            },
            frame,
            format: StillFormat::Png,
            jpeg_quality: None,
            output: &output,
        })
        .await
        .unwrap_or_else(|e| panic!("{name}: {e}"));
        hashes.insert(
            name.to_owned(),
            Value::from(sha256_hex(&std::fs::read(&output).unwrap(), WHOLE_HASH)),
        );
    }
    opsis_golden::check(STILLS, &Value::Object(hashes));
}

fn lit_share(picture: &image::RgbImage, background: [u8; 3]) -> f64 {
    let lit = picture
        .pixels()
        .filter(|p| {
            p.0.iter()
                .zip(background)
                .map(|(a, b)| a.abs_diff(b))
                .max()
                .unwrap_or(0)
                > LIT_DIFFERENCE
        })
        .count();
    numbers::float(lit) / f64::from(picture.width() * picture.height())
}

#[tokio::test]
#[ignore = "needs the dev engine and the terminal page (bun run terminal:bundle)"]
async fn paints_a_live_terminal_frame_at_its_screen_size() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let out = home().join(".cache/opsis-film-test/painter");
    std::fs::create_dir_all(&out).unwrap();
    let cast = read_path(opsis_golden::path("terminal/take.cast")).unwrap();
    let theme = theme_named("Default").unwrap();
    let mut frames = prepare(&PrepareInput {
        cast: &cast,
        fps: 2,
        theme: &theme,
        masks: &[],
    })
    .unwrap();
    let last = frames.frames.pop().unwrap();
    frames.frames = vec![last];
    let engine = EngineFiles::downloaded(&home().join(".config/Opsis/engine"));
    let painter = TerminalPainter::open(
        engine.browser().unwrap(),
        &out,
        &repo.join("terminal-bundle"),
        &repo.join("public"),
        &terminal_fonts(),
    )
    .await
    .unwrap();
    let look = default_look(Some(theme));
    let result = painter.paint(&write(&frames).unwrap(), &look).await;
    painter.close().await;
    let (size, jpeg) = result.unwrap();
    let screen = screen_size(&grid(&look, frames.cols, frames.rows));
    assert_eq!((size.width, size.height), (screen.width, screen.height));
    let picture = image::load_from_memory(&jpeg).unwrap().to_rgb8();
    picture.save(out.join("painted.png")).unwrap();
    assert_eq!(picture.dimensions(), (screen.width, screen.height));
    let share = lit_share(&picture, BACKGROUND);
    assert!(
        (MIN_LIT..MAX_LIT).contains(&share),
        "{share} of the picture is text"
    );
}
