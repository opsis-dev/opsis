use std::f64::consts::PI;

use opsis_macros::settings;
use opsis_scene::css::{gradient, with_background};
use opsis_scene::node::{Node, SvgElement, div, svg};
use opsis_scene::paint::{BlendMode, Filter, LayerSize, MaskLayer, MaskMode};
use opsis_scene::random::{js_number_text, random};
use opsis_scene::style::{Overflow, Style};
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::{Length, Position2};
use opsis_types::looks::NodeLook;
use opsis_types::numbers::whole;

use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::shared::Colour;
use crate::elements::units::to_fixed;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct AuroraBg {
    #[setting(
        "Background color",
        default = "#050505",
        help = "Plate behind the curtains.",
        check = colour
    )]
    pub background_color: String,
    #[setting(
        "Colors",
        default = vec!["#e4ac59".into(), "#e07a5f".into(), "#c2557a".into()],
        length(min = 1),
        help = "Curtain colours, cycled."
    )]
    pub colors: Vec<Colour>,
    #[setting(
        "Ribbon count",
        default = 4,
        range(min = 1, max = 12),
        step = 1,
        help = "How many curtains."
    )]
    pub ribbon_count: u32,
    #[setting(
        "Amplitude",
        default = 9.0,
        help = "Vertical travel of a fold, in percent of the frame.",
        advanced
    )]
    pub amplitude: f64,
    #[setting(
        "Thickness",
        default = 15.0,
        help = "Curtain height at its thickest point, in percent.",
        advanced
    )]
    pub thickness: f64,
    #[setting(
        "Vertical position",
        default = 46.0,
        help = "Where the band sits, in percent of the frame height."
    )]
    pub center_y: f64,
    #[setting("Spread", default = 34.0, advanced)]
    pub spread: f64,
    #[setting(
        "Speed",
        default = 1.0,
        range(min = 0, max = 4),
        step = 0.25,
        help = "Drift speed. 0 freezes the sky."
    )]
    pub speed: f64,
    #[setting(
        "Blur",
        default = 14.0,
        help = "Much above 20 and the folds become blobs.",
        advanced
    )]
    pub blur: f64,
    #[setting("Intensity", default = 1.3, step = 0.1, help = "Overall brightness.")]
    pub intensity: f64,
    #[setting(
        "Horizon glow",
        default = 0.5,
        step = 0.01,
        help = "Ground glow under the band. 0 removes it.",
        advanced
    )]
    pub horizon_glow: f64,
    #[setting(
        "Striation",
        default = 0.55,
        step = 0.01,
        help = "Vertical ray structure through the curtains, 0-1.",
        advanced
    )]
    pub striation: f64,
    #[setting("Variation", default = 1, advanced)]
    pub seed: u32,
    #[setting("Folds", default = 2.0, range(min = 0.5, max = 6), advanced)]
    pub folds: f64,
    #[setting(
        "Sideways drift",
        default = 13.0,
        range(min = 0, max = 40),
        unit = "%/s",
        advanced
    )]
    pub sideways_drift: f64,
    #[setting(
        "Ray spacing",
        default = 72,
        range(min = 16, max = 240),
        step = 1,
        unit = "px",
        advanced
    )]
    pub ray_spacing: u32,
    #[setting("Vignette", default = 0.55, range(min = 0, max = 1), advanced)]
    pub vignette: f64,
    #[setting(
        "Horizon glow colour",
        widget = "color",
        zero_label = "First colour",
        check = colour_or_empty,
        advanced
    )]
    pub horizon_glow_color: String,
}

pub fn definition() -> ElementMeta {
    meta::<AuroraBg>(
        element(
            ElementKind::Component,
            "aurora-bg",
            "Aurora",
            "Soft ribbons of light drifting across a dark sky.",
            ElementCategory::Background,
            120,
        )
        .look(NodeLook::of("tabler:wave-sine")),
    )
}

const SAMPLES: u32 = 30;
const OVERSCAN: f64 = 12.0;
const SLOW_PERIOD: f64 = 2.3;
const FAST_PERIOD: f64 = 1.1;
const RAY_DRIFT: f64 = 9.0;

