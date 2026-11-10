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
    Collapse = "collapse",
    Draw = "draw",
    FadeThrough = "fade-through",
    Grow = "grow",
    HardCut = "hard-cut",
    March = "march",
    NudgeDown = "nudge-down",
    NudgeRight = "nudge-right",
    NudgeUp = "nudge-up",
    Pan = "pan",
    Press = "press",
    Ripple = "ripple",
    Roll = "roll",
    Smear = "smear",
    Stretch = "stretch",
    Turn = "turn",
    TurnOver = "turn-over",
    Twist = "twist",
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
    pub const fn moves(mut self, motion: NodeMotion) -> Self {
        self.motion = Some(motion);
        self
    }

    #[must_use]
    pub fn morphs(mut self, glyph: &str) -> Self {
        self.morph = Some(glyph.to_owned());
        self
    }
}
