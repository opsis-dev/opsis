use opsis_types::looks::{NodeLook, NodeMotion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::choices::{MorphShape, Orientation, PixelShape, PixelateOrder, WipeFrom};
use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct GridPixelateWipe {
    #[default = 12.0]
    #[schemars(description = "Columns", range(min = 1, max = 48), extend("x-step" = 1, "x-help" = "Grid column count."))]
    pub cols: f64,
    #[default = 8.0]
    #[schemars(description = "Rows", range(min = 1, max = 48), extend("x-step" = 1, "x-help" = "Grid row count."))]
    pub rows: f64,
    #[default(PixelateOrder::FromLeft)]
    #[schemars(description = "Order", extend("x-help" = "Which axis or point the cells light up from."))]
    pub order: PixelateOrder,
    #[default(PixelShape::Square)]
    #[schemars(description = "Shape", extend("x-help" = "Cells fill as blocks, or as points that grow with their own reveal."))]
    pub shape: PixelShape,
    #[default = 0.82]
    #[schemars(description = "Stagger", range(min = 0, max = 1), extend("x-step" = 0.01, "x-help" = "0 pops every cell together; 1 spreads them across the whole window.", "x-advanced" = true))]
    pub stagger: f64,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct FrostedGlassWipe {
    #[default = 20.0]
    #[schemars(description = "Blur", extend("x-help" = "Frost panel blur radius."))]
    pub blur: f64,
    #[default = 0.14]
    #[schemars(description = "Panel width", extend("x-step" = 0.01, "x-help" = "Sweep panel width as fraction of frame.", "x-advanced" = true))]
    pub panel_width: f64,
    #[default(WipeFrom::FromLeft)]
    #[schemars(description = "Direction", extend("x-help" = "Sweep direction."))]
    pub direction: WipeFrom,
    #[default = "rgba(255,255,255,0.12)"]
    #[schemars(description = "Frost color", extend("x-help" = "Tint of the glass panel.", "x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub frost_color: String,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ChromaticAberrationWipe {
    #[default = 12.0]
    #[schemars(description = "Intensity")]
    pub intensity: f64,
    #[default(Orientation::Horizontal)]
    #[schemars(description = "Axis", extend("x-help" = "Axis the scenes slide along and the channels separate on."))]
    pub axis: Orientation,
    #[default = 1.0]
    #[schemars(description = "Slide distance", extend("x-help" = "Share of the frame the scenes travel. Below 1 shows background.", "x-advanced" = true))]
    pub slide: f64,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct TransitionBlinds {
    #[default(Orientation::Horizontal)]
    #[schemars(description = "Orientation", extend("x-help" = "Horizontal slats stack top to bottom and sweep sideways."))]
    pub orientation: Orientation,
    #[default = 12.0]
    #[schemars(description = "Slats", extend("x-help" = "Number of slats. Above ~24 it stops reading as slats at 1080p."))]
    pub slats: f64,
    #[default = 0.45]
    #[schemars(description = "Stagger", extend("x-step" = 0.01, "x-help" = "Share of the window the cascade spans. 0 opens every slat together.", "x-advanced" = true))]
    pub stagger: f64,
    #[schemars(description = "Alternate", extend("x-help" = "Reverse alternate slats for a woven counter-sweep."))]
    pub alternate: bool,
    #[default = 0.06]
    #[schemars(description = "Edge softness", extend("x-step" = 0.01, "x-help" = "Soft leading edge per slat, as a share of its length.", "x-advanced" = true))]
    pub edge_softness: f64,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct TransitionMorphShape {
    #[default(MorphShape::Circle)]
    #[schemars(description = "From", extend("x-help" = "Shape the reveal starts as, at zero size. A raw path d also works."))]
    pub from: MorphShape,
    #[default(MorphShape::Squircle)]
    #[schemars(description = "To", extend("x-help" = "Shape it has become once it covers the frame."))]
    pub to: MorphShape,
    #[default = 0.5]
    #[schemars(description = "Horizontal origin", extend("x-step" = 0.01, "x-help" = "Where the shape grows from, 0→1 of the frame width.", "x-advanced" = true))]
    pub origin_x: f64,
    #[default = 0.5]
    #[schemars(description = "Vertical origin", extend("x-step" = 0.01, "x-help" = "Where the shape grows from, 0→1 of the frame height.", "x-advanced" = true))]
    pub origin_y: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Overshoot", range(min = 0.8, max = 3), extend("x-zero-label" = "Fit the shape", "x-step" = 0.05, "x-help" = "Final size as a multiple of the frame diagonal. Leave empty to grow each shape until it covers the frame.", "x-advanced" = true, "type" = "number"))]
    pub overshoot: Option<f64>,
    #[schemars(description = "Warp", extend("x-help" = "0 keeps the silhouette crisp.", "x-advanced" = true))]
    pub warp: f64,
}

pub(super) fn elements() -> Vec<ElementMeta> {
    use ElementCategory as C;
    vec![
        meta::<GridPixelateWipe>(
            element(
                ElementKind::Presentation,
                "remotion-ui/grid-pixelate-wipe",
                "Pixel grid wipe",
                "The next scene appears one grid cell at a time.",
                C::Transition,
                26,
            )
                .source("RemotionUI", "https://remotionui.com/docs/components/grid-pixelate-wipe", "MIT")
                .look(NodeLook::of("lucide:grid-2x2").moves(NodeMotion::WipeReveal)),
        ),
        meta::<FrostedGlassWipe>(
            element(
                ElementKind::Presentation,
                "remotion-ui/frosted-glass-wipe",
                "Frosted glass wipe",
                "A pane of frosted glass crosses the frame and leaves the next scene behind it.",
                C::Transition,
                24,
            )
                .source("RemotionUI", "https://remotionui.com/docs/components/frosted-glass-wipe", "MIT")
                .look(NodeLook::of("tabler:blur").moves(NodeMotion::WipeReveal)),
        ),
        meta::<ChromaticAberrationWipe>(
            element(
                ElementKind::Presentation,
                "remotion-ui/chromatic-aberration-wipe",
                "Colour split slide",
                "A fast slide that pulls the colours apart mid-move and brings them back together as it lands.",
                C::Transition,
                14,
            )
                .source("RemotionUI", "https://remotionui.com/docs/components/chromatic-aberration-wipe", "MIT")
                .look(NodeLook::of("tabler:color-filter").moves(NodeMotion::SlideThrough)),
        ),
        meta::<TransitionBlinds>(
            element(
                ElementKind::Presentation,
                "remotion-ui/transition-blinds",
                "Blinds",
                "Slats open one after another to reveal the next scene.",
                C::Transition,
                24,
            )
                .source("RemotionUI", "https://remotionui.com/docs/components/transition-blinds", "MIT")
                .look(NodeLook::of("lucide:blinds").moves(NodeMotion::Shutter)),
        ),
        meta::<TransitionMorphShape>(
            element(
                ElementKind::Presentation,
                "remotion-ui/transition-morph-shape",
                "Shape morph",
                "The next scene shows through a shape that grows and changes form until it fills the frame.",
                C::Transition,
                24,
            )
                .source("RemotionUI", "https://remotionui.com/docs/components/transition-morph-shape", "MIT")
                .look(NodeLook::of("lucide:square").morphs("lucide:circle")),
        ),
    ]
}
