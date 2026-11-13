use opsis_macros::{choice, settings};
use opsis_scene::keyed::round_to;
use opsis_scene::node::{Node, SvgElement, div, svg};
use opsis_scene::random::{js_number_text, random};
use opsis_scene::style::{Display, Position, Style};
use opsis_scene::values::Length;
use opsis_types::looks::NodeLook;
use serde_json::json;

use crate::elements::anim::{ease_out_quad, smoothstep};
use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
pub enum SpeedLinesMode {
    Trail,
    Focus,
}

#[settings(export)]
pub struct SpeedLines {
    #[setting("Mode", default = SpeedLinesMode::Trail)]
    pub mode: SpeedLinesMode,
    #[setting("Lines", default = 12, range(min = 3, max = 32), step = 1)]
    pub count: u32,
    #[setting("Stroke weight", default = 2.0, range(min = 1, max = 8), step = 0.5)]
    pub weight: f64,
    #[setting("Spread", default = 96, range(min = 16, max = 320), step = 4)]
    pub spread: u32,
    #[setting(
        "Travel frames",
        default = 14,
        range(min = 6, max = 40),
        step = 1,
        advanced
    )]
    pub duration: u32,
    #[setting(
        "Bounce",
        default = 0.5,
        range(min = 0, max = 1),
        step = 0.05,
        advanced
    )]
    pub bounce: f64,
    #[setting("Point x", default = 0.5, range(min = 0, max = 1), step = 0.01)]
    pub x: f64,
    #[setting("Point y", default = 0.5, range(min = 0, max = 1), step = 0.01)]
    pub y: f64,
    #[setting("Delay", range(min = 0, max = 60), step = 1, advanced)]
    pub delay: u32,
    #[setting("Line color", default = "#ffffff", check = colour)]
    pub color: String,
    #[setting("Seed", default = 1, range(min = 1), step = 1, advanced)]
    pub seed: u32,
    #[setting(
        "Speed",
        default = 1.0,
        range(min = 0.25, max = 4),
        step = 0.25,
        advanced
    )]
    pub speed: f64,
    #[setting("Start x", default = -0.25, range(min = -0.5, max = 1.5))]
    pub from_x: f64,
    #[setting("Start y", default = 0.5, range(min = -0.5, max = 1.5))]
    pub from_y: f64,
    #[setting(
        "Hold on the point",
        default = 0.2667,
        range(min = 0.1, max = 2),
        step = 0.0333,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub hold: f64,
    #[setting(
        "Streak length",
        default = 1.6,
        range(min = 0.5, max = 4),
        unit = "×",
        advanced
    )]
    pub streak_length: f64,
}

pub fn definition() -> ElementMeta {
    meta::<SpeedLines>(
            element(
                ElementKind::Component,
                "speed-lines",
                "Speed lines",
                "Streaks shoot across the frame toward a point and die as they land, or radial focus lines rush in from the edges and snap onto one point for an impact frame. Lay it over a clip with Layers.",
                ElementCategory::Effect,
                55,
            )
                .authored_clock()
                .fixed("children", json!(null))
                .look(NodeLook::of("lucide:chevrons-right")),
        )
}

const REFERENCE: f64 = 720.0;
const LANDING: f64 = 0.3;
const CARRY: f64 = 0.3;
const ECHO: f64 = 0.2;
const SOFTEN: f64 = 1.2;
const SETTLE: f64 = 6.0;
const SPAWN_START: f64 = 0.04;
const SPAWN_END: f64 = 0.8;
const LIFE: f64 = 0.3;
const MIN_LIFE: f64 = 3.0;
const DRAW: f64 = 0.4;
const ERASE: f64 = 0.35;
const DRIFT: f64 = 0.15;
const SETBACK: f64 = 0.35;
const SLOW: f64 = 5.0;
const FAST: f64 = 14.0;
const IMPACT: f64 = 3.0;
const RUSH_JITTER: f64 = 1.0;
const RELAX: f64 = 0.12;
const RELAX_RATE: f64 = 1.5;
const RELEASE_JITTER: f64 = 1.5;
const RETRACT: f64 = 3.5;
const ANGLE_JITTER: f64 = 0.6;
const RADIUS_JITTER: f64 = 0.6;
const MAX_COUNT: u32 = 64;

