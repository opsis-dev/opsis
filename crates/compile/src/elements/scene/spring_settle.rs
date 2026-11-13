use opsis_macros::settings;
use opsis_scene::easing::Easing;
use opsis_scene::interpolate::{Interpolation, interpolate};
use opsis_scene::node::{Node, div};
use opsis_scene::random::random;
use opsis_scene::spring::{DEFAULT_SPRING, Spring, SpringConfig, spring};
use opsis_scene::style::Style;
use opsis_scene::timeline::{SceneTracks, absolute_fill};
use opsis_scene::values::Length;
use opsis_types::looks::NodeLook;
use opsis_types::numbers::float;

use crate::elements::anim::clamped;
use crate::elements::draw::{Cx, DrawFailure, css_color};
use crate::elements::entry::{element, meta};
use crate::elements::oklch_mix::interpolate_colors;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct SpringSettle {
    #[setting("Enter scale", default = 1.24, range(min = 1, max = 1.8), step = 0.02)]
    pub enter_scale: f64,
    #[setting(
        "Time between items",
        default = 3.0,
        range(min = 0, max = 10),
        step = 0.5,
        unit = "frames",
        help = "How long each item in a scene waits after the one before it."
    )]
    pub enter_stagger: f64,
    #[setting(
        "Exit scale",
        default = 0.84,
        range(min = 0.5, max = 1),
        step = 0.02,
        advanced
    )]
    pub exit_scale: f64,
    #[setting(
        "Exit time",
        default = 6,
        range(min = 2, max = 24),
        step = 1,
        unit = "frames",
        advanced
    )]
    pub exit_frames: u32,
    #[setting(
        "Fade-in time",
        default = 5,
        range(min = 0, max = 30),
        step = 1,
        unit = "frames",
        advanced
    )]
    pub enter_fade_frames: u32,
    #[setting(
        "Spring damping",
        default = 30,
        range(min = 1, max = 100),
        step = 1,
        help = "Lower values bounce more as items land.",
        advanced
    )]
    pub spring_damping: u32,
    #[setting(
        "Spring stiffness",
        default = 320,
        range(min = 10, max = 1000),
        step = 10,
        help = "Higher values land faster.",
        advanced
    )]
    pub spring_stiffness: u32,
    #[setting(
        "Background fade",
        default = 4,
        range(min = 0, max = 30),
        step = 1,
        unit = "frames",
        advanced
    )]
    pub bg_fade_frames: u32,
    #[setting("Loop", help = "Start again from the first scene after the last.")]
    pub r#loop: bool,
    #[setting(
        "Uneven timing",
        default = 1,
        range(min = 0, max = 5),
        step = 1,
        unit = "frames",
        advanced
    )]
    pub enter_stagger_jitter: u32,
}

pub fn definition() -> ElementMeta {
    meta::<SpringSettle>(
            element(
                ElementKind::Sequencer,
                "spring-settle",
                "Spring settle",
                "Plays clips as scenes: each scene shrinks away, the stage holds empty for a beat, and the next one lands. Connect Layers to make a scene whose layers land one after another.",
                ElementCategory::Scene,
                20,
            )
                .look(NodeLook::of("tabler:carousel-vertical")),
        )
}

const GAP_FRAMES: f64 = 1.0;

