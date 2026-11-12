use std::f64::consts::PI;

use opsis_macros::settings;
use opsis_scene::easing::Easing;
use opsis_scene::node::{Node, SvgElement, div, svg};
use opsis_scene::random::js_number_text;
use opsis_scene::spring::{Spring, SpringConfig, spring};
use opsis_scene::style::{Display, Overflow, Position, Style};
use opsis_scene::values::Length;
use opsis_types::looks::NodeLook;
use opsis_types::numbers::truncate;

use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::oklch_mix::interpolate_colors;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct RadialBurst {
    #[setting("Segments", default = 8, range(min = 4, max = 16), step = 1)]
    pub segments: u32,
    #[setting("Burst radius", default = 210, range(min = 100, max = 280), step = 5)]
    pub radius: u32,
    #[setting("Thickness", default = 30, range(min = 8, max = 60), step = 1)]
    pub thickness: u32,
    #[setting("Rotation", default = 135, range(min = -360, max = 360), step = 15, advanced)]
    pub rotation: i32,
    #[setting("Intensity", default = 1.0, range(min = 0, max = 1.5), step = 0.1)]
    pub intensity: f64,
    #[setting("Ribbon twist", default = 110, range(min = -180, max = 180), step = 10, advanced)]
    pub twist: i32,
    #[setting(
        "Contour echoes",
        default = 3,
        range(min = 0, max = 5),
        step = 1,
        advanced
    )]
    pub echoes: u32,
    #[setting("Shape color", default = "#ffffff", check = colour)]
    pub color: String,
    #[setting("Accent color", default = "#f4e4a7", check = colour)]
    pub accent_color: String,
    #[setting("Background", default = "#a800b7", check = colour)]
    pub background_color: String,
    #[setting("Loop animation")]
    pub r#loop: bool,
    #[setting(
        "Speed",
        default = 1.0,
        range(min = 0.25, max = 4),
        step = 0.25,
        advanced
    )]
    pub speed: f64,
    #[setting("Echo strength", default = 0.36, range(min = 0, max = 1), advanced)]
    pub echo_opacity: f64,
    #[setting("Centre across", default = 0.5, range(min = 0, max = 1), step = 0.01)]
    pub x: f64,
    #[setting("Centre down", default = 0.5, range(min = 0, max = 1), step = 0.01)]
    pub y: f64,
}

pub fn definition() -> ElementMeta {
    meta::<RadialBurst>(
            element(
                ElementKind::Component,
                "radial-burst",
                "Radial burst",
                "Sculptural ribbons twist into a tilted ring and sweep outward through contour echoes for an expressive product introduction.",
                ElementCategory::Overlay,
                120,
            )
                .authored_clock()
                .look(NodeLook::of("lucide:sparkle")),
        )
}

const BURST_LENGTH: f64 = 114.0;
const RIBBON_SAMPLES: u32 = 36;
const DEFAULT_TWIST: f64 = 110.0;

fn ease_in_out(t: f64) -> f64 {
    Easing::Bezier(0.65, 0.0, 0.25, 1.0).at(t)
}

fn ease_out(t: f64) -> f64 {
    Easing::Bezier(0.16, 1.0, 0.3, 1.0).at(t)
}

fn normalize(value: f64, min: f64, max: f64) -> f64 {
    ((value - min) / (max - min)).clamp(0.0, 1.0)
}

fn lerp(from: f64, to: f64, t: f64) -> f64 {
    (to - from).mul_add(t, from)
}