fn golden() -> f64 {
    (5.0f64.sqrt() - 1.0) / 2.0
}

#[derive(Debug, Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
}

struct Lines {
    focus: bool,
    width: f64,
    height: f64,
    unit: f64,
    from: Point,
    to: Point,
    x: f64,
    y: f64,
    count: u32,
    weight: f64,
    spread: f64,
    duration: f64,
    bounce: f64,
    seed: u32,
    delay: f64,
    speed: f64,
    release: f64,
    shutter: f64,
}

impl Lines {
    fn of(props: &SpeedLines, width: f64, height: f64) -> Self {
        let height = height.max(1.0);
        Self {
            focus: props.mode == SpeedLinesMode::Focus,
            width: width.max(1.0),
            height,
            unit: height / REFERENCE,
            from: Point {
                x: props.from_x,
                y: props.from_y,
            },
            to: Point {
                x: props.x,
                y: props.y,
            },
            x: props.x,
            y: props.y,
            count: props.count.clamp(1, MAX_COUNT),
            weight: props.weight.clamp(0.5, 12.0),
            spread: f64::from(props.spread).clamp(8.0, 480.0),
            duration: f64::from(props.duration).clamp(4.0, 120.0),
            bounce: props.bounce.clamp(0.0, 1.0),
            seed: props.seed,
            delay: f64::from(props.delay).max(0.0),
            speed: props.speed.max(0.0),
            release: props.hold.max(0.0),
            shutter: props.streak_length.max(0.0),
        }
    }

    fn time(&self, frame: f64) -> f64 {
        (frame - self.delay).max(0.0) * self.speed
    }

    fn direction(&self) -> (f64, f64, f64) {
        let dx = (self.to.x - self.from.x) * self.width;
        let dy = (self.to.y - self.from.y) * self.height;
        let distance = dx.hypot(dy);
        if distance < 1e-6 {
            (1.0, 0.0, 0.0)
        } else {
            (dx / distance, dy / distance, distance)
        }
    }

    fn key(&self, index: u32, name: &str) -> f64 {
        random(format!("speed-lines-{}-{index}-{name}", self.seed))
    }
}

struct Spring {
    kappa: f64,
    gain: f64,
    contact: f64,
    decay: f64,
    ring: f64,
    settle: f64,
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film computes the spring with a separate multiply and add"
)]
fn spring_of(bounce: f64) -> Spring {
    let landing = CARRY.mul_add(bounce, LANDING);
    let kappa = f64::midpoint(1.0, (1.0 - landing).sqrt());
    let gain = 1.0 / (0.5 - kappa / 3.0);
    let echo = ECHO * bounce;
    let decrement = -echo.ln();
    let damping = if echo > 0.0 {
        decrement / std::f64::consts::PI.hypot(decrement)
    } else {
        1.0
    };
    let natural = 1.0 / SOFTEN.mul_add(bounce, 1.0);
    let decay = damping * natural;
    Spring {
        kappa,
        gain,
        contact: gain * (1.0 - kappa),
        decay,
        ring: natural * (1.0 - damping * damping).sqrt(),
        settle: (SETTLE / decay).ceil(),
    }
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film eases the travel with separate multiplies and adds"
)]
fn ease_of(time: f64, travel: f64, bounce: f64) -> f64 {
    if time <= 0.0 {
        return 0.0;
    }
    let s = spring_of(bounce);
    if time < travel {
        let w = time / travel;
        return s.gain * ((w * w) / 2.0 - (s.kappa * w * w * w) / 3.0);
    }
    let since = time - travel;
    if since >= s.settle {
        return 1.0;
    }
    let taper = smoothstep((2.0 * since) / s.settle - 1.0);
    let wave = if s.ring > 1e-9 {
        (s.ring * since).sin() / s.ring
    } else {
        since
    };
    let envelope = (-s.decay * since).exp() * (1.0 - taper);
    1.0 + (s.contact / travel) * wave * envelope
}

