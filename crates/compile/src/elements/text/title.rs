use opsis_macros::{choice, settings};
use opsis_scene::node::{Node, div};
use opsis_scene::random::js_number_text;
use opsis_scene::style::{
    Align, LineHeight, Position, Style, TextAlign as SceneTextAlign, TextTransform,
};
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::{Edges, Length, Num};
use opsis_types::looks::NodeLook;
use opsis_types::numbers::float;
use serde_json::{Value, json};

use crate::elements::anim::{Entrance as LineEntrance, entrance_style, exit_opacity};
use crate::elements::backgrounds::backgrounded;
use crate::elements::draw::{Cx, DrawFailure, element as draw_element};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::{ElementCategory, ElementKind, ElementMeta, element_meta};

#[choice(export)]
pub enum TitleAlign {
    Center,
    Left,
}

#[choice(export)]
pub enum Entrance {
    #[label("Rise")]
    Rise,
    #[label("Fade")]
    Fade,
    #[label("Blur")]
    Blur,
    #[label("Scale")]
    Scale,
    #[label("None")]
    None,
    #[label("Soft blur in")]
    SoftBlur,
    #[label("Typewriter")]
    Typewriter,
    #[label("Staggered fade up")]
    FadeUp,
    #[label("Word push")]
    WordPush,
    #[label("Mask reveal up")]
    MaskReveal,
    #[label("Tracking in")]
    Tracking,
    #[label("Colour glitch")]
    Glitch,
    #[label("Decode")]
    Decode,
    #[label("Line by line slide")]
    LineByLine,
    #[label("Animated headline")]
    Headline,
    #[label("Handwriting")]
    Handwriting,
    #[label("Neon sign")]
    Neon,
    #[label("Gradient scale cut")]
    ScaleCut,
}

#[settings(export)]
pub struct Title {
    #[setting("Eyebrow")]
    pub eyebrow: String,
    #[setting("Title", default = "Sign up in 10 seconds")]
    pub title: String,
    #[setting("Subtitle")]
    pub subtitle: String,
    #[setting("Alignment", default = TitleAlign::Center)]
    pub align: TitleAlign,
    #[setting(
        "Entrance",
        default = Entrance::Rise,
        help = "Rise, Fade, Blur and Scale bring every line in. The others are Animated text's animations: the title plays it and the other lines fade in."
    )]
    pub entrance: Entrance,
    #[setting("Text colour", default = "#fafafa", check = colour)]
    pub color: String,
    #[setting("Accent", default = "#a78bfa", check = colour)]
    pub accent: String,
    #[setting(
        "Background (colour, gradient or 'transparent')",
        default = "transparent"
    )]
    pub background: String,
    #[setting("Font size", default = 112, range(min = 24, max = 260), step = 1)]
    pub font_size: u32,
    #[setting("Font weight", default = 760, range(min = 100, max = 900), step = 10)]
    pub font_weight: u32,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project."
    )]
    pub font_family: String,
    #[setting(
        "Side margin",
        default = 9.0,
        range(min = 0, max = 30),
        unit = "%",
        advanced
    )]
    pub side_margin: f64,
    #[setting(
        "Line gap",
        default = 0.22,
        range(min = 0, max = 1),
        unit = "em",
        advanced
    )]
    pub line_gap: f64,
    #[setting(
        "Eyebrow size",
        default = 0.26,
        range(min = 0.1, max = 0.8),
        unit = "× title",
        advanced
    )]
    pub eyebrow_size: f64,
    #[setting("Uppercase eyebrow", default = true, advanced)]
    pub eyebrow_uppercase: bool,
    #[setting(
        "Subtitle size",
        default = 0.34,
        range(min = 0.15, max = 0.8),
        unit = "× title",
        advanced
    )]
    pub subtitle_size: f64,
    #[setting("Letter spacing", default = -0.035, range(min = -0.1, max = 0.3), unit = "em", advanced)]
    pub letter_spacing: f64,
    #[setting(
        "Line stagger",
        default = 1.0,
        range(min = 0, max = 4),
        unit = "×",
        advanced
    )]
    pub line_stagger: f64,
    #[setting(
        "Fade out",
        default = 0.3333,
        range(min = 0.0333, max = 2),
        unit = "s",
        advanced
    )]
    pub fade_out: f64,
    #[setting(
        "Subtitle colour",
        widget = "color",
        zero_label = "Text colour, faded",
        check = colour_or_empty,
        advanced
    )]
    pub subtitle_color: String,
}

pub fn definition() -> ElementMeta {
    meta::<Title>(
        element(
            ElementKind::Component,
            "title",
            "Title card",
            "A full-screen title with an optional eyebrow and subtitle.",
            ElementCategory::Text,
            75,
        )
        .look(NodeLook::of("lucide:letter-h")),
    )
}

const SUBTITLE_FADE: f64 = 0.7;
const ANIMATED_LINE_HEIGHT: f64 = 1.15;
const SIDE_ALIGN_WEIGHT_MIN: f64 = 400.0;
const SIDE_ALIGN_WEIGHT_MAX: f64 = 700.0;

struct LineLook {
    size_em: f64,
    delay: f64,
    style: Style,
}

