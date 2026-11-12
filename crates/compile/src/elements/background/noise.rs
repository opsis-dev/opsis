use opsis_macros::settings;
use opsis_scene::color::Color;
use opsis_scene::css::with_background;
use opsis_scene::node::{Node, div};
use opsis_scene::noise::NoiseField;
use opsis_scene::paint::{ColorStop, Filter, Gradient as Painted, ImageLayer, RadialShape};
use opsis_scene::random::random;
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::{Length, Position2};
use opsis_types::looks::NodeLook;

use crate::elements::backgrounds::grain;
use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct Noise {
    #[setting("Base", default = "#0b1020", check = colour)]
    pub color1: String,
    #[setting("Highlight", default = "#4f46e5", check = colour)]
    pub color2: String,
    #[setting(
        "Scale",
        default = 0.004,
        range(min = 0.0005, max = 0.02),
        step = 0.0005,
        advanced
    )]
    pub scale: f64,
    #[setting("Speed", default = 0.004, range(min = 0, max = 0.05), step = 0.001)]
    pub speed: f64,
    #[setting("Blobs", default = 5, range(min = 1, max = 12), step = 1, advanced)]
    pub blobs: u32,
    #[setting(
        "Wander",
        default = 40.0,
        range(min = 0, max = 60),
        unit = "%",
        advanced
    )]
    pub wander: f64,
    #[setting(
        "Blob size",
        default = 30.0,
        range(min = 5, max = 80),
        unit = "%",
        advanced
    )]
    pub blob_size: f64,
    #[setting("Intensity", default = 0.55, range(min = 0, max = 1))]
    pub intensity: f64,
    #[setting(
        "Softness",
        default = 40,
        range(min = 0, max = 120),
        step = 1,
        unit = "px",
        advanced
    )]
    pub softness: u32,
    #[setting("Variation", step = 1, advanced)]
    pub seed: u32,
    #[setting("Grain", default = 0.08, range(min = 0, max = 0.4))]
    pub grain: f64,
    #[setting(
        "Grain size",
        default = 1.0,
        range(min = 0.5, max = 4),
        unit = "x",
        advanced
    )]
    pub grain_size: f64,
}

pub fn definition() -> ElementMeta {
    meta::<Noise>(
        element(
            ElementKind::Component,
            "noise",
            "Noise",
            "A soft moving noise background.",
            ElementCategory::Background,
            150,
        )
        .look(NodeLook::of("tabler:grain")),
    )
}

const BLOB_RADIUS_SPREAD_PERCENT: f64 = 25.0;
const BLOB_NOISE_SPACING: f64 = 1000.0;
const Y_LANE: f64 = 731.0;
const NOISE_GRAIN_SEED: u32 = 7;

fn seeded(key: &str, seed: u32) -> String {
    if seed == 0 {
        key.to_owned()
    } else {
        format!("{key}-{seed}")
    }
}

pub fn draw(props: &Noise, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let field = NoiseField::new(&format!("noise:{}", props.seed));
    let highlight = cx.color(&props.color2);
    let time = cx.frame() * props.speed;
    let layers: Vec<ImageLayer> = (0..props.blobs)
        .map(|i| {
            let offset = f64::from(i) * BLOB_NOISE_SPACING * props.scale;
            let x = field.at(offset, 0.0, time).mul_add(props.wander, 50.0);
            let y = field.at(offset, Y_LANE, time).mul_add(props.wander, 50.0);
            let r = random(seeded(&format!("r{i}"), props.seed)).mul_add(BLOB_RADIUS_SPREAD_PERCENT, props.blob_size);
            Painted::radial(
                RadialShape::Circle,
                Position2 {
                    x: Length::percent(x),
                    y: Length::percent(y),
                },
                vec![
                    ColorStop::at(highlight.clone(), Length::percent(0.0)),
                    ColorStop::at(Color::TRANSPARENT, Length::percent(r)),
                ],
            )
            .layer()
        })
        .collect();
    Ok(div(with_background(absolute_fill(), &props.color1))
        .child(div(absolute_fill()
            .background(layers)
            .opacity(props.intensity)
            .filter(vec![Filter::Blur(Length::px(f64::from(
                props.softness,
            )))])))
        .children(grain(props.grain, props.grain_size, NOISE_GRAIN_SEED))
        .into())
}
