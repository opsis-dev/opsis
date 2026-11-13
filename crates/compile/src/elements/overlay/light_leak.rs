use opsis_macros::settings;
use opsis_scene::node::{Node, div};
use opsis_scene::paint::BlendMode as SceneBlend;
use opsis_scene::timeline::absolute_fill;
use opsis_types::looks::NodeLook;

use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::shader::{generated, quads, rgb_uniform, value};
use crate::elements::shared::{BlendMode, Colour};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct LightLeak {
    #[setting("Seed", default = 3, range(min = 0, max = 100), step = 1)]
    pub seed: u32,
    #[setting(
        "Colors",
        default = leak_colours(),
        length(min = 1, max = 4),
        help = "The light's colours, one glow each."
    )]
    pub colors: Vec<Colour>,
    #[setting(
        "Strength",
        default = 100.0,
        range(min = 0, max = 100),
        unit = "%",
        advanced
    )]
    pub strength: f64,
    #[setting(
        "Blend",
        default = BlendMode::Normal,
        advanced
    )]
    pub blend: BlendMode,
}

pub fn leak_colours() -> Vec<Colour> {
    vec!["#ff8a3d".into(), "#ffd166".into(), "#ff4f7b".into()]
}

pub fn definition() -> ElementMeta {
    meta::<LightLeak>(
        element(
            ElementKind::Component,
            "light-leak",
            "Light leak",
            "A warm light flare over the video.",
            ElementCategory::Overlay,
            45,
        )
        .look(NodeLook::of("lucide:sun-dim")),
    )
}

const MAX_COLOURS: usize = 4;
const CANVAS_SCALE: f64 = 1.0;

const fn blend_of(blend: BlendMode) -> SceneBlend {
    match blend {
        BlendMode::Multiply => SceneBlend::Multiply,
        BlendMode::Screen => SceneBlend::Screen,
        BlendMode::Normal => SceneBlend::Normal,
    }
}

pub fn draw(props: &LightLeak, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let duration = cx.duration();
    let progress = if duration > 1.0 {
        cx.frame() / (duration - 1.0)
    } else {
        1.0
    };
    let shown: Vec<&Colour> = props.colors.iter().take(MAX_COLOURS).collect();
    let colours: Vec<f64> = shown
        .iter()
        .flat_map(|c| {
            let mut rgb = rgb_uniform(&c.0);
            rgb.push(1.0);
            rgb
        })
        .collect();
    let mut settings = vec![
        value("progress", vec![progress]),
        value("seed", vec![f64::from(props.seed)]),
        value(
            "colorsCount",
            vec![opsis_types::numbers::float(shown.len())],
        ),
    ];
    if !colours.is_empty() {
        settings.push(quads("colors", colours));
    }
    Ok(div(absolute_fill()
        .opacity(props.strength / 100.0)
        .mix_blend_mode(blend_of(props.blend)))
    .child(generated("light-leak", settings, CANVAS_SCALE))
    .into())
}