const fn animated_key(entrance: Entrance) -> Option<&'static str> {
    Some(match entrance {
        Entrance::SoftBlur => "soft-blur-in",
        Entrance::Typewriter => "typewriter",
        Entrance::FadeUp => "staggered-fade-up",
        Entrance::WordPush => "word-push",
        Entrance::MaskReveal => "mask-reveal-up",
        Entrance::Tracking => "tracking-in",
        Entrance::Glitch => "rgb-glitch-text",
        Entrance::Decode => "matrix-decode",
        Entrance::LineByLine => "line-by-line-slide",
        Entrance::Headline => "kinetic-headline",
        Entrance::Handwriting => "handwriting-text",
        Entrance::Neon => "neon-flicker-text",
        Entrance::ScaleCut => "gradient-scale-cut-text",
        Entrance::Rise | Entrance::Fade | Entrance::Blur | Entrance::Scale | Entrance::None => {
            return None;
        }
    })
}

const fn line_entrance(entrance: Entrance) -> LineEntrance {
    match entrance {
        Entrance::Rise => LineEntrance::Rise,
        Entrance::Blur => LineEntrance::Blur,
        Entrance::Scale => LineEntrance::Scale,
        Entrance::None => LineEntrance::None,
        _ => LineEntrance::Fade,
    }
}

fn weight_for(usual: Option<&Value>, weight: f64) -> Value {
    if usual.is_some_and(Value::is_string) {
        let rounded =
            ((weight / 100.0).round() * 100.0).clamp(SIDE_ALIGN_WEIGHT_MIN, SIDE_ALIGN_WEIGHT_MAX);
        Value::String(js_number_text(rounded))
    } else {
        json!(weight)
    }
}

fn animated_line(props: &Title, key: &str, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let mut drawn_props = element_meta(key).map_or_default(|meta| meta.default_props.clone());
    let weight = weight_for(drawn_props.get("fontWeight"), f64::from(props.font_weight));
    let family = props_font(&props.font_family);
    for (name, value) in [
        ("text", json!(props.title)),
        ("color", json!(props.color)),
        ("fontSize", json!(props.font_size)),
        ("fontWeight", weight),
        ("fontFamily", json!(family)),
        ("letterSpacing", json!(props.letter_spacing)),
        ("background", json!("transparent")),
    ] {
        drawn_props.insert(name.to_owned(), value);
    }
    let lines = float(props.title.split('\n').count().max(1));
    let inner = draw_element(key, &drawn_props, cx)
        .map_err(DrawFailure::Element)?
        .unwrap_or_else(|| div(Style::default()).into());
    Ok(div(Style::default()
        .position(Position::Relative)
        .width(Length::percent(100.0))
        .height(f64::from(props.font_size) * ANIMATED_LINE_HEIGHT * lines))
    .child(inner)
    .into())
}

fn props_font(family: &str) -> String {
    if family.is_empty() {
        "\"df-heading\"".to_owned()
    } else {
        family.to_owned()
    }
}

fn title_line(
    props: &Title,
    look: &LineLook,
    text: &str,
    color: &str,
    cx: &Cx<'_>,
) -> Result<Node, DrawFailure> {
    let style = entrance_style(
        line_entrance(props.entrance),
        cx.frame(),
        cx.fps(),
        look.delay * props.line_stagger,
    )?
    .color(cx.color(color))
    .font_size(f64::from(props.font_size) * look.size_em)
    .merged(&look.style);
    Ok(div(style).child(text).into())
}

pub fn draw(props: &Title, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let out = exit_opacity(
        cx.frame(),
        cx.duration(),
        Some((props.fade_out * cx.fps()).round()),
    )?;
    let mut lines = Vec::new();
    if !props.eyebrow.is_empty() {
        lines.push(title_line(
            props,
            &LineLook {
                size_em: props.eyebrow_size,
                delay: 0.0,
                style: Style::default()
                    .font_weight(700.0)
                    .letter_spacing(Length::em(0.18))
                    .text_transform(if props.eyebrow_uppercase {
                        TextTransform::Uppercase
                    } else {
                        TextTransform::None
                    }),
            },
            &props.eyebrow,
            &props.accent,
            cx,
        )?);
    }
    lines.push(match animated_key(props.entrance) {
        Some(key) => animated_line(props, key, cx)?,
        None => title_line(
            props,
            &LineLook {
                size_em: 1.0,
                delay: 4.0,
                style: Style::default()
                    .font_weight(f64::from(props.font_weight))
                    .line_height(LineHeight::Number(Num(1.02)))
                    .max_width(Length::percent(100.0))
                    .letter_spacing(Length::em(props.letter_spacing)),
            },
            &props.title,
            &props.color,
            cx,
        )?,
    });
    if !props.subtitle.is_empty() {
        let faded = props.subtitle_color.is_empty();
        lines.push(title_line(
            props,
            &LineLook {
                size_em: props.subtitle_size,
                delay: 10.0,
                style: Style::default()
                    .font_weight(450.0)
                    .max_width(Length::percent(80.0))
                    .opacity(if faded { SUBTITLE_FADE } else { 1.0 }),
            },
            &props.subtitle,
            if faded {
                &props.color
            } else {
                &props.subtitle_color
            },
            cx,
        )?);
    }
    let centered = props.align == TitleAlign::Center;
    let body = div(absolute_fill()
        .align_items(if centered {
            Align::Center
        } else {
            Align::FlexStart
        })
        .justify_content(Align::Center)
        .padding(Edges::axes(
            Length::ZERO,
            Length::percent(props.side_margin),
        ))
        .text_align(if centered {
            SceneTextAlign::Center
        } else {
            SceneTextAlign::Left
        })
        .font_family(font_or(&props.font_family, FontRole::Heading))
        .opacity(out)
        .gap(f64::from(props.font_size) * props.line_gap))
    .children(lines);
    Ok(div(absolute_fill())
        .child(backgrounded(&props.background, Vec::new()))
        .child(body)
        .into())
}
