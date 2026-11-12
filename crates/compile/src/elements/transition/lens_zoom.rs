use opsis_macros::settings;
use std::f64::consts::PI;

use opsis_scene::color::Color;
use opsis_scene::easing::Easing;
use opsis_scene::interpolate::{Interpolation, interpolate};
use opsis_scene::node::{Node, SvgElement, div, svg};
use opsis_scene::paint::{BlendMode, ClipPath, FillRule, Filter};
use opsis_scene::random::js_number_text;
use opsis_scene::style::{Overflow, Position, Style};
use opsis_scene::values::{Angle, Length};
use opsis_types::looks::NodeLook;
use opsis_types::numbers::{exact_eq, truncate, whole};

use crate::elements::draw::{Cx, DrawFailure, Transition};
use crate::elements::entry::{element, meta};
use crate::elements::transition::present::filled;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct LensZoom {
    #[setting(
        "Lens steps",
        default = 9,
        range(min = 4, max = 12),
        step = 1,
        advanced
    )]
    pub lens_steps: u32,
    #[setting(
        "Blur samples",
        default = 7,
        range(min = 2, max = 12),
        step = 1,
        advanced
    )]
    pub blur_samples: u32,
    #[setting("Shake amount", default = 50, range(min = 0, max = 300), step = 5)]
    pub shake_amount: u32,
    #[setting(
        "Shake travel (px)",
        default = 73,
        range(min = 0, max = 80),
        step = 1,
        advanced
    )]
    pub shake_translate_px: u32,
    #[setting(
        "Colour fringe",
        default = true,
        help = "Splits the colours at the edges. Off draws the transition a third as often, for heavy scenes.",
        advanced
    )]
    pub color_split: bool,
    #[setting("Zoom", default = 150.0, range(min = 100, max = 300), unit = "%")]
    pub zoom: f64,
    #[setting("Zoom blur", default = 60.0, range(min = 0, max = 100))]
    pub zoom_blur: f64,
    #[setting(
        "Distortion",
        default = 160.0,
        range(min = 0, max = 178),
        unit = "°",
        advanced
    )]
    pub distortion: f64,
    #[setting("Twist", default = -15.0, range(min = -90, max = 90), unit = "°", advanced)]
    pub twist: f64,
    #[setting(
        "Pull back first",
        default = 100.0,
        range(min = 50, max = 100),
        unit = "%",
        advanced
    )]
    pub pull_back: f64,
    #[setting(
        "Shake speed",
        default = 11.2,
        range(min = 1, max = 30),
        unit = "per second",
        advanced
    )]
    pub shake_speed: f64,
    #[setting(
        "Shake rotation",
        default = 1.1,
        range(min = 0, max = 10),
        unit = "°",
        advanced
    )]
    pub shake_rotation: f64,
    #[setting(
        "Fringe amount",
        default = 1.0,
        range(min = 0, max = 5),
        unit = "%",
        advanced
    )]
    pub fringe: f64,
    #[setting("Letterbox", default = 0.0, range(min = 0, max = 4), advanced)]
    pub letterbox: f64,
    #[setting("Background", default = "#000000", widget = "color", check = colour, advanced)]
    pub background: String,
}

pub fn definition() -> ElementMeta {
    meta::<LensZoom>(
            element(
                ElementKind::Presentation,
                "lens-zoom",
                "Lens zoom",
                "A hard cut disguised as a lens punch-in: barrel distortion, zoom blur, chromatic split and camera shake around the splice.",
                ElementCategory::Transition,
                28,
            )
                .look(NodeLook::of("lucide:zoom-in")),
        )
}

