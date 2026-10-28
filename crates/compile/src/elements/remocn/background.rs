use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::choices::{DitherPattern, DitherShape, GrainGradientShape};
use crate::elements::core::Colour;
use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ShaderMeshGradient {
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.1, max = 4), extend("x-step" = 0.1))]
    pub speed: f64,
    #[default = 0.6]
    #[schemars(description = "Distortion", range(min = 0, max = 1.5), extend("x-step" = 0.05))]
    pub distortion: f64,
    #[default = 0.1]
    #[schemars(description = "Swirl", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub swirl: f64,
    #[default(vec!["#12121a".into(), "#232338".into(), "#3a3a5c".into(), "#52527a".into()])]
    #[schemars(description = "Colours", length(min = 2, max = 10))]
    pub colors: Vec<Colour>,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ShaderGrainGradient {
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.1, max = 4), extend("x-step" = 0.1))]
    pub speed: f64,
    #[default = 0.6]
    #[schemars(description = "Softness", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub softness: f64,
    #[default = 0.2]
    #[schemars(description = "Intensity", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub intensity: f64,
    #[default = 0.15]
    #[schemars(description = "Noise", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub noise: f64,
    #[default = "#12121a"]
    #[schemars(description = "Background", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub color_back: String,
    #[default(vec!["#3a3a52".into(), "#4a4a68".into(), "#5a5a7e".into()])]
    #[schemars(description = "Colours", length(min = 2, max = 7))]
    pub colors: Vec<Colour>,
    #[default(GrainGradientShape::Corners)]
    #[schemars(description = "Pattern")]
    pub shape: GrainGradientShape,
    #[default = 1.0]
    #[schemars(description = "Scale", range(min = 0.1, max = 4), extend("x-step" = 0.05, "x-help" = "How large the pattern is in the frame.", "x-advanced" = true))]
    pub scale: f64,
    #[schemars(description = "Turn", range(min = 0, max = 360), extend("x-step" = 1, "x-unit" = "°", "x-help" = "Turns the pattern around the middle of the frame.", "x-advanced" = true))]
    pub rotation: u32,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ShaderWarp {
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.1, max = 4), extend("x-step" = 0.1))]
    pub speed: f64,
    #[default = 0.2]
    #[schemars(description = "Distortion", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub distortion: f64,
    #[default = 0.4]
    #[schemars(description = "Swirl", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub swirl: f64,
    #[default = 1.0]
    #[schemars(description = "Softness", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub softness: f64,
    #[default = 0.5]
    #[schemars(description = "Proportion", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub proportion: f64,
    #[default(vec!["#12121a".into(), "#3a3a5c".into(), "#12121a".into(), "#52527a".into()])]
    #[schemars(description = "Colours", length(min = 2, max = 10))]
    pub colors: Vec<Colour>,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ShaderSwirl {
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.1, max = 4), extend("x-step" = 0.1))]
    pub speed: f64,
    #[default = 4]
    #[schemars(description = "Bands", range(min = 1, max = 12), extend("x-step" = 1))]
    pub band_count: u32,
    #[default = 0.1]
    #[schemars(description = "Twist", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub twist: f64,
    #[default = 0.2]
    #[schemars(description = "Softness", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub softness: f64,
    #[default = "#12121a"]
    #[schemars(description = "Background", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub color_back: String,
    #[default(vec!["#52527a".into(), "#3a3a5c".into(), "#232338".into()])]
    #[schemars(description = "Colours", length(min = 2, max = 10))]
    pub colors: Vec<Colour>,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ShaderGodRays {
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.1, max = 4), extend("x-step" = 0.1))]
    pub speed: f64,
    #[default = 0.8]
    #[schemars(description = "Intensity", range(min = 0, max = 2), extend("x-step" = 0.05))]
    pub intensity: f64,
    #[default = 0.3]
    #[schemars(description = "Density", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub density: f64,
    #[default = 0.4]
    #[schemars(description = "Bloom", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub bloom: f64,
    #[default = "#12121a"]
    #[schemars(description = "Background", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub color_back: String,
    #[default(vec!["#5a5a7e".into(), "#8a8a95".into(), "#ffffff".into(), "#3a3a5c".into()])]
    #[schemars(description = "Colours", length(min = 1, max = 5))]
    pub colors: Vec<Colour>,
    #[default = "#3a3a5c"]
    #[schemars(description = "Glow colour", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub color_bloom: String,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ShaderPerlinNoise {
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.1, max = 4), extend("x-step" = 0.1))]
    pub speed: f64,
    #[default = 0.35]
    #[schemars(description = "Proportion", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub proportion: f64,
    #[default = 0.1]
    #[schemars(description = "Softness", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub softness: f64,
    #[default = "#12121a"]
    #[schemars(description = "Background", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub color_back: String,
    #[default = "#6a6a85"]
    #[schemars(description = "Front", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub color_front: String,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ShaderDithering {
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.1, max = 4), extend("x-step" = 0.1))]
    pub speed: f64,
    #[default = 2]
    #[schemars(description = "Dot Size", range(min = 1, max = 8), extend("x-step" = 1))]
    pub size: u32,
    #[default = "#12121a"]
    #[schemars(description = "Background", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub color_back: String,
    #[default = "#6a6a85"]
    #[schemars(description = "Front", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub color_front: String,
    #[default(DitherShape::Wave)]
    #[schemars(description = "Pattern", extend("x-advanced" = true))]
    pub shape: DitherShape,
    #[default(DitherPattern::Grid4x4)]
    #[schemars(description = "Dot grid", extend("x-advanced" = true, "x-labels" = {"random":"Random","2x2":"2 × 2","4x4":"4 × 4","8x8":"8 × 8"}))]
    pub r#type: DitherPattern,
}

