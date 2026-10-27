use serde::{Deserialize, Serialize};
use specta::Type;

pub const GLYPH_SETS: [&str; 6] = [
    "lucide",
    "tabler",
    "lucide-lab",
    "hugeicons",
    "iconoir",
    "mynaui",
];

macro_rules! node_motions {
    ($($motion:ident = $name:literal),+ $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
        pub enum NodeMotion {
            $(#[serde(rename = $name)] $motion),+
        }

        impl NodeMotion {
            pub const ALL: &'static [Self] = &[$(Self::$motion),+];
        }
    };
}

node_motions! {
    BlurSwap = "blur-swap",
    Bob = "bob",
    Caret = "caret",
    ClockWipe = "clock-wipe",
    Collapse = "collapse",
    Dissolve = "dissolve",
    Draw = "draw",
    FadeThrough = "fade-through",
    Flare = "flare",
    Flicker = "flicker",
    Flip = "flip",
    Glitch = "glitch",
    Grow = "grow",
    HardCut = "hard-cut",
    Iris = "iris",
    March = "march",
    MaskUp = "mask-up",
    NudgeDown = "nudge-down",
    NudgeRight = "nudge-right",
    NudgeUp = "nudge-up",
    Open = "open",
    Pan = "pan",
    Press = "press",
    Push = "push",
    RackFocus = "rack-focus",
    Ripple = "ripple",
    Rise = "rise",
    Roll = "roll",
    Shimmer = "shimmer",
    Shutter = "shutter",
    SlideThrough = "slide-through",
    Smear = "smear",
    Snap = "snap",
    Stretch = "stretch",
    Track = "track",
    Tracking = "tracking",
    Turn = "turn",
    TurnOver = "turn-over",
    Twist = "twist",
    Whip = "whip",
    WipeReveal = "wipe-reveal",
    ZoomBlur = "zoom-blur",
    ZoomThrough = "zoom-through",
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct NodeLook {
    pub glyph: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub motion: Option<NodeMotion>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub morph: Option<String>,
}

impl NodeLook {
    pub fn of(glyph: &str) -> Self {
        Self {
            glyph: glyph.to_owned(),
            motion: None,
            morph: None,
        }
    }

    #[must_use]
    pub fn moves(mut self, motion: NodeMotion) -> Self {
        self.motion = Some(motion);
        self
    }

    #[must_use]
    pub fn morphs(mut self, glyph: &str) -> Self {
        self.morph = Some(glyph.to_owned());
        self
    }
}
