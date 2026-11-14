use std::path::PathBuf;
use std::sync::Arc;

use opsis_compile::elements::lengths::fits_its_animation;
use opsis_compile::elements::{ElementMeta, elements};
use opsis_engine::tools::EngineFiles;
use opsis_film::{Drawing, Driver, Film, renderer_program};
use opsis_footage::decoder::FootageTools;
use opsis_footage::footage::Footage;
use opsis_golden::{dev_cache, home, repo};
use opsis_prepare::serve::PreparedDirs;
use opsis_render::element_spec::{element_spec, frame_at, frames_at};
use opsis_render::native::{NATIVE_BASE, NativeFiles};
use opsis_scene::node::{Kind, Node};

const PICTURES: &str = "elements";
const SCENES: &str = "scene/elements";
const SCALE: f64 = 0.25;
const MIDDLE: f64 = 0.5;
const SCENE_POSITIONS: [f64; 3] = [0.25, 0.5, 0.75];
const TOLERANCE: f32 = 0.05;
const FILAMENT_SHARE: f64 = 0.0005;
const SOFTWARE_VULKAN: &str = "libvk_swiftshader.so";
const WORKERS: usize = 8;
const SCENE_ELEMENTS: [&str; 12] = [
    "title",
    "typewriter",
    "stat-card",
    "pricing-card",
    "laptop-frame",
    "confetti",
    "spring-settle",
    "shader-mesh-gradient",
    "card-3d",
    "glitch-cut",
    "check-list",
    "animated-bar-chart",
];

fn software() -> Driver {
    let engine = EngineFiles::downloaded(&home().join(".config/Opsis/engine"));
    let browser = engine
        .browser()
        .unwrap_or_else(|e| panic!("the software graphics driver comes with the engine: {e}"));
    Driver::Software {
        library: browser
            .parent()
            .map_or_else(PathBuf::new, |dir| dir.join(SOFTWARE_VULKAN)),
    }
}

fn files(out: &std::path::Path) -> Arc<NativeFiles> {
    let project = out.join("project");
    std::fs::create_dir_all(&project).unwrap();
    Arc::new(NativeFiles {
        dirs: PreparedDirs {
            project,
            public: repo().join("public"),
        },
        footage: Arc::new(Footage::new(
            FootageTools {
                ffmpeg: opsis_golden::ffmpeg(),
                temp: out.to_path_buf(),
            },
            Vec::new(),
        )),
        drawing: Drawing {
            program: renderer_program(),
            driver: software(),
            temp: out.join("drawing"),
        },
    })
}

fn exact_setup() -> bool {
    #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
    {
        std::arch::is_x86_feature_detected!("avx2")
    }
    #[cfg(not(all(target_arch = "x86_64", target_os = "linux")))]
    {
        false
    }
}

fn film_for<'f>(
    files: &'f NativeFiles,
    spec: &'f opsis_types::timeline::TimelineSpec,
    bytes: &'f (dyn Fn(&str) -> Option<Vec<u8>> + Sync),
    sizes: &'f dyn Fn(&str) -> Option<(f64, f64)>,
    pictures: &'f (dyn Fn(&str) -> Option<opsis_paint::Picture> + Sync),
) -> Film<'f> {
    Film::new(spec, bytes, sizes)
        .unwrap_or_else(|e| panic!("{e}"))
        .with_frames(pictures)
        .with_drawing(files.drawing.clone())
}

fn rgb(rgba: &[u8]) -> Vec<u8> {
    rgba.as_chunks::<4>()
        .0
        .iter()
        .flat_map(|[r, g, b, _]| [*r, *g, *b])
        .collect()
}

fn differing_share(a: &[u8], b: &[u8]) -> f64 {
    let pixels = a.as_chunks::<4>().0.iter().zip(b.as_chunks::<4>().0);
    let (mut differing, mut all) = (0usize, 0usize);
    for (p, q) in pixels {
        all += 1;
        differing += usize::from(p != q);
    }
    opsis_types::numbers::float(differing) / opsis_types::numbers::float(all.max(1))
}

