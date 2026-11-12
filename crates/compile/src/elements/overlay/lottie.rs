use opsis_macros::{choice, settings};
use opsis_scene::measure::AnimationTiming;
use opsis_scene::node::{Element, Kind, LottieFrame, Node, div};
use opsis_scene::paint::AssetRef;
use opsis_scene::style::{ObjectFit, Style};
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::{Length, Num, Position2};
use opsis_types::looks::NodeLook;

use crate::elements::backgrounds::backgrounded;
use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::shared::TextAlign;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
pub enum AnimationFit {
    #[label("Show all")]
    Contain,
    #[label("Fill the frame")]
    Cover,
}

#[settings(export)]
pub struct Lottie {
    #[setting("Animation file", widget = "asset", accept = ".json,.lottie")]
    pub src: String,
    #[setting("Loop", default = true)]
    pub r#loop: bool,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4), step = 0.25)]
    pub speed: f64,
    #[setting("Play backwards", advanced)]
    pub backwards: bool,
    #[setting(
        "Fit",
        default = AnimationFit::Contain
    )]
    pub fit: AnimationFit,
    #[setting("Background", default = "transparent", advanced)]
    pub background: String,
    #[setting(
        "Align",
        default = TextAlign::Center,
        advanced
    )]
    pub align: TextAlign,
}

pub fn definition() -> ElementMeta {
    meta::<Lottie>(
        element(
            ElementKind::Component,
            "lottie",
            "Animation",
            "Plays an animation file from your project, looped or once, at any speed.",
            ElementCategory::Overlay,
            90,
        )
        .look(NodeLook::of("lucide:shapes")),
    )
}

pub fn animation_frame(timing: AnimationTiming, frame: f64, fps: f64, props: &Lottie) -> f64 {
    let first = timing.in_point;
    let length = (timing.out_point - first).max(1.0);
    let last = length - 1.0;
    let played = frame / fps * props.speed * timing.frame_rate;
    let wrapped = if props.r#loop {
        ((played % length) + length) % length
    } else {
        played.max(0.0)
    };
    let along = wrapped.min(last);
    first + if props.backwards { last - along } else { along }
}

const fn object_position(align: TextAlign) -> Position2 {
    Position2 {
        x: Length::percent(match align {
            TextAlign::Left => 0.0,
            TextAlign::Center => 50.0,
            TextAlign::Right => 100.0,
        }),
        y: Length::percent(50.0),
    }
}

pub fn draw(props: &Lottie, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    if props.src.is_empty() {
        return Ok(div(absolute_fill()).into());
    }
    let timing = cx
        .measure
        .animation(&props.src)
        .ok_or_else(|| DrawFailure::Animation(props.src.clone()))?;
    let frame = animation_frame(timing, cx.frame(), cx.fps(), props);
    let player = Element::new(
        Kind::Lottie(LottieFrame {
            source: AssetRef(props.src.clone()),
            frame: Num(frame),
        }),
        Style::default()
            .width(Length::percent(100.0))
            .height(Length::percent(100.0))
            .object_fit(match props.fit {
                AnimationFit::Contain => ObjectFit::Contain,
                AnimationFit::Cover => ObjectFit::Cover,
            })
            .object_position(object_position(props.align)),
    );
    Ok(backgrounded(&props.background, vec![player.into()]))
}
