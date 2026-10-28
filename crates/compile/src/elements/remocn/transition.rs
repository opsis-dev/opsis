use opsis_types::looks::{NodeLook, NodeMotion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::choices::{
    DitherShape, GrainDissolveShape, ParticleDissolveDirection, WhipDirection,
};
use crate::elements::core::Colour;
use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct WhipPan {
    #[default(WhipDirection::Left)]
    #[schemars(description = "Direction")]
    pub direction: WhipDirection,
    #[default = 24]
    #[schemars(description = "Blur", range(min = 0, max = 48), extend("x-step" = 2))]
    pub blur: u32,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct PushThrough {
    #[default = 2.4]
    #[schemars(description = "Zoom", range(min = 1.5, max = 4), extend("x-step" = 0.1))]
    pub zoom: f64,
    #[default = 14]
    #[schemars(description = "Blur", range(min = 0, max = 30), extend("x-step" = 1))]
    pub blur: u32,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ZoomBlur {
    #[default = 16]
    #[schemars(description = "Blur", range(min = 0, max = 40), extend("x-step" = 2))]
    pub blur: u32,
    #[schemars(description = "Rise", range(min = 0, max = 160), extend("x-step" = 10))]
    pub rise: u32,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct FocusPull {
    #[default = 16]
    #[schemars(description = "Blur", range(min = 4, max = 32), extend("x-step" = 1))]
    pub blur: u32,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct LensZoom {
    #[default = 9]
    #[schemars(description = "Lens steps", range(min = 4, max = 12), extend("x-step" = 1))]
    pub lens_steps: u32,
    #[default = 7]
    #[schemars(description = "Blur samples", range(min = 2, max = 12), extend("x-step" = 1))]
    pub blur_samples: u32,
    #[default = 50]
    #[schemars(description = "Shake amount", range(min = 0, max = 300), extend("x-step" = 5))]
    pub shake_amount: u32,
    #[default = 73]
    #[schemars(description = "Shake travel (px)", range(min = 0, max = 80), extend("x-step" = 1))]
    pub shake_translate_px: u32,
    #[default = true]
    #[schemars(description = "Colour fringe", extend("x-help" = "Splits the colours at the edges. Off draws the transition a third as often, for heavy scenes.", "x-advanced" = true))]
    pub color_split: bool,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct RippleZoom {
    #[default = 4.0]
    #[schemars(description = "Zoom", range(min = 1, max = 8), extend("x-step" = 0.1, "x-advanced" = true))]
    pub zoom: f64,
    #[default = 0.5]
    #[schemars(description = "Intensity", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub intensity: f64,
    #[default = 0.5]
    #[schemars(description = "Softness", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub softness: f64,
    #[default = "#141318"]
    #[schemars(description = "Background", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub color_back: String,
    #[default(vec!["#3a3a52".into(), "#4a4a68".into(), "#8f88ae".into()])]
    #[schemars(description = "Colours", length(min = 2, max = 7))]
    pub colors: Vec<Colour>,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25, "x-advanced" = true))]
    pub speed: f64,
    #[default = 0.5]
    #[schemars(description = "Grain", range(min = 0, max = 1), extend("x-step" = 0.01, "x-advanced" = true))]
    pub noise: f64,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct WaveWipe {
    #[default = 0.2]
    #[schemars(description = "Intensity", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub intensity: f64,
    #[default = 0.7]
    #[schemars(description = "Softness", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub softness: f64,
    #[default = 0.4]
    #[schemars(description = "Noise", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub noise: f64,
    #[default = "#141318"]
    #[schemars(description = "Background", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub color_back: String,
    #[default(vec!["#3a3a52".into(), "#4a4a68".into(), "#8f88ae".into()])]
    #[schemars(description = "Colours", length(min = 2, max = 7))]
    pub colors: Vec<Colour>,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25, "x-advanced" = true))]
    pub speed: f64,
    #[default = 1.16]
    #[schemars(description = "Zoom", range(min = 0.5, max = 4), extend("x-step" = 0.01, "x-advanced" = true))]
    pub zoom: f64,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct SmokeDissolve {
    #[default = "#8f88ae"]
    #[schemars(description = "Ring", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub ring_color: String,
    #[default = "#141318"]
    #[schemars(description = "Background", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub color_back: String,
    #[default(vec!["#8f88ae".into()])]
    #[schemars(description = "Colours", length(min = 1, max = 10))]
    pub colors: Vec<Colour>,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25, "x-advanced" = true))]
    pub speed: f64,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct GlitchCut {
    #[default = 1.0]
    #[schemars(description = "Intensity", range(min = 0, max = 2), extend("x-step" = 0.1))]
    pub intensity: f64,
    #[default = 24]
    #[schemars(description = "Slices", range(min = 8, max = 64), extend("x-step" = 2))]
    pub slices: u32,
    #[default = 1.0]
    #[schemars(description = "RGB split", range(min = 0, max = 2), extend("x-step" = 0.1))]
    pub rgb_split: f64,
    #[default = 0.6]
    #[schemars(description = "Block noise", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub block_noise: f64,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct PageTurn {
    #[default(-7)]
    #[schemars(description = "Angle", range(min = -20, max = 20), extend("x-step" = 1))]
    pub angle: i32,
    #[default = 8]
    #[schemars(description = "Poses", range(min = 2, max = 16), extend("x-step" = 1))]
    pub poses: u32,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct GrainDissolve {
    #[default(GrainDissolveShape::Blob)]
    #[schemars(description = "Shape")]
    pub shape: GrainDissolveShape,
    #[default = 0.3]
    #[schemars(description = "Noise", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub noise: f64,
    #[default = 2.0]
    #[schemars(description = "Zoom", range(min = 0.5, max = 4), extend("x-step" = 0.25))]
    pub zoom: f64,
    #[default = "#141318"]
    #[schemars(description = "Background", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub color_back: String,
    #[default(vec!["#3a3a52".into(), "#4a4a68".into(), "#8f88ae".into()])]
    #[schemars(description = "Colours", length(min = 2, max = 7))]
    pub colors: Vec<Colour>,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25, "x-advanced" = true))]
    pub speed: f64,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct DitherDissolve {
    #[default(DitherShape::Simplex)]
    #[schemars(description = "Shape")]
    pub shape: DitherShape,
    #[default = "#141318"]
    #[schemars(description = "Background", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub color_back: String,
    #[default = "#8f88ae"]
    #[schemars(description = "Ink", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub color_front: String,
    #[default = 1.5]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25, "x-advanced" = true))]
    pub speed: f64,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct SwirlDissolve {
    #[default = 10]
    #[schemars(description = "Bands", range(min = 1, max = 12), extend("x-step" = 1))]
    pub band_count: u32,
    #[default = 0.35]
    #[schemars(description = "Softness", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub softness: f64,
    #[default = "#141318"]
    #[schemars(description = "Background", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub color_back: String,
    #[default(vec!["#1f1d29".into(), "#413d56".into(), "#8f88ae".into()])]
    #[schemars(description = "Colours", length(min = 2, max = 10))]
    pub colors: Vec<Colour>,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25, "x-advanced" = true))]
    pub speed: f64,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ParticleDissolve {
    #[default = 4]
    #[schemars(description = "Grain size", range(min = 1, max = 16), extend("x-step" = 1))]
    pub particle_size: u32,
    #[default = 0.08]
    #[schemars(description = "Scatter", range(min = 0.01, max = 0.4), extend("x-step" = 0.01))]
    pub scatter: f64,
    #[default = 0.6]
    #[schemars(description = "Shimmer", range(min = 0, max = 1.5), extend("x-step" = 0.05))]
    pub shimmer: f64,
    #[default = 0.5]
    #[schemars(description = "Aberration", range(min = 0, max = 1.5), extend("x-step" = 0.05))]
    pub aberration: f64,
    #[default(ParticleDissolveDirection::Reveal)]
    #[schemars(description = "Direction")]
    pub direction: ParticleDissolveDirection,
    #[default = 0.55]
    #[schemars(description = "Stagger", range(min = 0, max = 0.9), extend("x-step" = 0.05))]
    pub stagger: f64,
}