#[derive(Debug, Clone)]
struct Ribbon {
    color: String,
    band: f64,
    slow_rate: f64,
    fast_rate: f64,
    slow_phase: f64,
    fast_phase: f64,
    waves: f64,
    drift: f64,
    scale: f64,
    alpha: f64,
}

fn ray_mask(strength: f64) -> String {
    let dip = format!("{:.3}", (1.0 - strength).max(0.0));
    format!(
        "linear-gradient(90deg, rgba(0,0,0,1) 0%, rgba(0,0,0,{dip}) 38%, rgba(0,0,0,0.82) 64%, rgba(0,0,0,1) 100%)"
    )
}

fn ribbons(count: u32, colors: &[Colour], seed: u32, folds: f64, sideways: f64) -> Vec<Ribbon> {
    let draw = |index: u32, n: u32| random(format!("{seed}:{index}:{n}"));
    (0..count)
        .map(|index| {
            let t = if count == 1 {
                0.5
            } else {
                f64::from(index) / f64::from(count - 1)
            };
            Ribbon {
                color: colors
                    .get(usize::try_from(index).unwrap_or(0) % colors.len().max(1))
                    .map_or_else(String::new, |c| c.0.clone()),
                band: t - 0.5,
                slow_rate: draw(index, 1).mul_add(0.5, 0.8),
                fast_rate: draw(index, 2).mul_add(0.6, 0.75),
                slow_phase: draw(index, 3) * PI * 2.0,
                fast_phase: draw(index, 4) * PI * 2.0,
                waves: folds * draw(index, 5).mul_add(0.7, 0.65),
                drift: (draw(index, 6) - 0.5) * 2.0 * sideways,
                scale: draw(index, 7).mul_add(0.6, 0.65),
                alpha: draw(index, 8).mul_add(0.5, 0.5),
            }
        })
        .collect()
}

#[derive(Debug, Clone, Copy)]
struct Wave {
    time: f64,
    amplitude: f64,
    thickness: f64,
    center_y: f64,
    spread: f64,
}

fn sample_x(i: u32) -> f64 {
    f64::mul_add(
        f64::from(i) / f64::from(SAMPLES),
        OVERSCAN.mul_add(2.0, 100.0),
        -OVERSCAN,
    )
}

fn point(command: &str, x: f64, y: f64) -> String {
    format!(
        "{command}{},{}",
        js_number_text(to_fixed(x, 2)),
        js_number_text(to_fixed(y, 2))
    )
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film's ribbons round every step, and the frames have to match it"
)]
fn ribbon_path(ribbon: &Ribbon, wave: Wave) -> String {
    let Wave {
        time,
        amplitude,
        thickness,
        center_y,
        spread,
    } = wave;
    let k = ribbon.waves * PI * 2.0 / (100.0 + OVERSCAN * 2.0);
    let slow = time / SLOW_PERIOD * ribbon.slow_rate * PI * 2.0;
    let fast = time / FAST_PERIOD * ribbon.fast_rate * PI * 2.0;
    let base =
        center_y + ribbon.band * spread + (slow * 0.5 + ribbon.slow_phase).sin() * amplitude * 0.35;
    let (mut top, mut bottom) = (Vec::new(), Vec::new());
    for i in 0..=SAMPLES {
        let u = f64::from(i) / f64::from(SAMPLES);
        let x = -OVERSCAN + u * (100.0 + OVERSCAN * 2.0);
        let fold = (k * (x + ribbon.drift * time) + slow + ribbon.slow_phase).sin() * amplitude
            + (k * 2.3 * x + fast + ribbon.fast_phase).sin() * amplitude * 0.32;
        let taper = (PI * u).sin().powf(1.4);
        let height = thickness * ribbon.scale * (0.3 + 0.7 * taper);
        top.push(base + fold - height);
        bottom.push(base + fold + height * 0.25);
    }
    let forward: Vec<String> = (0..=SAMPLES)
        .zip(&top)
        .map(|(i, y)| point(if i == 0 { "M" } else { "L" }, sample_x(i), *y))
        .collect();
    let back: Vec<String> = (0..=SAMPLES)
        .rev()
        .zip(bottom.iter().rev())
        .map(|(i, y)| point("L", sample_x(i), *y))
        .collect();
    format!("{} {} Z", forward.join(" "), back.join(" "))
}