const SHAKE_STOPS: [f64; 4] = [-9.0 / 11.0, -3.0 / 11.0, 4.0 / 11.0, 8.0 / 11.0];
const EASE_IN: Easing = Easing::Bezier(0.54, 0.0, 0.82, 0.8);
const EASE_OUT: Easing = Easing::Bezier(0.18, 0.2, 0.46, 1.0);
const EXIT_ZOOM_SHARE: f64 = 0.9;
const EXIT_BLUR_SHARE: f64 = 0.75;
const EXIT_FOV_SHARE: f64 = 135.0 / 160.0;
const ANTICIPATE_PORTION: f64 = 0.35;
const BLUR_FADE: f64 = 1.5;
const MAX_FOV: f64 = 178.0;
const PEAK_FLOOR: f64 = 1.001;
const RADIUS_STEPS: u32 = 24;
const SECOND_OCTAVE: f64 = 0.45;
const SECOND_OCTAVE_RATE: f64 = 2.6;
const SECOND_OCTAVE_SEED: i32 = 977;
const CHANNELS: [(&str, &str); 3] = [
    ("r", "1 0 0 0 0  0 0 0 0 0  0 0 0 0 0  0 0 0 1 0"),
    ("g", "0 0 0 0 0  0 1 0 0 0  0 0 0 0 0  0 0 0 1 0"),
    ("b", "0 0 0 0 0  0 0 0 0 0  0 0 1 0 0  0 0 0 1 0"),
];
const FILTER_ID: &str = "lens-zoom";

fn hashed(seed: i32, i: i32) -> f64 {
    let mut t = (i ^ seed).cast_unsigned().wrapping_mul(0x27d4_eb2d);
    t ^= t >> 15;
    t = t.wrapping_mul(0x85eb_ca6b);
    t ^= t >> 13;
    f64::from(t) / 4_294_967_296.0 * 2.0 - 1.0
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film's shake rounds each step, and the frames have to match it"
)]
fn value_noise(seed: i32, t: f64) -> f64 {
    let i: i32 = truncate(t.floor());
    let f = t - t.floor();
    let u = f * f * (3.0 - 2.0 * f);
    hashed(seed, i) * (1.0 - u) + hashed(seed, i + 1) * u
}

fn fov_half(fov: f64) -> f64 {
    fov.min(MAX_FOV) / 2.0 * (PI / 180.0)
}

fn lens_magnification(r: f64, fov: f64) -> f64 {
    if fov < 0.01 {
        return 1.0;
    }
    let theta = fov_half(fov);
    let peak = theta.tan() / theta;
    if r <= 0.0 {
        return peak;
    }
    let a = r * theta;
    if a >= PI / 2.0 - 1e-3 {
        return 1.0;
    }
    let source = a.tan() / theta.tan();
    if source <= 1e-6 {
        return peak;
    }
    (r / source).max(1.0)
}

