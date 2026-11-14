use std::path::{Path, PathBuf};
use std::sync::Arc;

use opsis_compile::clip::{clip_to_spec, map_items};
use opsis_compile::footage::footage_uses;
use opsis_compile::hosted::hosted;
use opsis_compile::node::{CLIP_PORT, Ctx};
use opsis_compile::nodes::output::StillFormat;
use opsis_compile::nodes::terminal::with_look;
use opsis_compile::registry::node_def;
use opsis_compile::vars::Vars;
use opsis_film::Drawing;
use opsis_footage::decoder::FootageTools;
use opsis_footage::footage::Footage;
use opsis_footage::terminal::TerminalFootage;
use opsis_golden::pictures::{Picture, check_picture};
use opsis_golden::{dev_cache, repo};
use opsis_prepare::serve::PreparedDirs;
use opsis_render::native::{NATIVE_BASE, NativeFiles, NativeStill, render_native_still};
use opsis_types::terminal::{TerminalCursor, TerminalLook, TerminalTheme};
use opsis_types::timeline::{SpecAssets, TimelineItem, TimelineSpec};
use opsis_types::values::{ClipValue, CompileValue, PortValues};
use opsis_types::workflow::{Workflow, WorkflowSettings};
use serde_json::json;

const GRID: u32 = 16;
const SPREAD: u32 = 2;
const SIDE: u32 = GRID * SPREAD;
const WHITE: &str = "#ffffff";
const BLACK: &str = "#000000";

const DRAWN: [(u32, u32); 19] = [
    (0x2500, 0x259f),
    (0x25e2, 0x25e5),
    (0x25f8, 0x25fa),
    (0x25ff, 0x25ff),
    (0x2800, 0x28ff),
    (0xe0b0, 0xe0bf),
    (0xe0d2, 0xe0d2),
    (0xe0d4, 0xe0d4),
    (0xf5d0, 0xf60d),
    (0x1fb00, 0x1fbaf),
    (0x1fbbd, 0x1fbbf),
    (0x1fbce, 0x1fbef),
    (0x1cc1b, 0x1cc1e),
    (0x1cc21, 0x1cc3f),
    (0x1cd00, 0x1cde5),
    (0x1ce00, 0x1ce01),
    (0x1ce0b, 0x1ce0c),
    (0x1ce16, 0x1ce19),
    (0x1ce51, 0x1ceaf),
];

const GRIDS: [u32; 9] = [
    0x2500, 0x2800, 0xe000, 0xf500, 0xf600, 0x1cc00, 0x1cd00, 0x1ce00, 0x1fb00,
];

struct Cell {
    width: u32,
    height: u32,
    font_size: u32,
}

const CELLS: [Cell; 4] = [
    Cell {
        width: 18,
        height: 36,
        font_size: 65,
    },
    Cell {
        width: 12,
        height: 24,
        font_size: 45,
    },
    Cell {
        width: 11,
        height: 21,
        font_size: 25,
    },
    Cell {
        width: 9,
        height: 17,
        font_size: 15,
    },
];

fn drawn(cp: u32) -> bool {
    DRAWN.iter().any(|(a, b)| (*a..=*b).contains(&cp))
}

fn grid_cast(base: u32) -> String {
    let rows: Vec<String> = (0..SIDE)
        .map(|y| {
            (0..SIDE)
                .map(|x| {
                    let cp = base + (y / SPREAD) * GRID + x / SPREAD;
                    if x % SPREAD == 0 && y % SPREAD == 0 && drawn(cp) {
                        char::from_u32(cp).unwrap_or(' ')
                    } else {
                        ' '
                    }
                })
                .collect()
        })
        .collect();
    let screen = format!("\u{1b}[?25l{}", rows.join("\r\n"));
    let header = json!({ "version": 3, "term": { "cols": SIDE, "rows": SIDE } });
    format!(
        "{header}\n{}\n{}\n",
        json!([0.0, "o", screen]),
        json!([1.0, "o", ""])
    )
}