const fn default_gap() -> f64 {
    GAP_FRAMES
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sequenced {
    #[serde(flatten)]
    pub settings: SpringSettle,
    #[serde(default = "default_gap")]
    pub gap_frames: f64,
}
const STAGGER_POWER: f64 = 0.8;
const SPRING_MASS: f64 = 1.0;
const EXIT_POWER: i32 = 5;

fn stagger(index: usize, base: f64, jitter: f64) -> f64 {
    float(index)
        .powf(STAGGER_POWER)
        .mul_add(
            base,
            (random(format!("stagger-{index}")) - 0.5) * 2.0 * jitter,
        )
        .max(0.0)
}

fn item(
    props: &SpringSettle,
    index: usize,
    local: f64,
    fps: f64,
    children: Vec<Node>,
) -> Result<Node, DrawFailure> {
    let entered = local
        - stagger(
            index,
            props.enter_stagger,
            f64::from(props.enter_stagger_jitter),
        );
    let progress = spring(&Spring {
        config: SpringConfig {
            damping: f64::from(props.spring_damping),
            stiffness: f64::from(props.spring_stiffness),
            mass: SPRING_MASS,
            ..DEFAULT_SPRING
        },
        ..Spring::at(entered, fps)
    })?;
    let grow = interpolate(
        progress,
        &[0.0, 1.0],
        &[props.enter_scale, 1.0],
        &Interpolation::default(),
    )?;
    let fade = interpolate(
        entered,
        &[0.0, f64::from(props.enter_fade_frames).max(0.001)],
        &[0.0, 1.0],
        &Interpolation {
            easing: Some(Easing::ease_out(Easing::Quad)),
            ..Interpolation::clamped()
        },
    )?;
    Ok(div(Style::default()
        .position(opsis_scene::style::Position::Absolute)
        .inset(Length::ZERO)
        .scale(grow, grow)
        .opacity(fade))
    .children(children)
    .into())
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film shrinks the leaving scene with a separate multiply and add"
)]
fn exit_style(props: &SpringSettle, local: f64, dwell: f64) -> Result<Style, DrawFailure> {
    let exit = f64::from(props.exit_frames);
    let t = clamped(local, [dwell - exit, dwell], [0.0, 1.0])?;
    if t <= 0.0 {
        return Ok(absolute_fill());
    }
    let eased = t.powi(EXIT_POWER);
    let shrink = 1.0 - (1.0 - props.exit_scale) * eased;
    Ok(absolute_fill().scale(shrink, shrink).opacity(1.0 - eased))
}

pub fn scenes(
    sequenced: &Sequenced,
    list: &dyn SceneTracks,
    cx: &Cx<'_>,
) -> Result<Node, DrawFailure> {
    let props = &sequenced.settings;
    let gap = sequenced.gap_frames;
    let count = list.len();
    if count == 0 {
        return Ok(div(Style::default()).into());
    }
    let (frame, fps) = (cx.frame(), cx.fps());
    let lens: Vec<f64> = (0..count)
        .map(|i| f64::from(list.duration_in_frames(i)) + gap)
        .collect();
    let mut w = frame;
    let mut index = 0;
    while index < count - 1 && w >= lens[index] {
        w -= lens[index];
        index += 1;
    }
    let local = w.min(lens[index] - 1.0);
    let dwell = f64::from(list.duration_in_frames(index));
    let slot_start = frame - local;
    let half = f64::from(props.bg_fade_frames).max(0.001) / 2.0;
    let current = list.background(index);
    let previous = index.checked_sub(1).and_then(|i| list.background(i));
    let next = if index + 1 < count {
        list.background(index + 1)
    } else {
        None
    };
    let mut ground = current.map(str::to_owned);
    if let (Some(before), Some(now)) = (previous, current) {
        let d = frame - (slot_start - gap / 2.0);
        if d.abs() <= half {
            ground = Some(interpolate_colors(d, &[-half, half], &[before, now])?);
        }
    }
    if let (Some(after), Some(now)) = (next, current)
        && ground.as_deref() == Some(now)
    {
        let d = frame - (slot_start + dwell + gap / 2.0);
        if d.abs() <= half {
            ground = Some(interpolate_colors(d, &[-half, half], &[now, after])?);
        }
    }
    let mut style = absolute_fill();
    if let Some(color) = &ground {
        style = style.background_color(css_color(color));
    }
    let mut root = div(style);
    if local < dwell {
        let mut items = Vec::new();
        for track in 0..list.track_count(index) {
            items.push(item(
                props,
                track,
                local,
                fps,
                list.draw(index, track, cx.at)?,
            )?);
        }
        root = root.child(div(exit_style(props, local, dwell)?).children(items));
    }
    Ok(root.into())
}
