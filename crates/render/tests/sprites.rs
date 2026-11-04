use std::path::{Path, PathBuf};
use std::sync::Arc;

use image::GrayImage;
use opsis_capture::film::{Capture, DriverOptions, FilmPage};
use opsis_compile::asset_ref::FILM_FOLDER;
use opsis_compile::clip::{clip_to_spec, map_items};
use opsis_compile::footage::footage_uses;
use opsis_compile::hosted::hosted;
use opsis_compile::node::{CLIP_PORT, Ctx};
use opsis_compile::nodes::output::{Gl, StillFormat};
use opsis_compile::nodes::terminal::with_look;
use opsis_compile::registry::node_def;
use opsis_compile::vars::Vars;
use opsis_engine::http_files::{FileServer, folder_routes};
use opsis_engine::tools::EngineFiles;
use opsis_footage::decoder::FootageTools;
use opsis_footage::footage::Footage;
use opsis_footage::serve::footage_routes;
use opsis_footage::terminal::TerminalFootage;
use opsis_golden::home;
use opsis_render::frames::{StillRender, render_still};
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
const LIT: u8 = 128;
const MOST_ALIKE: f64 = 0.85;
const SMALL_GLYPH: usize = 8;

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

const ATLASES: [u32; 9] = [
    0x2500, 0x2800, 0xe000, 0xf500, 0xf600, 0x1cc00, 0x1cd00, 0x1ce00, 0x1fb00,
];

struct Cell {
    width: u32,
    height: u32,
    thickness: u32,
    font_size: u32,
}

const CELLS: [Cell; 4] = [
    Cell {
        width: 18,
        height: 36,
        thickness: 4,
        font_size: 65,
    },
    Cell {
        width: 12,
        height: 24,
        thickness: 3,
        font_size: 45,
    },
    Cell {
        width: 11,
        height: 21,
        thickness: 2,
        font_size: 25,
    },
    Cell {
        width: 9,
        height: 17,
        thickness: 1,
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
        { "nodeId": "shot", "type": "camera.shot", "params": { "windowSize": 100, "cornerRadius": 0, "shadow": 0, "liftColours": false } },
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
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
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
    let output = project.join("ours.png");
    render_still(&StillRender {
        driver: DriverOptions {
            executable: engine.browser().unwrap(),
            temp_dir: out,
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
        frame: spec.duration_in_frames - 1,
        format: StillFormat::Png,
        jpeg_quality: None,
        output: &output,
    })
    .await
    .unwrap_or_else(|e| panic!("{base:X}: {e}"));
    output
}

fn reference(cell: &Cell, base: u32) -> GrayImage {
    let name = format!(
        "U+{base:X}...U+{:X}-{}x{}+{}.png",
        base + 0xff,
        cell.width,
        cell.height,
        cell.thickness
    );
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/sprites")
        .join(name);
    image::open(&path)
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
        .to_luma8()
}

fn tile(image: &GrayImage, x0: u32, y0: u32, cell: &Cell) -> Vec<u8> {
    (0..cell.height)
        .flat_map(|y| (0..cell.width).map(move |x| (x, y)))
        .map(|(x, y)| image.get_pixel(x0 + x, y0 + y).0[0])
        .collect()
}

fn alike(ours: &[u8], theirs: &[u8]) -> f64 {
    let both: u32 = ours
        .iter()
        .zip(theirs)
        .map(|(a, b)| u32::from(*a.min(b)))
        .sum();
    let either: u32 = ours
        .iter()
        .zip(theirs)
        .map(|(a, b)| u32::from(*a.max(b)))
        .sum();
    if either == 0 {
        return 1.0;
    }
    f64::from(both) / f64::from(either)
}

#[tokio::test]
#[ignore = "needs the dev engine and the film page (bun run film:bundle)"]
async fn draws_every_sprite_like_ghosttys_own_reference_atlases() {
    let out = home().join(".cache/opsis-film-test/sprites");
    std::fs::create_dir_all(&out).unwrap();
    let mut unlike = Vec::new();
    for cell in &CELLS {
        let pad_x = cell.width / 4;
        let pad_y = cell.height / 4;
        for base in ATLASES {
            let ours = image::open(render_grid(cell, base, &out).await)
                .unwrap()
                .to_luma8();
            let theirs = reference(cell, base);
            for cp in (base..base + GRID * GRID).filter(|cp| drawn(*cp)) {
                let (col, row) = ((cp - base) % GRID, (cp - base) / GRID);
                let ours = tile(
                    &ours,
                    col * SPREAD * cell.width,
                    row * SPREAD * cell.height,
                    cell,
                );
                let theirs = tile(
                    &theirs,
                    col * (cell.width + 2 * pad_x) + pad_x,
                    row * (cell.height + 2 * pad_y) + pad_y,
                    cell,
                );
                let lit = theirs.iter().filter(|p| **p >= LIT).count();
                let score = alike(&ours, &theirs);
                if lit >= SMALL_GLYPH && score < MOST_ALIKE {
                    unlike.push(format!(
                        "U+{cp:X} at {}x{}+{}: {score:.2}",
                        cell.width, cell.height, cell.thickness
                    ));
                }
            }
        }
    }
    assert!(
        unlike.is_empty(),
        "{} unlike Ghostty:\n{}",
        unlike.len(),
        unlike.join("\n")
    );
}