fn plain_theme() -> TerminalTheme {
    TerminalTheme {
        name: "Plain".to_owned(),
        fg: WHITE.to_owned(),
        bg: BLACK.to_owned(),
        cursor: WHITE.to_owned(),
        palette: std::array::from_fn(|_| WHITE.to_owned()),
    }
}

fn grid_clip(cell: &Cell) -> ClipValue {
    let take = json!({
        "type": "recording",
        "log": { "viewport": { "width": SIDE, "height": SIDE }, "dpr": 1, "t0": 0, "durationMs": 1000, "actions": [] },
        "durationMs": 1000, "nodeId": "rec", "edits": [],
        "source": { "kind": "terminal", "cast": "project:grid.cast", "cols": SIDE, "rows": SIDE }
    });
    let style = json!({ "type": "recording-style", "entries": [
        { "nodeId": "camera", "type": "camera.camera", "params": { "zoom": false } },
        { "nodeId": "shot", "type": "camera.frame", "params": { "windowStyle": "none", "windowSize": 100, "cornerRadius": 0, "shadow": 0, "liftColours": false } },
        { "nodeId": "blur", "type": "camera.motionBlur", "params": { "enabled": false } }
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
    let Some(CompileValue::Clip(clip)) = def
        .run_compile(&ctx)
        .unwrap()
        .unwrap()
        .shift_remove(CLIP_PORT)
    else {
        panic!("no clip");
    };
    let height = f64::from(cell.height) / f64::from(cell.font_size);
    let spacing = f64::from(cell.font_size).mul_add(-0.6, f64::from(cell.width));
    ClipValue {
        tracks: map_items(&clip.tracks, &|item| match item {
            TimelineItem::Recording { base, props } => {
                let look = TerminalLook {
                    theme: plain_theme(),
                    font_size: cell.font_size,
                    line_height: height,
                    letter_spacing: spacing,
                    padding: 0,
                    cursor: TerminalCursor::Program,
                    cursor_blink: false,
                };
                TimelineItem::Recording {
                    base,
                    props: Box::new(with_look(*props, |_| look)),
                }
            }
            other => other,
        }),
        ..clip
    }
}

async fn render_grid(cell: &Cell, base: u32, out: &Path) -> PathBuf {
    let project = out.join(format!("{base:X}-{}x{}", cell.width, cell.height));
    std::fs::create_dir_all(&project).unwrap();
    std::fs::write(project.join("grid.cast"), grid_cast(base)).unwrap();
    let settings = WorkflowSettings {
        width: SIDE * cell.width,
        height: SIDE * cell.height,
        ..WorkflowSettings::default()
    };
    let spec = clip_to_spec(&grid_clip(cell), &settings);
    let terminals = footage_uses(&spec.tracks).into_iter().map(|u| {
        (
            u.id,
            TerminalFootage {
                cast: project.join("grid.cast"),
                theme: plain_theme(),
                masks: Vec::new(),
                fps: spec.fps,
                grade: None,
            },
        )
    });
    let footage = Footage::new(
        FootageTools {
            ffmpeg: opsis_golden::ffmpeg(),
            temp: out.to_path_buf(),
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
    let output = project.join("ours.png");
    render_native_still(NativeStill {
        frame: spec.duration_in_frames - 1,
        spec,
        files,
        scale: 1.0,
        format: StillFormat::Png,
        jpeg_quality: None,
        output: output.clone(),
    })
    .await
    .unwrap_or_else(|e| panic!("{base:X}: {e}"));
    output
}

#[tokio::test]
async fn draws_every_sprite_as_recorded() {
    let out = dev_cache("film-test/sprites");
    std::fs::create_dir_all(&out).unwrap();
    let mut failures = Vec::new();
    for cell in &CELLS {
        for base in GRIDS {
            let file = render_grid(cell, base, &out).await;
            let picture = image::open(&file).unwrap().to_rgba8();
            let png = std::fs::read(&file).unwrap();
            if let Some(failed) = check_picture(
                &format!("sprites/{base:X}-{}x{}.png", cell.width, cell.height),
                Picture {
                    png: &png,
                    rgba: picture.as_raw(),
                    size: picture.dimensions(),
                },
                0.0,
                &out,
            ) {
                failures.push(failed);
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