#[derive(Debug, Clone, Copy)]
struct Placed {
    x: f64,
    y: f64,
    progress: f64,
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film places the point with separate multiplies and adds"
)]
fn position_at(time: f64, lines: &Lines) -> Placed {
    if lines.focus {
        return Placed {
            x: lines.x * lines.width,
            y: lines.y * lines.height,
            progress: 1.0,
        };
    }
    let progress = ease_of(time, lines.duration, lines.bounce);
    let start_x = lines.from.x * lines.width;
    let start_y = lines.from.y * lines.height;
    Placed {
        x: start_x + (lines.to.x * lines.width - start_x) * progress,
        y: start_y + (lines.to.y * lines.height - start_y) * progress,
        progress,
    }
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film computes the speed along the path with a separate multiply and add"
)]
fn forward_speed(frame: f64, lines: &Lines) -> f64 {
    let now = position_at(lines.time(frame), lines);
    let before = position_at(lines.time(frame - 1.0), lines);
    let (dx, dy, _) = lines.direction();
    (now.x - before.x) * dx + (now.y - before.y) * dy
}

struct Segment {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film lays out the streaks with separate multiplies and adds"
)]
fn streaks(frame: f64, lines: &Lines) -> Vec<Segment> {
    if lines.focus {
        return Vec::new();
    }
    let time = lines.time(frame);
    let (dx, dy, distance) = lines.direction();
    if distance == 0.0 || time <= 0.0 {
        return Vec::new();
    }
    let forward = forward_speed(frame, lines);
    let unit = lines.unit;
    let fast = smoothstep((forward - SLOW * unit) / ((FAST - SLOW) * unit));
    let gate = fast * (1.0 - smoothstep(time - lines.duration));
    if gate <= 0.0 {
        return Vec::new();
    }
    let center = position_at(time, lines);
    let progress = time / lines.duration;
    let life = MIN_LIFE.max(LIFE * lines.duration) / lines.duration;
    let minimum = 0.5 * lines.weight * unit;
    let band = lines.spread * unit;
    let phase = random(format!("speed-lines-{}-lanes", lines.seed));
    let count = f64::from(lines.count);
    let mut out = Vec::new();
    for index in 0..lines.count {
        let i = f64::from(index);
        let jitter = (lines.key(index, "lane") - 0.5) / (2.0 * count);
        let lane = phase + golden() * i + jitter;
        let lane = lane - lane.floor();
        let slot = (i + lines.key(index, "spawn")) / count;
        let spawn = SPAWN_START + (SPAWN_END - SPAWN_START) * slot;
        let offset = band * (lane - 0.5);
        let factor = 0.5 + 0.5 * lines.key(index, "length");
        let setback = SETBACK * band * lines.key(index, "setback");
        let age = (progress - spawn) / life;
        if age <= 0.0 || age >= 1.0 {
            continue;
        }
        let reach = lines.shutter * factor * forward * gate;
        let drawn = ease_out_quad(age / DRAW);
        let erased = smoothstep((age - ERASE) / (1.0 - ERASE));
        let length = reach * (drawn - erased);
        if length < minimum {
            continue;
        }
        let spawned = ease_of(spawn * lines.duration, lines.duration, lines.bounce);
        let since = ((center.progress - spawned) * distance).max(0.0);
        let base = setback + DRIFT * since;
        let front = base + reach * erased;
        let back = base + reach * drawn;
        let lane_x = center.x - dy * offset;
        let lane_y = center.y + dx * offset;
        out.push(Segment {
            x1: round_to(lane_x - dx * front, 2),
            y1: round_to(lane_y - dy * front, 2),
            x2: round_to(lane_x - dx * back, 2),
            y2: round_to(lane_y - dy * back, 2),
        });
    }
    out
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film moves each ray with separate multiplies and adds"
)]
fn ray_radius(t: f64, rest: f64, outer: f64, start: f64, release: f64) -> f64 {
    if t <= start {
        return outer;
    }
    if t < IMPACT {
        let q = (t - start) / (IMPACT - start);
        return outer + (rest - outer) * q * q;
    }
    let settle = |at: f64| rest * (1.0 + RELAX * (1.0 - (-(at - IMPACT) / RELAX_RATE).exp()));
    if t < release {
        return settle(t);
    }
    let held = settle(release);
    let q = ((t - release) / RETRACT).clamp(0.0, 1.0);
    outer - (outer - held) * (1.0 - q * q)
}

