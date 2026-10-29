use opsis_macros::{choice, settings};
use opsis_types::looks::NodeLook;

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