pub fn draw(props: &AuroraBg, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let n = js_number_text;
    let time = cx.frame() / cx.fps() * props.speed;
    let list = ribbons(
        whole(f64::from(props.ribbon_count)).max(1),
        &props.colors,
        props.seed,
        props.folds,
        props.sideways_drift,
    );
    let first = props.colors.first().map_or("", |c| c.0.as_str());
    let mut layers: Vec<Node> = Vec::new();
    if props.horizon_glow > 0.0 {
        let glow = if props.horizon_glow_color.is_empty() {
            first
        } else {
            &props.horizon_glow_color
        };
        let pulse = 0.15f64.mul_add((time / SLOW_PERIOD * PI).sin(), 0.85);
        layers.push(
            div(with_background(
                absolute_fill(),
                &format!(
                    "radial-gradient(120% 60% at 50% {}%, {glow} 0%, transparent 70%)",
                    n(props.spread.mul_add(0.5, props.center_y))
                ),
            )
            .opacity(0.16 * props.horizon_glow * props.intensity * pulse)
            .mix_blend_mode(BlendMode::Screen))
            .into(),
        );
    }
    let wave = Wave {
        time,
        amplitude: props.amplitude,
        thickness: props.thickness,
        center_y: props.center_y,
        spread: props.spread,
    };
    let defs = list
        .iter()
        .enumerate()
        .fold(SvgElement::new("defs"), |defs, (index, ribbon)| {
            defs.child(
                SvgElement::new("linearGradient")
                    .attr("id", format!("aurora-{index}"))
                    .attr("x1", "0")
                    .attr("y1", "0")
                    .attr("x2", "0")
                    .attr("y2", "1")
                    .child(stop("0%", &ribbon.color, "0"))
                    .child(stop("55%", &ribbon.color, "0.55"))
                    .child(stop("100%", &ribbon.color, "1")),
            )
        });
    let picture = list.iter().enumerate().fold(
        SvgElement::new("svg")
            .attr("viewBox", "0 0 100 100")
            .attr("preserveAspectRatio", "none")
            .child(defs),
        |picture, (index, ribbon)| {
            picture.child(
                SvgElement::new("path")
                    .attr("d", ribbon_path(ribbon, wave))
                    .attr("fill", format!("url(#aurora-{index})"))
                    .attr("opacity", ribbon.alpha * 0.72 * props.intensity)
                    .attr("style", "mix-blend-mode:screen"),
            )
        },
    );
    let mut veil = absolute_fill().filter(vec![Filter::Blur(Length::px(props.blur))]);
    if props.striation > 0.0
        && let Some(gradient) = gradient(&ray_mask(props.striation))
    {
        let spacing = f64::from(props.ray_spacing);
        let mut layer = gradient.layer();
        layer.size = LayerSize::Explicit(Length::px(spacing), Length::percent(100.0));
        layer.position = Position2 {
            x: Length::px(to_fixed((time * RAY_DRIFT) % spacing, 2)),
            y: Length::px(0.0),
        };
        veil = veil.mask(vec![MaskLayer {
            layer,
            mode: MaskMode::Alpha,
        }]);
    }
    layers.push(
        div(veil)
            .child(svg(
                picture,
                Style::default()
                    .width(Length::percent(100.0))
                    .height(Length::percent(100.0)),
            ))
            .into(),
    );
    layers.push(
        div(with_background(
            absolute_fill(),
            &format!(
                "radial-gradient(circle at 50% 45%, transparent 40%, rgba(0,0,0,{}) 100%)",
                n(props.vignette)
            ),
        ))
        .into(),
    );
    Ok(
        div(with_background(absolute_fill(), &props.background_color).overflow(Overflow::Hidden))
            .children(layers)
            .into(),
    )
}

fn stop(offset: &str, color: &str, opacity: &str) -> SvgElement {
    SvgElement::new("stop")
        .attr("offset", offset)
        .attr("stop-color", color)
        .attr("stop-opacity", opacity)
}