fn along(position: f64, size: f64, step: f64) -> f64 {
    if step > 1e-9 {
        (size - position) / step
    } else if step < -1e-9 {
        -position / step
    } else {
        f64::INFINITY
    }
}

fn exit_distance(center: Point, cos: f64, sin: f64, width: f64, height: f64) -> f64 {
    let inside = center.x >= 0.0 && center.x <= width && center.y >= 0.0 && center.y <= height;
    if !inside {
        let reach_x = center.x.abs().max((width - center.x).abs());
        let reach_y = center.y.abs().max((height - center.y).abs());
        return reach_x.hypot(reach_y);
    }
    along(center.x, width, cos).min(along(center.y, height, sin))
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film aims each ray with separate multiplies and adds"
)]
fn rays(frame: f64, lines: &Lines) -> Vec<Segment> {
    if !lines.focus {
        return Vec::new();
    }
    let time = lines.time(frame);
    if time <= 0.0 || time >= lines.release + RELEASE_JITTER + RETRACT {
        return Vec::new();
    }
    let unit = lines.unit;
    let center = Point {
        x: lines.x * lines.width,
        y: lines.y * lines.height,
    };
    let minimum = 0.5 * lines.weight * unit;
    let rotation = random(format!("speed-lines-{}-rotation", lines.seed));
    let count = f64::from(lines.count);
    let mut out = Vec::new();
    for index in 0..lines.count {
        let jitter = ANGLE_JITTER * (lines.key(index, "angle") - 0.5);
        let angle = 2.0 * std::f64::consts::PI * (rotation + (f64::from(index) + jitter) / count);
        let (sin, cos) = angle.sin_cos();
        let reach = RADIUS_JITTER * lines.key(index, "radius");
        let rest = lines.spread * unit * (1.0 + reach);
        let outer =
            exit_distance(center, cos, sin, lines.width, lines.height) + lines.weight * unit;
        let inner = ray_radius(
            time,
            rest,
            outer,
            RUSH_JITTER * lines.key(index, "rush"),
            lines.release + RELEASE_JITTER * lines.key(index, "release"),
        );
        if outer - inner < minimum {
            continue;
        }
        out.push(Segment {
            x1: round_to(center.x + cos * inner, 2),
            y1: round_to(center.y + sin * inner, 2),
            x2: round_to(center.x + cos * outer, 2),
            y2: round_to(center.y + sin * outer, 2),
        });
    }
    out
}

pub fn draw(props: &SpeedLines, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let (width, height) = (cx.at.stage.width, cx.at.stage.height);
    let lines = Lines::of(props, width, height);
    let frame = cx.authored_frame();
    let n = js_number_text;
    let path: Vec<String> = streaks(frame, &lines)
        .into_iter()
        .chain(rays(frame, &lines))
        .map(|s| format!("M {} {} L {} {}", n(s.x1), n(s.y1), n(s.x2), n(s.y2)))
        .collect();
    let mut root = SvgElement::new("svg")
        .attr("width", "100%")
        .attr("height", "100%")
        .attr("viewBox", format!("0 0 {} {}", n(width), n(height)));
    if !path.is_empty() {
        root = root.child(
            SvgElement::new("path")
                .attr("d", path.join(" "))
                .attr("fill", "none")
                .attr("stroke", props.color.as_str())
                .attr("stroke-width", lines.weight * lines.unit)
                .attr("stroke-linecap", "round"),
        );
    }
    Ok(div(Style::default()
        .position(Position::Absolute)
        .inset(Length::ZERO))
    .child(svg(
        root,
        Style::default()
            .position(Position::Absolute)
            .inset(Length::ZERO)
            .width(Length::percent(100.0))
            .height(Length::percent(100.0))
            .display(Display::Block),
    ))
    .into())
}