pub(super) fn elements() -> Vec<ElementMeta> {
    use ElementCategory as C;
    vec![
        meta::<WhipPan>(
            element(
                ElementKind::Presentation,
                "whip-pan",
                "Whip pan",
                "A whip-pan transition that flies both scenes through the frame in one move with velocity-based motion blur.",
                C::Transition,
                114,
            )
                .source("Remocn", "https://remocn.dev/docs/components/whip-pan", "MIT")
                .look(NodeLook::of("lucide:move-horizontal").moves(NodeMotion::Whip)),
        ),
        meta::<PushThrough>(
            element(
                ElementKind::Presentation,
                "push-through",
                "Push through",
                "A dolly transition that blows the outgoing scene past the camera while the next one approaches from the depth.",
                C::Transition,
                112,
            )
                .source("Remocn", "https://remocn.dev/docs/components/push-through", "MIT")
                .look(NodeLook::of("lucide:arrow-right-to-line").moves(NodeMotion::Push)),
        ),
        meta::<ZoomBlur>(
            element(
                ElementKind::Presentation,
                "zoom-blur",
                "Zoom blur",
                "A depth punch-in transition that pushes the outgoing scene through the viewer while the incoming one resolves out of blur.",
                C::Transition,
                114,
            )
                .source("Remocn", "https://remocn.dev/docs/components/zoom-blur", "MIT")
                .look(NodeLook::of("lucide:expand").moves(NodeMotion::ZoomBlur)),
        ),
        meta::<FocusPull>(
            element(
                ElementKind::Presentation,
                "focus-pull",
                "Focus pull",
                "A rack-focus transition that defocuses the outgoing scene into a bright blur and resolves the next one into focus.",
                C::Transition,
                114,
            )
                .source("Remocn", "https://remocn.dev/docs/components/focus-pull", "MIT")
                .look(NodeLook::of("lucide:focus").moves(NodeMotion::RackFocus)),
        ),
        meta::<LensZoom>(
            element(
                ElementKind::Presentation,
                "lens-zoom",
                "Lens zoom",
                "A hard cut disguised as a lens punch-in: barrel distortion, zoom blur, chromatic split and camera shake around the splice.",
                C::Transition,
                112,
            )
                .source("Remocn", "https://remocn.dev/docs/components/lens-zoom", "MIT")
                .look(NodeLook::of("lucide:zoom-in").moves(NodeMotion::ZoomThrough)),
        ),
        meta::<RippleZoom>(
            element(
                ElementKind::Presentation,
                "ripple-zoom",
                "Ripple zoom",
                "A grain-gradient ripple transition that flies the camera through the center of the rings while the next scene approaches from the depth.",
                C::Transition,
                116,
            )
                .source("Remocn", "https://remocn.dev/docs/components/ripple-zoom", "MIT")
                .look(NodeLook::of("tabler:ripple").moves(NodeMotion::Ripple)),
        ),
        meta::<WaveWipe>(
            element(
                ElementKind::Presentation,
                "wave-wipe",
                "Wave wipe",
                "A grain-gradient wave that sweeps up across the frame, covering the outgoing scene and revealing the next one from below.",
                C::Transition,
                116,
            )
                .source("Remocn", "https://remocn.dev/docs/components/wave-wipe", "MIT")
                .look(NodeLook::of("lucide:waves-horizontal").moves(NodeMotion::WipeReveal)),
        ),
        meta::<SmokeDissolve>(
            element(
                ElementKind::Presentation,
                "smoke-dissolve",
                "Smoke dissolve",
                "A smoke-ring shader transition that expands the ring outward to reveal the next scene.",
                C::Transition,
                116,
            )
                .source("Remocn", "https://remocn.dev/docs/components/smoke-dissolve", "MIT")
                .look(NodeLook::of("lucide:cloud-fog").moves(NodeMotion::Dissolve)),
        ),
        meta::<GlitchCut>(
            element(
                ElementKind::Presentation,
                "glitch-cut",
                "Glitch cut",
                "A broadcast-glitch hard cut: the frame tears into displaced slices with channel split and corrupted blocks, and the incoming scene takes over slice by slice at the peak.",
                C::Transition,
                78,
            )
                .source("Remocn", "https://remocn.dev/docs/components/glitch-cut", "MIT")
                .look(NodeLook::of("lucide:zap").moves(NodeMotion::Glitch)),
        ),
        meta::<PageTurn>(
            element(
                ElementKind::Presentation,
                "page-turn",
                "Page turn",
                "The exiting scene swings up and away like a notebook page in stop-motion poses, staying above the scene it reveals.",
                C::Transition,
                128,
            )
                .source("Remocn", "https://remocn.dev/docs/components/page-turn", "MIT")
                .look(NodeLook::of("lucide:book-open").moves(NodeMotion::Flip)),
        ),
        meta::<GrainDissolve>(
            element(
                ElementKind::Presentation,
                "grain-dissolve",
                "Grain dissolve",
                "A grain-gradient shader transition that disintegrates the outgoing scene into soft grainy shapes and condenses the next scene out of the noise.",
                C::Transition,
                116,
            )
                .source("Remocn", "https://remocn.dev/docs/components/grain-dissolve", "MIT")
                .look(NodeLook::of("tabler:grain").moves(NodeMotion::Dissolve)),
        ),
        meta::<DitherDissolve>(
            element(
                ElementKind::Presentation,
                "dither-dissolve",
                "Dither dissolve",
                "A dithering shader transition that dissolves one scene into the next.",
                C::Transition,
                100,
            )
                .source("Remocn", "https://remocn.dev/docs/components/dither-dissolve", "MIT")
                .look(NodeLook::of("lucide:grip").moves(NodeMotion::Dissolve)),
        ),
        meta::<SwirlDissolve>(
            element(
                ElementKind::Presentation,
                "swirl-dissolve",
                "Swirl dissolve",
                "A swirl shader transition that unwinds open and winds shut to reveal the next scene.",
                C::Transition,
                116,
            )
                .source("Remocn", "https://remocn.dev/docs/components/swirl-dissolve", "MIT")
                .look(NodeLook::of("tabler:spiral").moves(NodeMotion::Dissolve)),
        ),
        meta::<ParticleDissolve>(
            element(
                ElementKind::Presentation,
                "particle-dissolve",
                "Particle dissolve",
                "The frame is ground into drifting grey dust whose specks carry real fragments of the picture, and the next scene coalesces back out of the same dust.",
                C::Transition,
                84,
            )
                .source("Remocn", "https://remocn.dev/docs/components/particle-dissolve", "MIT")
                .look(NodeLook::of("lucide:sparkles").moves(NodeMotion::Dissolve)),
        ),
    ]
}