#[derive(Debug, Clone, Copy)]
struct BurstState {
    frame: f64,
    count: u32,
    unit: f64,
    strength: f64,
    energy: f64,
    plane_scale: f64,
    plane_angle: f64,
    offset_x: f64,
    offset_y: f64,
    dot_radius: f64,
    dot_squash: f64,
    show_segments: bool,
    show_ring: bool,
    segment_opacity: f64,
    radius: f64,
    stroke_width: f64,
    segment_length: f64,
    curl: f64,
    arc_angle: f64,
    rotation: f64,
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film page's JavaScript rounds every step, and the pictures have to match it"
)]
fn burst_state(
    frame: f64,
    props: &RadialBurst,
    (width, height): (f64, f64),
) -> Result<BurstState, DrawFailure> {
    let (w, h) = (width.max(1.0), height.max(1.0));
    let unit = w.min(h) / 720.0;
    let local = frame.max(0.0);
    let count = f64::from(props.segments).clamp(4.0, 16.0).round();
    let burst_radius = f64::from(props.radius).clamp(100.0, 280.0) * unit;
    let stroke = f64::from(props.thickness).clamp(8.0, 60.0) * unit;
    let ring_radius = burst_radius * 0.68;
    let assemble = ease_in_out(normalize(local, 44.0, 76.0));
    let curl = ease_in_out(normalize(local, 48.0, 78.0));
    let close = ease_in_out(normalize(local, 72.0, 84.0));
    let anticipate = ease_in_out(normalize(local, 86.0, 96.0));
    let exit = normalize(local, 96.0, BURST_LENGTH).powf(2.6);
    let launch = spring(&Spring {
        config: SpringConfig {
            damping: 13.0,
            stiffness: 170.0,
            mass: 0.8,
            overshoot_clamping: false,
        },
        duration_in_frames: Some(26.0),
        ..Spring::at((local - 18.0).max(0.0), 30.0)
    })?;
    let pop = spring(&Spring {
        config: SpringConfig {
            damping: 11.0,
            stiffness: 180.0,
            mass: 0.65,
            overshoot_clamping: false,
        },
        duration_in_frames: Some(16.0),
        ..Spring::at((local - 2.0).max(0.0), 30.0)
    })?;
    let ring_stroke = lerp(stroke * 0.8, stroke * 1.1, anticipate);
    let exit_stroke = lerp(ring_stroke, stroke * 2.2, exit);
    let clear_radius = w.hypot(h) / 2.0 + stroke * 2.2 + 2.0;
    let orbit_radius = lerp(
        lerp(28.0 * unit, burst_radius, launch),
        ring_radius,
        assemble,
    );
    let final_radius = lerp(ring_radius, ring_radius * 0.84, anticipate);
    let strength = props.intensity.clamp(0.0, 1.5);
    let energy = (PI * normalize(local, 18.0, 84.0)).sin();
    let tilt =
        ease_out(normalize(local, 18.0, 48.0)) * (1.0 - ease_in_out(normalize(local, 96.0, 114.0)));
    let travel = normalize(local, 18.0, 114.0);
    Ok(BurstState {
        frame: local,
        count: truncate(count),
        unit,
        strength,
        energy: energy.max(0.0) * strength,
        plane_scale: 1.0 - tilt * 0.46 * strength.min(1.5),
        plane_angle: (travel * PI * 1.6).sin() * 32.0 * strength,
        offset_x: (PI * travel).sin() * 65.0 * unit * strength,
        offset_y: -(PI * travel).sin() * 32.0 * unit * strength,
        dot_radius: 37.0 * unit * pop * (1.0 - ease_out(normalize(local, 18.0, 26.0))),
        dot_squash: 1.0 - (PI * normalize(local, 2.0, 18.0)).sin() * 0.16,
        show_segments: (18.0..84.0).contains(&local),
        show_ring: (84.0..BURST_LENGTH).contains(&local),
        segment_opacity: ease_out(normalize(local, 18.0, 23.0)),
        radius: if local < 84.0 {
            orbit_radius
        } else {
            lerp(final_radius, clear_radius, exit)
        },
        stroke_width: if local < 84.0 {
            lerp(stroke, stroke * 0.8, assemble)
        } else {
            exit_stroke
        },
        segment_length: lerp(
            12.0 * unit,
            (82.0 * unit).min(burst_radius * 0.42),
            ease_out(normalize(local, 18.0, 36.0)),
        ),
        curl,
        arc_angle: PI * 2.0 / count * lerp(0.62, 1.002, close),
        rotation: f64::from(props.rotation) * ease_in_out(normalize(local, 34.0, 84.0)),
    })
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film page's JavaScript rounds every step, and the pictures have to match it"
)]
fn segment_points(radius: f64, segment_length: f64, arc_angle: f64, curl: f64) -> [[f64; 2]; 4] {
    let half = arc_angle / 2.0;
    let (x, y) = (radius * half.cos(), radius * half.sin());
    let tangent = 4.0 / 3.0 * (arc_angle / 4.0).tan() * radius;
    let arc = [
        [x, -y],
        [x + tangent * half.sin(), -y + tangent * half.cos()],
        [x + tangent * half.sin(), y - tangent * half.cos()],
        [x, y],
    ];
    let angle = curl * PI / 2.0;
    let offsets = [-0.5, -1.0 / 6.0, 1.0 / 6.0, 0.5];
    let mut out = [[0.0; 2]; 4];
    for (i, offset) in offsets.iter().enumerate() {
        let px = radius + segment_length * offset * angle.cos();
        let py = segment_length * offset * angle.sin();
        out[i] = [lerp(px, arc[i][0], curl), lerp(py, arc[i][1], curl)];
    }
    out
}

fn fixed3(value: f64) -> String {
    format!("{value:.3}")
}

