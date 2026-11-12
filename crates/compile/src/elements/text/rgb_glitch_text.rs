use opsis_macros::settings;
use opsis_scene::color::Color;
use opsis_scene::node::{Node, div, span};
use opsis_scene::paint::BlendMode as SceneBlend;
use opsis_scene::random::{js_number_text, random};
use opsis_scene::style::{Display, Position, Style, WhiteSpace};
use opsis_scene::values::Length;
use opsis_types::looks::NodeLook;

use crate::elements::backgrounds::centered_stage;
use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::shared::{BlendMode, Colour, FontWeight};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct RgbGlitchText {
    #[setting("Text", default = "GLITCH")]
    pub text: String,
    #[setting("Font size", default = 96, range(min = 12, max = 200), step = 1)]
    pub font_size: u32,
    #[setting("Color", default = "#171717", check = colour)]
    pub color: String,
    #[setting("Font weight", default = FontWeight::W700)]
    pub font_weight: FontWeight,
    #[setting(
        "Glitch at",
        default = 0.6667,
        range(min = 0, max = 4),
        step = 0.01,
        unit = "s",
        frames_at = 30
    )]
    pub glitch_at: f64,
    #[setting(
        "Glitch length",
        default = 0.2667,
        range(min = 0.0333, max = 2),
        step = 0.01,
        unit = "s",
        frames_at = 30
    )]
    pub glitch_duration: f64,
    #[setting("Intensity (px)", default = 6, range(min = 0, max = 30), step = 1)]
    pub intensity: u32,
    #[setting("Seed", default = "glitch", advanced)]
    pub seed: String,
    #[setting(
        "Glitch colours",
        default = vec!["#ff0040".into(), "#00ff80".into(), "#0080ff".into()],
        length(min = 3, max = 3),
        help = "The three copies that split off while the text glitches.",
        advanced
    )]
    pub glitch_colors: Vec<Colour>,
    #[setting(
        "Glitch blend",
        default = BlendMode::Multiply,
        help = "How the split copies mix with what is behind them.",
        labels = { "multiply": "On a light background", "screen": "On a dark background", "normal": "On any background" },
        advanced
    )]
    pub blend: BlendMode,
    #[setting(
        "Background",
        default = "#fafafa",
        help = "Set it to transparent to lay the text over other layers.",
        check = colour
    )]
    pub background: String,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project."
    )]
    pub font_family: String,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4), step = 0.25)]
    pub speed: f64,
    #[setting("Letter spacing", default = -0.03, range(min = -0.2, max = 0.5), unit = "em", advanced)]
    pub letter_spacing: f64,
}

pub fn definition() -> ElementMeta {
    meta::<RgbGlitchText>(
        element(
            ElementKind::Component,
            "rgb-glitch-text",
            "RGB glitch text",
            "Three RGB-offset text copies jitter for a brief glitch window.",
            ElementCategory::Text,
            90,
        )
        .authored_clock()
        .look(NodeLook::of("lucide:type")),
    )
}

const VERTICAL_SHARE: f64 = 0.4;

const fn blend_of(blend: BlendMode) -> SceneBlend {
    match blend {
        BlendMode::Multiply => SceneBlend::Multiply,
        BlendMode::Screen => SceneBlend::Screen,
        BlendMode::Normal => SceneBlend::Normal,
    }
}

pub fn draw(props: &RgbGlitchText, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let frame = cx.authored_frame() * props.speed;
    let glitching = frame >= props.glitch_at && frame < props.glitch_at + props.glitch_duration;
    let intensity = f64::from(props.intensity);
    let offset = |axis: &str, scale: f64| {
        if glitching {
            random(format!("{}-{axis}-{}", props.seed, js_number_text(frame))).mul_add(2.0, -1.0)
                * scale
        } else {
            0.0
        }
    };
    let font = font_or(&props.font_family, FontRole::Main);
    let text_style = Style::default()
        .font_size(f64::from(props.font_size))
        .font_weight(props.font_weight.value())
        .letter_spacing(Length::em(props.letter_spacing))
        .font_family(font)
        .white_space(WhiteSpace::Pre);
    let copy = |index: usize, axis: char| -> Node {
        let color = props
            .glitch_colors
            .get(index)
            .map_or(Color::TRANSPARENT, |c| cx.color(&c.0));
        span(
            text_style
                .clone()
                .position(Position::Absolute)
                .top(Length::ZERO)
                .left(Length::ZERO)
                .mix_blend_mode(blend_of(props.blend))
                .color(color)
                .opacity(if glitching { 1.0 } else { 0.0 })
                .translate(
                    Length::px(offset(&format!("{axis}-x"), intensity)),
                    Length::px(offset(&format!("{axis}-y"), intensity * VERTICAL_SHARE)),
                ),
        )
        .child(props.text.as_str())
        .into()
    };
    Ok(div(centered_stage(&props.background))
        .child(
            div(Style::default()
                .position(Position::Relative)
                .display(Display::InlineBlock))
            .child(
                span(
                    text_style
                        .clone()
                        .position(Position::Relative)
                        .color(cx.color(&props.color)),
                )
                .child(props.text.as_str()),
            )
            .child(copy(0, 'r'))
            .child(copy(1, 'g'))
            .child(copy(2, 'b')),
        )
        .into())
}
