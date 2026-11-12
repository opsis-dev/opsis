use std::path::PathBuf;

use opsis_cases::{HEIGHT, WIDTH, app_file, case_font_files, ids, oracle_dir, scene};
use opsis_golden::dev_cache;
use opsis_layout::fonts::Fonts;
use opsis_layout::{Images, Viewport, lay_out};
use opsis_paint::{Resources, paint};
use opsis_skia::{Image, Paint, RectF, Rgba, Sampling, Surface};

const SIDE: (u32, u32) = (640, 360);

fn fonts() -> Fonts {
    let mut fonts = Fonts::new();
    for (name, path) in case_font_files() {
        let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        fonts
            .register(&name, &bytes)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
    }
    for (subset, path) in opsis_cases::fallback_font_files() {
        let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        fonts
            .register_fallback(&subset, &bytes)
            .unwrap_or_else(|e| panic!("{subset}: {e}"));
    }
    fonts
}

fn picture_size(source: &str) -> Option<(f64, f64)> {
    let probe = opsis_probe::probe(&app_file(source)?).ok()?;
    let picture = probe.video?;
    Some((f64::from(picture.width), f64::from(picture.height)))
}

fn file_bytes(source: &str) -> Option<Vec<u8>> {
    std::fs::read(app_file(source)?).ok()
}

fn surface_of(rgba: &[u8]) -> Surface {
    let mut surface = Surface::new(SIDE.0, SIDE.1).expect("a surface");
    let image = Image::from_rgba(SIDE.0, SIDE.1, rgba).expect("a picture");
    let full = RectF::new(0.0, 0.0, 640.0, 360.0);
    surface.draw_image_rect(&image, (full, full), Sampling::Nearest, &Paint::new());
    surface
}

fn decoded(path: &std::path::Path) -> Vec<u8> {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let image = Image::decode(&bytes).expect("a decoded oracle picture");
    let mut surface = Surface::new(SIDE.0, SIDE.1).expect("a surface");
    let full = RectF::new(0.0, 0.0, 640.0, 360.0);
    surface.draw_image_rect(&image, (full, full), Sampling::Nearest, &Paint::new());
    surface.rgba()
}

fn rgb(rgba: &[u8]) -> Vec<u8> {
    rgba.chunks(4).flat_map(|p| [p[0], p[1], p[2]]).collect()
}

fn render(id: &str, fonts: &mut Fonts) -> Option<Surface> {
    let root = scene(id)?;
    let viewport = Viewport {
        width: WIDTH,
        height: HEIGHT,
    };
    let laid = lay_out(
        &root,
        viewport,
        fonts,
        &Images {
            size_of: &picture_size,
        },
    );
    let mut surface = Surface::new(SIDE.0, SIDE.1).expect("a surface");
    surface.clear(Rgba::new(1.0, 1.0, 1.0, 1.0));
    let mut resources = Resources::new(&file_bytes, fonts.faces());
    paint(&root, &laid, viewport, &mut surface, &mut resources);
    Some(surface)
}

fn out_dir() -> PathBuf {
    dev_cache("paint-cases")
}

fn compare(id: &str, fonts: &mut Fonts) -> Option<f32> {
    let reference = oracle_dir().join("cases").join(format!("{id}.png"));
    if !reference.is_file() {
        return None;
    }
    let ours = render(id, fonts)?;
    let want = decoded(&reference);
    let got = ours.rgba();
    let a = nv_flip::FlipImageRgb8::with_data(SIDE.0, SIDE.1, &rgb(&want));
    let b = nv_flip::FlipImageRgb8::with_data(SIDE.0, SIDE.1, &rgb(&got));
    let error = nv_flip::flip(a, b, nv_flip::DEFAULT_PIXELS_PER_DEGREE);
    let mean = nv_flip::FlipPool::from_image(&error).mean();
    let map = error.apply_color_lut(&nv_flip::magma_lut()).to_vec();
    let map_rgba: Vec<u8> = map
        .chunks(3)
        .flat_map(|p| [p[0], p[1], p[2], 255])
        .collect();
    let dir = out_dir().join(id.split('/').next().unwrap_or("cases"));
    std::fs::create_dir_all(&dir).expect("the output folder");
    let name = id.split('/').next_back().unwrap_or(id);
    std::fs::write(dir.join(format!("{name}.png")), ours.png()).expect("our picture");
    std::fs::write(
        dir.join(format!("{name}.flip.png")),
        surface_of(&map_rgba).png(),
    )
    .expect("the error map");
    Some(mean)
}

const TOLERANCE: f32 = 0.05;

#[test]
#[ignore = "needs the oracle's case pictures (bun run oracle cases)"]
fn every_case_draws_like_the_browser() {
    let mut fonts = fonts();
    let filter = std::env::var("OPSIS_PAINT_CASE").unwrap_or_default();
    let mut lines = Vec::new();
    let mut failing = Vec::new();
    for id in ids().into_iter().filter(|id| id.contains(&filter)) {
        if let Some(mean) = compare(id, &mut fonts) {
            lines.push(format!("{mean:.4}  {id}"));
            if mean > TOLERANCE {
                failing.push(format!("{id}: {mean:.4}"));
            }
        }
    }
    lines.sort();
    println!("{}", lines.join("\n"));
    println!("pictures in {}", out_dir().display());
    assert!(
        failing.is_empty(),
        "above {TOLERANCE}: {}",
        failing.join(", ")
    );
}
