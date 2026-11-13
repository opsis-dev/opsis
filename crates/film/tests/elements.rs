use std::path::{Path, PathBuf};

use opsis_compile::elements::draw::{Cx, Transition, element, presentation, wrapper};
use opsis_film::Film;
use opsis_golden::{dev_cache, repo};
use opsis_scene::clock::Clock;
use opsis_scene::measure::Unmeasured;
use opsis_scene::node::div;
use opsis_scene::style::Style;
use opsis_scene::timeline::{At, Stage};
use opsis_skia::{Image, Paint, RectF, Sampling, Surface};
use opsis_types::numbers::{float, narrow};
use opsis_types::timeline::{TimelineItem, TimelineSpec, TimelineTrack};

const ORACLE_ENV: &str = "OPSIS_ORACLE";
const FILTER_ENV: &str = "OPSIS_FILM_ELEMENT";
const TOLERANCE: f32 = 0.05;
const WRONG_PIXEL: f32 = 0.5;
const WRONG_SHARE: f64 = 0.002;

struct Allowance {
    key: &'static str,
    frames: &'static [u32],
    wrong_share: f64,
    reason: &'static str,
}

const ALLOWANCES: [Allowance; 3] = [
    Allowance {
        key: "punch-lines",
        frames: &[0, 35, 107, 143],
        wrong_share: 0.006,
        reason: "fractional glyph advances; the Linux oracle rounds each advance to a whole pixel, which moves very large words by up to a pixel",
    },
    Allowance {
        key: "announce-title",
        frames: &[84, 169],
        wrong_share: 0.012,
        reason: "fractional glyph advances; the Linux oracle rounds each advance to a whole pixel, which moves the 1166 px tagline by about a pixel",
    },
    Allowance {
        key: "announce-title",
        frames: &[0],
        wrong_share: 0.016,
        reason: "the 3D receding plane is drawn as vectors through the perspective, where Chrome rasterises the layer first and projects it with stair-stepped edges",
    },
];

fn wrong_share_for(key: &str, frame: u32) -> f64 {
    ALLOWANCES
        .iter()
        .find(|a| a.key == key && a.frames.contains(&frame) && !a.reason.is_empty())
        .map_or(WRONG_SHARE, |a| a.wrong_share)
}
const MAX_WORKERS: usize = 8;

fn oracle_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os(ORACLE_ENV) {
        return PathBuf::from(dir);
    }
    let root = dev_cache("oracle");
    std::fs::read_dir(&root)
        .unwrap_or_else(|e| panic!("{}: {e}. Run bun run oracle first.", root.display()))
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.join("elements.json").is_file())
        .min()
        .unwrap_or_else(|| panic!("no oracle with elements in {}", root.display()))
}

fn app_file(source: &str) -> Option<PathBuf> {
    source
        .strip_prefix("app:")
        .map(|file| repo().join("public").join(file))
}

fn file_bytes(source: &str) -> Option<Vec<u8>> {
    std::fs::read(app_file(source)?).ok()
}

fn picture_size(source: &str) -> Option<(f64, f64)> {
    let probe = opsis_probe::probe(&app_file(source)?).ok()?;
    let picture = probe.video?;
    Some((f64::from(picture.width), f64::from(picture.height)))
}