fn segment_path(state: &BurstState) -> String {
    let points = segment_points(
        state.radius,
        state.segment_length,
        state.arc_angle,
        state.curl,
    )
    .map(|p| format!("{} {}", fixed3(p[0]), fixed3(p[1])));
    format!(
        "M {} C {} {} {}",
        points[0], points[1], points[2], points[3]
    )
}

#[expect(
    clippy::suboptimal_flops,
    clippy::many_single_char_names,
    reason = "the film page's JavaScript rounds every step and names its curve terms so, and the pictures have to match it"
)]
fn ribbon_path(state: &BurstState, index: u32, twist: f64) -> String {
    let energy = state.energy;
    let i = f64::from(index);
    let mut p = segment_points(
        state.radius * (1.0 + (i * 2.4).sin() * energy * 0.15),
        state.segment_length * (1.0 + energy * 2.8),
        state.arc_angle,
        state.curl,
    );
    let bend = twist.clamp(-180.0, 180.0) / DEFAULT_TWIST * energy * 145.0 * state.unit;
    p[1][1] += bend * 0.55;
    p[2][1] += bend;
    p[3][1] += bend * 1.25;
    p[2][0] += energy * 38.0 * state.unit;
    let mut upper = Vec::new();
    let mut lower = Vec::new();
    let (mut first_width, mut last_width) = (0.0, 0.0);
    for sample in 0..=RIBBON_SAMPLES {
        let t = f64::from(sample) / f64::from(RIBBON_SAMPLES);
        let q = 1.0 - t;
        let at = |k: usize| {
            q.powi(3) * p[0][k]
                + 3.0 * q * q * t * p[1][k]
                + 3.0 * q * t * t * p[2][k]
                + t.powi(3) * p[3][k]
        };
        let d = |k: usize| {
            3.0 * q * q * (p[1][k] - p[0][k])
                + 6.0 * q * t * (p[2][k] - p[1][k])
                + 3.0 * t * t * (p[3][k] - p[2][k])
        };
        let dd = |k: usize| {
            6.0 * q * (p[2][k] - 2.0 * p[1][k] + p[0][k])
                + 6.0 * t * (p[3][k] - 2.0 * p[2][k] + p[1][k])
        };
        let (x, y, dx, dy) = (at(0), at(1), d(0), d(1));
        let length = dx.hypot(dy).max(0.001);
        let curvature = (dx * dd(1) - dy * dd(0)).abs() / length.powi(3);
        let swell = (PI * t).sin().powf(1.4);
        let half = (state.stroke_width / 2.0 * (1.0 + energy * (2.4 * swell - 0.65 * t)))
            .min(0.72 / curvature.max(0.000_01));
        if sample == 0 {
            first_width = half;
        }
        if sample == RIBBON_SAMPLES {
            last_width = half;
        }
        upper.push(format!(
            "{} {}",
            fixed3(x - dy / length * half),
            fixed3(y + dx / length * half)
        ));
        lower.push(format!(
            "{} {}",
            fixed3(x + dy / length * half),
            fixed3(y - dx / length * half)
        ));
    }
    let back: Vec<&String> = lower[..lower.len() - 1].iter().rev().collect();
    format!(
        "M {} A {} {} 0 0 0 {} L {} A {} {} 0 0 0 {} Z",
        upper.join(" L "),
        js_number_text(last_width),
        js_number_text(last_width),
        lower[lower.len() - 1],
        back.iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(" L "),
        js_number_text(first_width),
        js_number_text(first_width),
        upper[0]
    )
}

fn plane(pose: &BurstState) -> String {
    format!(
        "translate({} {}) rotate({}) scale(1 {})",
        js_number_text(pose.offset_x),
        js_number_text(pose.offset_y),
        js_number_text(pose.plane_angle),
        js_number_text(pose.plane_scale)
    )
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film page's JavaScript rounds every step, and the pictures have to match it"
)]
fn ribbons(
    props: &RadialBurst,
    pose: &BurstState,
    outline: bool,
) -> Result<Vec<SvgElement>, DrawFailure> {
    let mut out = Vec::new();
    for index in 0..pose.count {
        let i = f64::from(index);
        let turn = i * 360.0 / f64::from(pose.count) + pose.rotation - 90.0
            + (i * 1.7).sin() * pose.energy * 16.0;
        let fill = if outline {
            "none".to_owned()
        } else if index % 3 == 0 {
            interpolate_colors(
                pose.energy.clamp(0.0, 1.0),
                &[0.0, 1.0],
                &[props.color.as_str(), props.accent_color.as_str()],
            )?
        } else {
            props.color.clone()
        };
        out.push(
            SvgElement::new("path")
                .attr("d", ribbon_path(pose, index, f64::from(props.twist)))
                .attr("transform", format!("rotate({})", js_number_text(turn)))
                .attr("fill", fill)
                .attr(
                    "stroke",
                    if outline {
                        props.color.as_str()
                    } else {
                        "none"
                    },
                )
                .attr("stroke-width", 1.5 * pose.unit),
        );
    }
    Ok(out)
}