fn compared(
    key: &str,
    png: &[u8],
    made: &[u8],
    size: (u32, u32),
    filament: bool,
) -> Option<String> {
    let recorded = image::load_from_memory(png)
        .unwrap_or_else(|e| panic!("{key}: {e}"))
        .to_rgba8();
    if recorded.dimensions() != size {
        return Some(format!(
            "{key}: size {:?} against {size:?}",
            recorded.dimensions()
        ));
    }
    if exact_setup() {
        let share = differing_share(recorded.as_raw(), made);
        let allowed = if filament { FILAMENT_SHARE } else { 0.0 };
        return (share > allowed)
            .then(|| format!("{key}: {share:.5} of its pixels differ from its golden"));
    }
    let a = nv_flip::FlipImageRgb8::with_data(size.0, size.1, &rgb(recorded.as_raw()));
    let b = nv_flip::FlipImageRgb8::with_data(size.0, size.1, &rgb(made));
    let mean =
        nv_flip::FlipPool::from_image(&nv_flip::flip(a, b, nv_flip::DEFAULT_PIXELS_PER_DEGREE))
            .mean();
    (mean > TOLERANCE).then(|| format!("{key}: mean {mean:.4} above {TOLERANCE}"))
}

struct Drawn {
    png: Vec<u8>,
    rgba: Vec<u8>,
    size: (u32, u32),
    filament: bool,
}

fn drawn_by_filament(node: &Node) -> bool {
    let Node::Element(element) = node else {
        return false;
    };
    matches!(element.kind, Kind::Stage3d(_) | Kind::Effect(_))
        || element.children.iter().any(drawn_by_filament)
}

fn picture_of(meta: &ElementMeta, files: &NativeFiles) -> Result<Drawn, String> {
    let spec = element_spec(meta, NATIVE_BASE).map_err(|e| e.to_string())?;
    let bytes = |source: &str| files.bytes(source);
    let sizes = |source: &str| files.size(source);
    let pictures = |source: &str| files.frame(source);
    let mut film = film_for(files, &spec, &bytes, &sizes, &pictures);
    let frame = f64::from(frame_at(spec.duration_in_frames, MIDDLE));
    let filament = drawn_by_filament(&film.scene(frame).map_err(|e| e.to_string())?);
    let surface = film.scaled_frame(frame, SCALE).map_err(|e| e.to_string())?;
    Ok(Drawn {
        png: surface.png(),
        rgba: surface.rgba(),
        size: (surface.width(), surface.height()),
        filament,
    })
}