fn read_spec(dir: &Path) -> TimelineSpec {
    let file = dir.join("spec.json");
    let text = std::fs::read_to_string(&file).unwrap_or_else(|e| panic!("{}: {e}", file.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", file.display()))
}

fn items_drawn(tracks: &[TimelineTrack], cx: &Cx<'_>) -> bool {
    tracks.iter().flat_map(|t| t.items.iter()).all(|item| {
        let base = item.base();
        let presented = [&base.enter, &base.exit]
            .into_iter()
            .flatten()
            .all(|transition| {
                presentation(
                    &transition.kind,
                    transition.params.as_ref(),
                    Transition {
                        entering: true,
                        progress: 0.0,
                        duration: 1.0,
                        children: Vec::new(),
                    },
                    cx,
                )
                .is_ok_and(|d| d.is_some())
            });
        presented
            && match item {
                TimelineItem::Element {
                    component, props, ..
                } => element(component, props, cx).is_ok_and(|d| d.is_some()),
                TimelineItem::Group { props, .. } => {
                    props.wrapper.as_ref().is_none_or(|w| {
                        wrapper(
                            &w.component,
                            &w.props,
                            &|_| Ok(div(Style::default()).into()),
                            cx,
                        )
                        .is_ok_and(|d| d.is_some())
                    }) && props.sequencer.is_none()
                        && items_drawn(&props.tracks, cx)
                }
                TimelineItem::Recording { .. } => false,
            }
    })
}

fn is_drawn(spec: &TimelineSpec) -> bool {
    let at = At {
        clock: Clock::new(0.0, spec.fps, 1.0),
        stage: Stage {
            width: f64::from(spec.width),
            height: f64::from(spec.height),
        },
    };
    items_drawn(
        &spec.tracks,
        &Cx {
            at: &at,
            measure: &Unmeasured,
        },
    )
}

fn full_rect((width, height): (u32, u32)) -> RectF {
    RectF::new(
        0.0,
        0.0,
        narrow(f64::from(width)),
        narrow(f64::from(height)),
    )
}

fn rgba_of(path: &Path, (width, height): (u32, u32)) -> Vec<u8> {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let image = Image::decode(&bytes).expect("a decoded oracle picture");
    let mut surface = Surface::new(width, height).expect("a surface");
    let full = full_rect((width, height));
    surface.draw_image_rect(&image, (full, full), Sampling::Nearest, &Paint::new());
    surface.rgba()
}

fn rgb(rgba: &[u8]) -> Vec<u8> {
    rgba.chunks(4).flat_map(|p| [p[0], p[1], p[2]]).collect()
}

#[derive(Debug, Clone, Copy)]
struct Score {
    mean: f32,
    wrong: f64,
}

impl Score {
    fn passes(self, wrong_share: f64) -> bool {
        self.mean <= TOLERANCE && self.wrong <= wrong_share
    }
}

fn flip(want: &[u8], got: &[u8], (width, height): (u32, u32)) -> (Score, Vec<u8>) {
    let a = nv_flip::FlipImageRgb8::with_data(width, height, &rgb(want));
    let b = nv_flip::FlipImageRgb8::with_data(width, height, &rgb(got));
    let error = nv_flip::flip(a, b, nv_flip::DEFAULT_PIXELS_PER_DEGREE);
    let mean = nv_flip::FlipPool::from_image(&error).mean();
    let values = error.to_vec();
    let wrong =
        float(values.iter().filter(|v| **v > WRONG_PIXEL).count()) / float(values.len().max(1));
    let map = error.apply_color_lut(&nv_flip::magma_lut()).to_vec();
    (
        Score { mean, wrong },
        map.chunks(3)
            .flat_map(|p| [p[0], p[1], p[2], 255])
            .collect(),
    )
}

fn frames_of(dir: &Path) -> Vec<(u32, PathBuf)> {
    let mut frames: Vec<(u32, PathBuf)> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter_map(|p| {
            let frame = p
                .extension()
                .is_some_and(|e| e == "png")
                .then(|| p.file_stem()?.to_str()?.parse().ok())??;
            Some((frame, p))
        })
        .collect();
    frames.sort();
    frames
}

fn compare(key: &str, dir: &Path, out: &Path) -> Option<Vec<(u32, Score)>> {
    let spec = read_spec(dir);
    if !is_drawn(&spec) {
        return None;
    }
    let size = (spec.width, spec.height);
    let mut film =
        Film::new(&spec, &file_bytes, &picture_size).unwrap_or_else(|e| panic!("{key}: {e}"));
    let mut results = Vec::new();
    for (frame, path) in frames_of(dir) {
        let ours = film
            .frame(f64::from(frame))
            .unwrap_or_else(|e| panic!("{key} at {frame}: {e}"));
        let (score, map) = flip(&rgba_of(&path, size), &ours.rgba(), size);
        std::fs::write(out.join(format!("{key}-{frame:05}.png")), ours.png()).expect("our picture");
        let error = Image::from_rgba(size.0, size.1, &map).expect("the error map");
        let mut surface = Surface::new(size.0, size.1).expect("a surface");
        let full = full_rect(size);
        surface.draw_image_rect(&error, (full, full), Sampling::Nearest, &Paint::new());
        std::fs::write(
            out.join(format!("{key}-{frame:05}.flip.png")),
            surface.png(),
        )
        .expect("the error map");
        results.push((frame, score));
    }
    Some(results)
}

#[test]
#[ignore = "needs the oracle's element frames (bun run oracle elements)"]
fn every_drawn_element_matches_the_browser() {
    let root = oracle_dir().join("elements");
    let out = dev_cache("film-elements");
    std::fs::create_dir_all(&out).expect("the output folder");
    let filter = std::env::var(FILTER_ENV).unwrap_or_default();
    let mut keys: Vec<String> = std::fs::read_dir(&root)
        .unwrap_or_else(|e| panic!("{}: {e}", root.display()))
        .filter_map(Result::ok)
        .filter_map(|e| e.file_name().to_str().map(str::to_owned))
        .filter(|k| k.contains(&filter))
        .collect();
    keys.sort();
    let workers = std::thread::available_parallelism()
        .map_or(1, usize::from)
        .min(MAX_WORKERS);
    let next = std::sync::atomic::AtomicUsize::new(0);
    let scored = std::sync::Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| {
                loop {
                    let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let Some(key) = keys.get(i) else {
                        break;
                    };
                    if let Some(results) = compare(key, &root.join(key), &out) {
                        scored
                            .lock()
                            .expect("the results")
                            .push((key.clone(), results));
                    }
                }
            });
        }
    });
    let mut scored = scored.into_inner().expect("the results");
    scored.sort_by(|a, b| a.0.cmp(&b.0));
    let mut lines = Vec::new();
    let mut failing = Vec::new();
    for (key, results) in scored {
        for (frame, score) in results {
            let line = format!("{:.4} {:.5}  {key} {frame}", score.mean, score.wrong);
            if !score.passes(wrong_share_for(&key, frame)) {
                failing.push(line.clone());
            }
            lines.push(line);
        }
    }
    println!("{}", lines.join("\n"));
    println!("pictures in {}", out.display());
    assert!(!lines.is_empty(), "no element is drawn natively");
    assert!(
        failing.is_empty(),
        "above a mean of {TOLERANCE} or {WRONG_SHARE} of pixels above {WRONG_PIXEL}: {}",
        failing.join(", ")
    );
}