fn group(children: Vec<SvgElement>) -> SvgElement {
    children
        .into_iter()
        .fold(SvgElement::new("g"), SvgElement::child)
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film page's JavaScript rounds every step, and the pictures have to match it"
)]
fn echoes(
    props: &RadialBurst,
    state: &BurstState,
    size: (f64, f64),
) -> Result<Vec<SvgElement>, DrawFailure> {
    let mut out = Vec::new();
    if !(state.strength > 0.0 && state.frame >= 18.0 && state.frame < BURST_LENGTH) {
        return Ok(out);
    }
    let count = f64::from(props.echoes).clamp(0.0, 5.0).round();
    let total: u32 = truncate(count);
    for index in 0..total {
        let i = f64::from(index);
        let lag = (count - i) * 2.2;
        let echo = burst_state((state.frame - lag).max(18.0), props, size)?;
        let opacity = props.echo_opacity
            * (1.0 - i / (count + 1.0))
            * state.strength.min(1.0)
            * (1.0 - ease_out(normalize(state.frame, 107.0, 114.0)));
        let mut g = SvgElement::new("g")
            .attr("transform", plane(&echo))
            .attr("opacity", opacity);
        if echo.show_segments {
            for path in ribbons(props, &echo, true)? {
                g = g.child(path);
            }
        } else if echo.show_ring {
            g = g.child(
                SvgElement::new("circle")
                    .attr("r", echo.radius)
                    .attr("fill", "none")
                    .attr(
                        "stroke",
                        if index % 2 == 1 {
                            props.color.as_str()
                        } else {
                            props.accent_color.as_str()
                        },
                    )
                    .attr("stroke-width", echo.stroke_width * (0.22 + i * 0.07)),
            );
        }
        out.push(g);
    }
    Ok(out)
}

pub fn draw(props: &RadialBurst, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let (width, height) = (cx.at.stage.width, cx.at.stage.height);
    let time = cx.authored_frame() * (30.0 / Cx::authored_fps()) * props.speed.max(0.0);
    let state = burst_state(time, props, (width, height))?;
    let mut main = SvgElement::new("g").attr("transform", plane(&state));
    if state.dot_radius > 0.001 {
        main = main.child(
            SvgElement::new("circle")
                .attr("r", state.dot_radius)
                .attr("fill", props.color.as_str())
                .attr(
                    "transform",
                    format!(
                        "scale({} {})",
                        js_number_text(1.0 / state.dot_squash),
                        js_number_text(state.dot_squash)
                    ),
                ),
        );
    }
    if state.show_segments && state.strength > 0.0 {
        main = main
            .child(group(ribbons(props, &state, false)?).attr("opacity", state.segment_opacity));
    } else if state.show_segments {
        let path = segment_path(&state);
        let mut g = SvgElement::new("g")
            .attr("fill", "none")
            .attr("stroke", props.color.as_str())
            .attr("stroke-width", state.stroke_width)
            .attr("stroke-linecap", "round")
            .attr("opacity", state.segment_opacity)
            .attr(
                "transform",
                format!("rotate({})", js_number_text(state.rotation - 90.0)),
            );
        for index in 0..state.count {
            g = g.child(SvgElement::new("path").attr("d", path.as_str()).attr(
                "transform",
                format!(
                    "rotate({})",
                    js_number_text(f64::from(index) * 360.0 / f64::from(state.count))
                ),
            ));
        }
        main = main.child(g);
    }
    if state.show_ring {
        main = main.child(
            SvgElement::new("circle")
                .attr("r", state.radius)
                .attr("fill", "none")
                .attr("stroke", props.color.as_str())
                .attr("stroke-width", state.stroke_width),
        );
    }
    let mut placed = SvgElement::new("g").attr(
        "transform",
        format!(
            "translate({} {})",
            js_number_text(width * props.x),
            js_number_text(height * props.y)
        ),
    );
    for echo in echoes(props, &state, (width, height))? {
        placed = placed.child(echo);
    }
    let root = SvgElement::new("svg")
        .attr("width", "100%")
        .attr("height", "100%")
        .attr(
            "viewBox",
            format!("0 0 {} {}", js_number_text(width), js_number_text(height)),
        )
        .child(placed.child(main));
    Ok(div(Style::default()
        .position(Position::Absolute)
        .inset(Length::ZERO)
        .overflow(Overflow::Hidden)
        .background_color(cx.color(&props.background_color)))
    .child(svg(root, Style::default().display(Display::Block)))
    .into())
}
