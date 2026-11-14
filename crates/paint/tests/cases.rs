use opsis_cases::{HEIGHT, WIDTH, app_file, case_font_files, fallback_font_files, ids, scene};
use opsis_golden::dev_cache;
use opsis_golden::pictures::{Picture, check_picture};
use opsis_layout::fonts::Fonts;
use opsis_layout::{Images, Viewport, lay_out};
use opsis_paint::{Resources, paint};
use opsis_skia::{Rgba, Surface};

const SIDE: (u32, u32) = (640, 360);

fn fonts() -> Fonts {
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

fn file_bytes(source: &str) -> Option<Vec<u8>> {
    std::fs::read(app_file(source)?).ok()
}

fn render(id: &str, fonts: &mut Fonts) -> Surface {
    let root = scene(id).unwrap_or_else(|| panic!("no case {id}"));
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
    surface
}

#[test]
fn every_case_draws_as_recorded() {
    let mut fonts = fonts();
    let out = dev_cache("paint-cases");
    let failures: Vec<String> = ids()
        .into_iter()
        .filter_map(|id| {
            let surface = render(id, &mut fonts);
            check_picture(
                &format!("cases/{id}.png"),
                Picture {
                    png: &surface.png(),
                    rgba: &surface.rgba(),
                    size: SIDE,
                },
                0.0,
                &out,
            )
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