fn radius_for_magnification(target: f64, fov: f64) -> f64 {
    let (mut lo, mut hi) = (0.0, 1.0);
    for _ in 0..RADIUS_STEPS {
        let mid = f64::midpoint(lo, hi);
        if lens_magnification(mid, fov) > target {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    f64::midpoint(lo, hi)
}

fn circle(r: f64, clockwise: bool, (cx, cy): (f64, f64)) -> String {
    let n = js_number_text;
    let s = u8::from(clockwise);
    format!(
        "M {} {} A {} {} 0 1 {s} {} {} A {} {} 0 1 {s} {} {} Z",
        n(cx + r),
        n(cy),
        n(r),
        n(r),
        n(cx - r),
        n(cy),
        n(r),
        n(r),
        n(cx + r),
        n(cy)
    )
}

fn ring_clip(inner: f64, outer: Option<f64>, (w, h): (f64, f64), center: (f64, f64)) -> ClipPath {
    let n = js_number_text;
    let outer = outer.map_or_else(
        || format!("M 0 0 L {} 0 L {} {} L 0 {} Z", n(w), n(w), n(h), n(h)),
        |r| circle(r, true, center),
    );
    let data = if inner > 0.0 {
        format!("{outer} {}", circle(inner, false, center))
    } else {
        outer
    };
    ClipPath::Path {
        fill: FillRule::Nonzero,
        data,
    }
}

#[derive(Debug, Clone, Copy)]
struct Lens {
    scale: f64,
    fov: f64,
    blur: f64,
    rotation: f64,
}

fn centered(style: Style, (cx, cy): (f64, f64)) -> Style {
    style.transform_origin((Length::px(cx), Length::px(cy), Length::ZERO))
}

fn half(lens: Lens, props: &LensZoom, size: (f64, f64), scene: &[Node]) -> Node {
    let center = (size.0 / 2.0, size.1 / 2.0);
    let base = lens.scale / 100.0;
    let peak = lens_magnification(0.0, lens.fov);
    let steps = if peak <= PEAK_FLOOR { 0 } else { props.lens_steps };
    let rings: Vec<(f64, f64)> = (0..steps)
        .map(|i| {
            let s = 1.0 + (peak - 1.0) * (f64::from(i) + 0.5) / f64::from(steps);
            (s, radius_for_magnification(s, lens.fov))
        })
        .collect();
    let samples = if lens.blur <= 0.05 {
        1
    } else {
        whole(f64::from(props.blur_samples) * lens.blur / 60.0).max(2)
    };
    let weights: Vec<f64> = (0..samples)
        .map(|j| (1.0 - f64::from(j) / f64::from(samples)).powf(BLUR_FADE))
        .collect();
    let copy = |clip: ClipPath, k: f64| -> Node {
        filled(
            centered(
                Style::default()
                    .clip_path(clip)
                    .rotate(Angle::deg(lens.rotation))
                    .scale(k, k),
                center,
            ),
            scene.to_vec(),
        )
    };
    let mut running = 0.0;
    let groups = (0..samples)
        .rev()
        .map(|j| {
            let weight = weights[usize::try_from(j).unwrap_or(0)];
            running += weight;
            let g = if samples == 1 {
                1.0
            } else {
                1.0 + lens.blur / 100.0 * (f64::from(j) / f64::from(samples - 1))
            };
            let half_width = size.0 / 2.0 * g;
            let first = rings.first().map_or(0.0, |r| r.1 * half_width);
            let mut copies = vec![copy(ring_clip(first, None, size, center), g * base)];
            for (i, (s, r)) in rings.iter().enumerate() {
                let inner = rings.get(i + 1).map_or(0.0, |next| next.1 * half_width);
                copies.push(copy(ring_clip(inner, Some(r * half_width), size, center), g * s * base));
            }
            filled(Style::default().opacity(weight / running), copies)
        })
        .collect();
    filled(Style::default(), groups)
}

fn shake(cx: &Cx<'_>, props: &LensZoom, envelope: f64) -> (f64, f64, f64) {
    let t = cx.frame() / cx.fps() * props.shake_speed;
    let octave = |seed: i32| {
        SECOND_OCTAVE.mul_add(
            value_noise(seed + SECOND_OCTAVE_SEED, t * SECOND_OCTAVE_RATE),
            value_noise(seed, t),
        )
    };
    let reach = f64::from(props.shake_translate_px) * envelope;
    (
        octave(11) * reach,
        octave(23) * reach,
        octave(41) * props.shake_rotation * envelope,
    )
}

fn stage(lens: Lens, props: &LensZoom, envelope: f64, scene: &[Node], cx: &Cx<'_>) -> Node {
    let size = (cx.at.stage.width, cx.at.stage.height);
    let center = (size.0 / 2.0, size.1 / 2.0);
    let core = half(lens, props, size, scene);
    let fringe = props.fringe / 100.0;
    let scales = if props.color_split {
        [1.0 + fringe, 1.0, 1.0 - fringe]
    } else {
        [1.0; 3]
    };
    let split = !exact_eq(scales[0], scales[1]) || !exact_eq(scales[2], scales[1]);
    let background = cx.color(&props.background);
    let mut layers: Vec<Node> = Vec::new();
    let composited = if split {
        let defs = CHANNELS.iter().fold(SvgElement::new("defs"), |defs, (id, matrix)| {
            defs.child(
                SvgElement::new("filter")
                    .attr("id", format!("{FILTER_ID}-{id}"))
                    .attr("color-interpolation-filters", "sRGB")
                    .child(SvgElement::new("feColorMatrix").attr("type", "matrix").attr("values", *matrix)),
            )
        });
        layers.push(
            svg(
                SvgElement::new("svg").attr("width", 0).attr("height", 0).child(defs),
                Style::default().position(Position::Absolute),
            )
            .into(),
        );
        let channels = CHANNELS
            .iter()
            .zip(scales)
            .map(|((id, _), scale)| {
                filled(
                    centered(
                        Style::default()
                            .mix_blend_mode(BlendMode::Screen)
                            .filter(vec![Filter::Url(format!("{FILTER_ID}-{id}"))])
                            .scale(scale, scale),
                        center,
                    ),
                    vec![core.clone()],
                )
            })
            .collect();
        filled(Style::default().background_color(Color::BLACK), channels)
    } else {
        core
    };
    let (x, y, turn) = shake(cx, props, envelope);
    layers.push(filled(
        centered(
            Style::default()
                .translate(Length::px(x), Length::px(y))
                .rotate(Angle::deg(turn)),
            center,
        ),
        vec![composited],
    ));
    let bar = if props.letterbox > 0.0 {
        ((size.1 - size.0 / props.letterbox) / 2.0).max(0.0)
    } else {
        0.0
    };
    if bar > 0.0 {
        for top in [true, false] {
            let place = Style::default()
                .position(Position::Absolute)
                .left(Length::ZERO)
                .right(Length::ZERO)
                .height(Length::px(bar))
                .background_color(background.clone());
            let place = if top { place.top(Length::ZERO) } else { place.bottom(Length::ZERO) };
            layers.push(div(place).into());
        }
    }
    filled(
        Style::default()
            .width(Length::px(size.0))
            .height(Length::px(size.1))
            .overflow(Overflow::Hidden)
            .background_color(background),
        layers,
    )
}

pub fn present(props: &LensZoom, side: Transition, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let exiting = !side.entering;
    let progress = side.progress;
    if (exiting && progress >= 0.5) || (!exiting && progress < 0.5) {
        return Ok(filled(Style::default(), Vec::new()));
    }
    let eased = |from: [f64; 2], easing: Easing| {
        interpolate(
            progress,
            &from,
            &[0.0, 1.0],
            &Interpolation {
                easing: Some(easing),
                ..Interpolation::clamped()
            },
        )
    };
    let p = if exiting { eased([0.0, 0.5], EASE_IN)? } else { eased([0.5, 1.0], EASE_OUT)? };
    let anticipating = exiting && props.pull_back < 100.0;
    let p_fx = if anticipating {
        interpolate(p, &[ANTICIPATE_PORTION, 1.0], &[0.0, 1.0], &Interpolation::clamped())?
    } else {
        p
    };
    let lerp = |from: f64, to: f64| (to - from).mul_add(p_fx, from);
    let exit_scale = if anticipating {
        interpolate(
            p,
            &[0.0, ANTICIPATE_PORTION, 1.0],
            &[100.0, props.pull_back, props.zoom],
            &Interpolation {
                easing: Some(Easing::ease_in_out(Easing::EASE)),
                ..Interpolation::clamped()
            },
        )?
    } else {
        lerp(100.0, props.zoom)
    };
    let amount = f64::from(props.shake_amount);
    let envelope = interpolate(
        progress,
        &SHAKE_STOPS.map(|s| 0.5 + s / 2.0),
        &[0.0, amount, amount, 0.0],
        &Interpolation::clamped(),
    )? / 100.0;
    let lens = if exiting {
        Lens {
            scale: exit_scale,
            fov: lerp(0.0, props.distortion),
            blur: lerp(0.0, props.zoom_blur),
            rotation: lerp(0.0, props.twist),
        }
    } else {
        Lens {
            scale: lerp(props.zoom * EXIT_ZOOM_SHARE, 100.0),
            fov: lerp(props.distortion * EXIT_FOV_SHARE, 0.0),
            blur: lerp(props.zoom_blur * EXIT_BLUR_SHARE, 0.0),
            rotation: lerp(360.0 + props.twist, 360.0),
        }
    };
    Ok(stage(lens, props, envelope, &side.children, cx))
}
