use opsis_cases::{HEIGHT, WIDTH, app_file, case_font_files, fallback_font_files, ids, scene};
use opsis_layout::fonts::Fonts;
use opsis_layout::output::{Laid, PlacedKind};
use opsis_layout::{Images, Viewport, lay_out};
use serde_json::{Value, json};

fn case_fonts() -> Fonts {
    let mut fonts = Fonts::new();
    for (name, path) in case_font_files() {
        let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        fonts
            .register(&name, &bytes)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
    }
    for (subset, path) in fallback_font_files() {
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

fn boxes(laid: &Laid) -> Value {
    laid.boxes
        .iter()
        .map(|placed| {
            let lines: Vec<_> = match &placed.kind {
                PlacedKind::Text { lines, .. } => {
                    lines.iter().map(|r| placed.matrix.bounds(*r)).collect()
                }
                PlacedKind::Element(_) => Vec::new(),
            };
            json!({ "path": placed.path, "client": placed.client, "lines": lines })
        })
        .collect()
}

#[test]
fn every_case_lays_out_as_recorded() {
    let mut fonts = case_fonts();
    let viewport = Viewport {
        width: WIDTH,
        height: HEIGHT,
    };
    for id in ids() {
        let root = scene(id).unwrap_or_else(|| panic!("no case {id}"));
        let laid = lay_out(
            &root,
            viewport,
            &mut fonts,
            &Images {
                size_of: &picture_size,
            },
        );
        opsis_golden::check(&format!("layout/{id}.json"), &boxes(&laid));
    }
}
