use opsis_macros::{choice, settings};
use opsis_scene::css::with_background;
use opsis_scene::node::{Node, div, span};
use opsis_scene::paint::{Shadow, Transform};
use opsis_scene::spring::{DEFAULT_SPRING, SpringConfig};
use opsis_scene::style::{Align, Display, FlexWrap, Style};
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::{Edges, Length, Num};
use opsis_types::values::Caption;
use serde::Deserialize;

use crate::elements::anim::sprung;
use crate::elements::draw::{Cx, DrawFailure, css_color};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::overlay_frame::{Insets, OverlayFrame, overlay_frame, overlay_padding};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

pub const CAPTIONS_ELEMENT: &str = "captions";

#[choice(export)]
pub enum CaptionStyle {
    WordByWord,
    Karaoke,
    Line,
}

#[choice(export)]
pub enum CaptionPosition {
    Bottom,
    Center,
    Top,
}

#[settings(export)]
pub struct Captions {
    #[setting("Style", default = CaptionStyle::WordByWord)]
    pub style: CaptionStyle,
    #[setting("Position", default = CaptionPosition::Bottom)]
    pub position: CaptionPosition,
    #[setting("Font size", default = 56, range(min = 16, max = 140), step = 1)]
    pub font_size: u32,
    #[setting("Text colour", default = "#ffffff", check = colour)]
    pub color: String,
    #[setting("Highlight colour", default = "#facc15", check = colour)]
    pub highlight_color: String,
    #[setting(
        "Panel colour",
        default = "rgba(0,0,0,0.35)",
        help = "The box behind the words. Lower its opacity to let the video show through.",
        check = colour
    )]
    pub panel: String,
    #[setting("Font weight", default = 800, range(min = 100, max = 900), step = 10)]
    pub font_weight: u32,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project."
    )]
    pub font_family: String,
    #[setting("Words per line", default = 6, range(min = 1, max = 20), step = 1)]
    pub words_per_line: u32,
    #[setting(
        "Stay after the last word",
        default = 250,
        range(min = 0, max = 2000),
        step = 50,
        unit = "ms",
        advanced
    )]
    pub linger_ms: u32,
    #[setting(
        "Distance from the edge",
        default = 90,
        range(min = 0, max = 400),
        step = 1,
        unit = "px",
        advanced
    )]
    pub edge_offset: u32,
    #[setting(
        "Side margin",
        default = 10.0,
        range(min = 0, max = 40),
        unit = "%",
        advanced
    )]
    pub side_margin: f64,
    #[setting(
        "Panel padding",
        default = 22,
        range(min = 0, max = 80),
        step = 1,
        unit = "px",
        advanced
    )]
    pub panel_padding: u32,
    #[setting(
        "Panel corners",
        default = 16,
        range(min = 0, max = 60),
        step = 1,
        unit = "px",
        advanced
    )]
    pub panel_radius: u32,
    #[setting("Text shadow", default = true, advanced)]
    pub text_shadow: bool,
    #[setting(
        "Unspoken words",
        default = 55.0,
        range(min = 0, max = 100),
        unit = "%",
        advanced
    )]
    pub unspoken_opacity: f64,
}

pub fn definition() -> ElementMeta {
    meta::<Captions>(element(
        ElementKind::Component,
        CAPTIONS_ELEMENT,
        "Captions",
        "Captions shown over the video.",
        ElementCategory::Overlay,
        150,
    ))
}

const WORD_GAP_EM: f64 = 0.28;
const BOX_PAD_SHARE: f64 = 10.0 / 22.0;
const POP_FROM: f64 = 0.85;
const POP_GROWTH: f64 = 0.15;
const POP_SPRING: SpringConfig = SpringConfig {
    damping: 14.0,
    stiffness: 220.0,
    ..DEFAULT_SPRING
};

#[derive(Debug, Clone, Deserialize)]
pub struct CaptionsProps {
    #[serde(flatten)]
    pub settings: Captions,
    #[serde(default)]
    pub captions: Vec<Caption>,
}

fn ends_sentence(text: &str) -> bool {
    text.ends_with(['.', '!', '?'])
}