#[test]
#[ignore = "needs the dev engine's software graphics driver and the opsis-renderer program"]
fn every_element_matches_its_golden_picture() {
    assert!(
        !opsis_golden::writing() || exact_setup(),
        "golden pictures are recorded on x86 Linux with AVX2"
    );
    let out = dev_cache("goldens");
    let files = files(&out);
    let all: Vec<&ElementMeta> = elements().iter().collect();
    let failures = std::sync::Mutex::new(Vec::new());
    let next = std::sync::atomic::AtomicUsize::new(0);
    std::thread::scope(|scope| {
        for _ in 0..WORKERS {
            scope.spawn(|| {
                loop {
                    let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let Some(meta) = all.get(i) else {
                        break;
                    };
                    let file = opsis_golden::path(&format!("{PICTURES}/{}.png", meta.key));
                    let failed = match picture_of(meta, &files) {
                        Err(error) => Some(format!("{}: {error}", meta.key)),
                        Ok(Drawn {
                            png,
                            rgba,
                            size,
                            filament,
                        }) => {
                            if opsis_golden::writing() {
                                std::fs::create_dir_all(file.parent().unwrap()).unwrap();
                                std::fs::write(&file, &png).unwrap();
                                None
                            } else {
                                std::fs::write(out.join(format!("{}.png", meta.key)), &png)
                                    .unwrap();
                                let recorded = std::fs::read(&file)
                                    .unwrap_or_else(|e| panic!("{}: {e}", file.display()));
                                compared(&meta.key, &recorded, &rgba, size, filament)
                            }
                        }
                    };
                    if let Some(failed) = failed {
                        failures.lock().unwrap().push(failed);
                    }
                }
            });
        }
    });
    let failures = failures.into_inner().unwrap();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
#[ignore = "needs the dev engine's software graphics driver and the opsis-renderer program"]
fn chosen_elements_build_their_golden_scenes() {
    let out = dev_cache("goldens");
    let files = files(&out);
    for key in SCENE_ELEMENTS {
        let meta = elements()
            .iter()
            .find(|m| m.key == key)
            .unwrap_or_else(|| panic!("no element {key}"));
        let spec = element_spec(meta, NATIVE_BASE).unwrap_or_else(|e| panic!("{key}: {e}"));
        let bytes = |source: &str| files.bytes(source);
        let sizes = |source: &str| files.size(source);
        let pictures = |source: &str| files.frame(source);
        let mut film = film_for(&files, &spec, &bytes, &sizes, &pictures);
        for frame in frames_at(spec.duration_in_frames, &SCENE_POSITIONS) {
            let scene = film
                .scene(f64::from(frame))
                .unwrap_or_else(|e| panic!("{key} at {frame}: {e}"));
            let value = serde_json::to_value(&scene).unwrap();
            opsis_golden::check(&format!("{SCENES}/{key}@{frame}.json"), &value);
        }
    }
}

const SETTLED: f64 = 1e-5;

const KEEPS_MOVING: [(&str, &str); 19] = [
    (
        "shader-mesh-gradient",
        "a shader background animates for as long as it is shown",
    ),
    (
        "shader-grain-gradient",
        "a shader background animates for as long as it is shown",
    ),
    (
        "shader-warp",
        "a shader background animates for as long as it is shown",
    ),
    (
        "shader-swirl",
        "a shader background animates for as long as it is shown",
    ),
    (
        "shader-god-rays",
        "a shader background animates for as long as it is shown",
    ),
    (
        "shader-perlin-noise",
        "a shader background animates for as long as it is shown",
    ),
    (
        "shader-dithering",
        "a shader background animates for as long as it is shown",
    ),
    ("confetti", "the pieces keep falling until the element ends"),
    (
        "neon-flicker-text",
        "the sign flickers for as long as it is shown",
    ),
    (
        "text-mask-video",
        "the video inside the letters keeps playing",
    ),
    ("status-cycle", "it keeps cycling through its states"),
    (
        "logo-drift",
        "the logos drift for as long as they are shown",
    ),
    ("signup-flow", "its spinner keeps turning"),
    ("ai-prompt-flow", "its spinner and shimmer keep moving"),
    (
        "laptop-frame",
        "the laptop bobs and its typing dots pulse for as long as it is shown",
    ),
    ("punch-lines", "the last line keeps pushing in"),
    ("mask-reveal-up", "its exit plays over its last frames"),
    ("line-by-line-slide", "its exit plays over its last frames"),
    ("logo-wipe", "its wipe out plays over its last frames"),
];

fn numbers_in(text: &str) -> (String, Vec<f64>) {
    let mut shape = String::new();
    let mut numbers = Vec::new();
    let mut current = String::new();
    for c in text.chars() {
        if c.is_ascii_digit() || c == '.' || c == '-' || c == 'e' {
            current.push(c);
            continue;
        }
        if let Ok(value) = current.parse::<f64>() {
            numbers.push(value);
            shape.push('#');
        } else {
            shape.push_str(&current);
        }
        current.clear();
        shape.push(c);
    }
    if let Ok(value) = current.parse::<f64>() {
        numbers.push(value);
        shape.push('#');
    } else {
        shape.push_str(&current);
    }
    (shape, numbers)
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= SETTLED * a.abs().max(b.abs()).max(1.0)
}

fn settled(a: &serde_json::Value, b: &serde_json::Value) -> bool {
    use serde_json::Value;
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => match (x.as_f64(), y.as_f64()) {
            (Some(x), Some(y)) => close(x, y),
            _ => x == y,
        },
        (Value::String(x), Value::String(y)) => {
            let ((shape_x, xs), (shape_y, ys)) = (numbers_in(x), numbers_in(y));
            shape_x == shape_y
                && xs.len() == ys.len()
                && xs.iter().zip(&ys).all(|(x, y)| close(*x, *y))
        }
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(x, y)| settled(x, y))
        }
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len()
                && x.iter()
                    .all(|(k, v)| y.get(k).is_some_and(|w| settled(v, w)))
        }
        _ => a == b,
    }
}

#[test]
#[ignore = "needs the dev engine's software graphics driver and the opsis-renderer program"]
fn elements_hold_still_after_their_animation() {
    let out = dev_cache("goldens");
    let files = files(&out);
    let mut moving = Vec::new();
    for meta in elements()
        .iter()
        .filter(|m| fits_its_animation(m))
        .filter(|m| !KEEPS_MOVING.iter().any(|(key, _)| *key == m.key))
    {
        let Ok(spec) = element_spec(meta, NATIVE_BASE) else {
            continue;
        };
        let last = spec.duration_in_frames.saturating_sub(1);
        if last == 0 {
            continue;
        }
        let bytes = |source: &str| files.bytes(source);
        let sizes = |source: &str| files.size(source);
        let pictures = |source: &str| files.frame(source);
        let mut film = film_for(&files, &spec, &bytes, &sizes, &pictures);
        let (Ok(before), Ok(after)) =
            (film.scene(f64::from(last - 1)), film.scene(f64::from(last)))
        else {
            continue;
        };
        let (before, after) = (
            serde_json::to_value(&before).unwrap(),
            serde_json::to_value(&after).unwrap(),
        );
        if !settled(&before, &after) {
            moving.push(meta.key.clone());
        }
    }
    assert!(
        moving.is_empty(),
        "still moving at their end: {}",
        moving.join(", ")
    );
}