pub(super) fn elements() -> Vec<ElementMeta> {
    use ElementCategory as C;
    vec![
        meta::<ShaderMeshGradient>(
            element(
                ElementKind::Component,
                "shader-mesh-gradient",
                "Shader mesh gradient",
                "Soft colour blobs that drift and blend into each other.",
                C::Background,
                150,
            )
                .source("Remocn", "https://remocn.dev/docs/components/shader-mesh-gradient", "MIT")
                .authored_clock()
                .look(NodeLook::of("lucide:blend")),
        ),
        meta::<ShaderGrainGradient>(
            element(
                ElementKind::Component,
                "shader-grain-gradient",
                "Shader grain gradient",
                "Grainy multi-stop gradient that drifts slowly with film-grain texture over the blend.",
                C::Background,
                150,
            )
                .source("Remocn", "https://remocn.dev/docs/components/shader-grain-gradient", "MIT")
                .authored_clock()
                .look(NodeLook::of("tabler:noise-reduction")),
        ),
        meta::<ShaderWarp>(
            element(
                ElementKind::Component,
                "shader-warp",
                "Shader warp",
                "Domain-warped color field that folds and stretches like liquid.",
                C::Background,
                150,
            )
                .source("Remocn", "https://remocn.dev/docs/components/shader-warp", "MIT")
                .authored_clock()
                .look(NodeLook::of("lucide:droplets")),
        ),
        meta::<ShaderSwirl>(
            element(
                ElementKind::Component,
                "shader-swirl",
                "Shader swirl",
                "Concentric bands swirling around a center point.",
                C::Background,
                150,
            )
                .source("Remocn", "https://remocn.dev/docs/components/shader-swirl", "MIT")
                .authored_clock()
                .look(NodeLook::of("tabler:spiral")),
        ),
        meta::<ShaderGodRays>(
            element(
                ElementKind::Component,
                "shader-god-rays",
                "Shader god rays",
                "Volumetric light rays radiating from an off-screen source.",
                C::Background,
                150,
            )
                .source("Remocn", "https://remocn.dev/docs/components/shader-god-rays", "MIT")
                .authored_clock()
                .look(NodeLook::of("lucide:sunrise")),
        ),
        meta::<ShaderPerlinNoise>(
            element(
                ElementKind::Component,
                "shader-perlin-noise",
                "Shader perlin noise",
                "Classic Perlin noise clouds drifting across the frame.",
                C::Background,
                150,
            )
                .source("Remocn", "https://remocn.dev/docs/components/shader-perlin-noise", "MIT")
                .authored_clock()
                .look(NodeLook::of("lucide:cloudy")),
        ),
        meta::<ShaderDithering>(
            element(
                ElementKind::Component,
                "shader-dithering",
                "Shader dithering",
                "Ordered-dither shading over an animated gradient field.",
                C::Background,
                150,
            )
                .source("Remocn", "https://remocn.dev/docs/components/shader-dithering", "MIT")
                .authored_clock()
                .look(NodeLook::of("lucide:grip")),
        ),
    ]
}
