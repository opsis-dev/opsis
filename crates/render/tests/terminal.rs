use std::path::Path;
use std::sync::Arc;

use opsis_compile::clip::clip_to_spec;
use opsis_compile::footage::footage_uses;
use opsis_compile::hosted::hosted;
use opsis_compile::node::{CLIP_PORT, Ctx};
use opsis_compile::nodes::output::StillFormat;
use opsis_compile::registry::node_def;
use opsis_compile::terminal::{default_look, grid, screen_size, terminal_fonts, theme_named};
use opsis_compile::vars::Vars;
use opsis_film::Drawing;
use opsis_footage::decoder::FootageTools;
use opsis_footage::footage::Footage;
use opsis_footage::terminal::TerminalFootage;
use opsis_golden::pictures::{Picture, check_picture};
use opsis_golden::{dev_cache, repo};
use opsis_prepare::serve::PreparedDirs;
use opsis_render::native::{NATIVE_BASE, NativeFiles, NativeStill, render_native_still};
use opsis_terminal::cast::read_path;
use opsis_terminal::prepare::{PrepareInput, prepare};
use opsis_terminal::timeline::Timeline;
use opsis_types::numbers;
use opsis_types::numbers::millis;
use opsis_types::timeline::{SpecAssets, TimelineSpec};
use opsis_types::values::{CompileValue, PortValues};
use opsis_types::workflow::{Workflow, WorkflowSettings};
use serde_json::json;

const STILLS: &str = "terminal/stills";
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
async fn draws_a_terminal_take_as_recorded() {
    let out = dev_cache("film-test/terminal");
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
    let files = Arc::new(NativeFiles {
        dirs: PreparedDirs {
            project: project.clone(),
            public: repo().join("public"),
        },
        footage: Arc::new(footage),
        drawing: Drawing::default(),
    });
    let spec = hosted(&TimelineSpec {
        assets: Some(SpecAssets {
            project_base: Some(NATIVE_BASE.to_owned()),
        }),
        ..spec
    });
    let mut failures = Vec::new();
    for (name, frame) in [
        ("middle.png", spec.duration_in_frames / 2),
        ("last.png", spec.duration_in_frames - 1),
    ] {
        let output = out.join(name);
        render_native_still(NativeStill {
            spec: spec.clone(),
            files: Arc::clone(&files),
            frame,
            scale: 1.0,
            format: StillFormat::Png,
            jpeg_quality: None,
            output: output.clone(),
        })
        .await
        .unwrap_or_else(|e| panic!("{name}: {e}"));
        let picture = image::open(&output).unwrap().to_rgb8();
        let share = lit_share(&picture, BACKGROUND);
        assert!(
            (MIN_LIT..MAX_LIT).contains(&share),
            "{name}: {share} of the picture is text"
        );
        let png = std::fs::read(&output).unwrap();
        let rgba = image::load_from_memory(&png).unwrap().to_rgba8();
        failures.extend(check_picture(
            &format!("{STILLS}/{name}"),
            Picture {
                png: &png,
                rgba: rgba.as_raw(),
                size: rgba.dimensions(),
            },
            0.0,
            &out,
        ));
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
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

#[test]
fn paints_a_live_terminal_frame_natively_at_its_screen_size() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let out = dev_cache("film-test/painter");
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
    let public = repo.join("public");
    let files = |source: &str| {
        let rest = source.strip_prefix("app:")?;
        std::fs::read(public.join(rest)).ok()
    };
    let mut painter = opsis_film::TerminalPainter::new(&files, &terminal_fonts()).unwrap();
    let look = default_look(Some(theme));
    let surface = painter.paint(&frames, &look).unwrap();
    let screen = screen_size(&grid(&look, frames.cols, frames.rows));
    assert_eq!(
        (surface.width(), surface.height()),
        (screen.width, screen.height)
    );
    let rgba = surface.rgba();
    let picture = image::RgbImage::from_fn(surface.width(), surface.height(), |x, y| {
        let at = usize::try_from((y * surface.width() + x) * 4).unwrap();
        image::Rgb([rgba[at], rgba[at + 1], rgba[at + 2]])
    });
    picture.save(out.join("painted-natively.png")).unwrap();
    let share = lit_share(&picture, BACKGROUND);
    assert!(
        (MIN_LIT..MAX_LIT).contains(&share),
        "{share} of the picture is text"
    );
}