pub fn group_captions(captions: &[Caption], words_per_line: u32) -> Vec<Vec<Caption>> {
    let per_line = usize::try_from(words_per_line).unwrap_or(usize::MAX);
    let mut lines = Vec::new();
    let mut line = Vec::new();
    for caption in captions {
        line.push(caption.clone());
        if line.len() >= per_line || ends_sentence(&caption.text) {
            lines.push(std::mem::take(&mut line));
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

fn frames_to_ms(frames: f64, fps: f64) -> f64 {
    (frames * 1000.0 / fps).round()
}

fn ms_to_frames(ms: f64, fps: f64) -> f64 {
    (ms / 1000.0 * fps).round().max(0.0)
}

fn caption_word(
    settings: &Captions,
    word: &Caption,
    t_ms: f64,
    cx: &Cx<'_>,
) -> Result<Option<Node>, DrawFailure> {
    let active = t_ms >= word.start_ms && t_ms < word.end_ms;
    let spoken = t_ms >= word.start_ms;
    if settings.style == CaptionStyle::WordByWord && !spoken {
        return Ok(None);
    }
    let pop = sprung(
        cx.frame() - ms_to_frames(word.start_ms, cx.fps()),
        cx.fps(),
        POP_SPRING,
    )?;
    let lit = match settings.style {
        CaptionStyle::Karaoke => spoken,
        CaptionStyle::WordByWord => active,
        CaptionStyle::Line => false,
    };
    let mut style = Style::default()
        .color(cx.color(if lit {
            &settings.highlight_color
        } else {
            &settings.color
        }))
        .display(Display::InlineBlock)
        .opacity(if settings.style == CaptionStyle::Karaoke && !spoken {
            settings.unspoken_opacity / 100.0
        } else {
            1.0
        });
    if settings.style == CaptionStyle::WordByWord {
        let scale = pop.mul_add(POP_GROWTH, POP_FROM);
        style = style.transform(vec![Transform::Scale(Num(scale), Num(scale), Num(1.0))]);
    }
    Ok(Some(span(style).child(word.text.as_str()).into()))
}

const fn justify_at(position: CaptionPosition) -> Align {
    match position {
        CaptionPosition::Top => Align::FlexStart,
        CaptionPosition::Center => Align::Center,
        CaptionPosition::Bottom => Align::FlexEnd,
    }
}

pub fn draw(props: &CaptionsProps, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let settings = &props.settings;
    let OverlayFrame { k, insets } = overlay_frame(cx.at.stage);
    let t_ms = frames_to_ms(cx.frame(), cx.fps());
    let lines = group_captions(&props.captions, settings.words_per_line);
    let linger = f64::from(settings.linger_ms);
    let Some(line) = lines.iter().find(|l| {
        l.first().is_some_and(|w| t_ms >= w.start_ms)
            && l.last().is_some_and(|w| t_ms <= w.end_ms + linger)
    }) else {
        return Ok(div(absolute_fill()).into());
    };
    let size = f64::from(settings.font_size);
    let pad_x = f64::from(settings.panel_padding);
    let pad_y = pad_x * BOX_PAD_SHARE;
    let mut words = Vec::new();
    for word in line {
        words.extend(caption_word(settings, word, t_ms, cx)?);
    }
    let mut box_style = with_background(Style::default(), &settings.panel)
        .display(Display::Flex)
        .flex_wrap(FlexWrap::Wrap)
        .justify_content(Align::Center)
        .row_gap(Length::ZERO)
        .column_gap(size * WORD_GAP_EM * k)
        .font_family(font_or(&settings.font_family, FontRole::Main))
        .font_size(size * k)
        .font_weight(f64::from(settings.font_weight))
        .padding(Edges::axes(Length::px(pad_y * k), Length::px(pad_x * k)))
        .border_radius_all(f64::from(settings.panel_radius) * k);
    if settings.text_shadow {
        box_style = box_style.text_shadow(vec![Shadow {
            inset: false,
            x: Length::ZERO,
            y: Length::px(4.0),
            blur: Length::px(24.0),
            spread: Length::ZERO,
            color: css_color("rgba(0,0,0,0.55)"),
        }]);
    }
    let vertical = Insets {
        top: insets.top,
        bottom: insets.bottom,
        ..Insets::default()
    };
    let fill = overlay_padding(
        absolute_fill()
            .justify_content(justify_at(settings.position))
            .align_items(Align::Center),
        f64::from(settings.edge_offset) * k,
        vertical,
    )
    .padding_left(Length::percent(settings.side_margin))
    .padding_right(Length::percent(settings.side_margin));
    Ok(div(fill).child(div(box_style).children(words)).into())
}
