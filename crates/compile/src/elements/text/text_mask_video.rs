use std::collections::HashMap;

use opsis_macros::{choice, settings};
use opsis_scene::css::with_background;
use opsis_scene::easing::Easing;
use opsis_scene::interpolate::{Interpolation, interpolate};
use opsis_scene::node::{Node, SvgElement, div, svg};
use opsis_scene::paint::{ClipPath, Transform};
use opsis_scene::style::{Align, Position, Style, TextAlign};
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::{Edges, Length, Num};
use opsis_types::looks::NodeLook;
use opsis_types::numbers::float;
use opsis_types::values::MediaKind;
use serde::Deserialize;

use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::media::image::{filled, footage_frame_url};
use crate::elements::parts::split_chars::scale_font;
use crate::elements::shared::Fit;
use crate::elements::units::js_fixed;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

const GLYPH_WIDTH: f64 = 0.72;
const ARRIVE: Easing = Easing::Bezier(0.33, 1.0, 0.68, 1.0);
const ENTER: Easing = Easing::Bezier(0.16, 1.0, 0.3, 1.0);
const ENTER_OPACITY_LEAD: f64 = 0.55;
const CLIP_ID: &str = "text-mask";
const OVERSCAN_PERCENT: f64 = -8.0;

#[choice(export)]
pub enum MaskReveal {
    Left,
    Right,
    Up,
    Down,
    None,
}

#[settings(export)]
pub struct TextMaskVideo {
    #[setting(
        "Text",
        default = "LAUNCH\nDAY",
        widget = "textarea",
        help = "The letterforms the media is seen through."
    )]
    pub text: String,
    #[setting(
        "Media",
        widget = "asset",
        accept = "video/*,image/*",
        zero_label = "Gradient",
        help = "A video or image to show through the letters. Leave empty for the gradient."
    )]
    pub src: String,
    #[setting(
        "Gradient fill",
        default = "linear-gradient(115deg, #e8b86d 0%, #f472b6 42%, #2dd4bf 100%)",
        widget = "gradient",
        show_if = { "src": [""] }
    )]
    pub gradient: String,
    #[setting("Reveal", default = MaskReveal::Left, help = "Which way the letters are uncovered.")]
    pub reveal: MaskReveal,
    #[setting("Drift", default = 26.0, advanced)]
    pub drift: f64,
    #[setting(
        "Zoom",
        default = 0.12,
        step = 0.01,
        help = "Extra scale the media breathes through. 0 holds it still.",
        advanced
    )]
    pub zoom: f64,
    #[setting(
        "Animation length",
        default = 30,
        range(min = 1, max = 300),
        step = 1,
        unit = "frames",
        help = "Length of the reveal.",
        advanced
    )]
    pub duration_in_frames: u32,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project."
    )]
    pub font_family: String,
    #[setting("Font weight", default = 800)]
    pub font_weight: u32,
    #[setting("Letter spacing", advanced)]
    pub letter_spacing: f64,
    #[setting("Line height", default = 0.98, step = 0.01, advanced)]
    pub line_height: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Font size",
        range(min = 12, max = 600),
        zero_label = "Fit",
        unit = "px",
        help = "Leave empty to fit the text to the frame.",
        "type" = "number"
    )]
    pub font_size: Option<f64>,
    #[setting(
        "Drift speed",
        default = 1.0,
        range(min = 0, max = 4),
        unit = "×",
        advanced
    )]
    pub drift_speed: f64,
    #[setting(
        "Largest fitted size",
        default = 150.0,
        range(min = 40, max = 600),
        unit = "px per 1080 px of width",
        advanced
    )]
    pub fit_max: f64,
    #[setting("Video starts at", default = 0.0, range(min = 0), unit = "s", advanced)]
    pub start_from: f64,
}

pub fn definition() -> ElementMeta {
    meta::<TextMaskVideo>(
        element(
            ElementKind::Component,
            "text-mask-video",
            "Text mask",
            "Big letters with a video, image or gradient showing through them.",
            ElementCategory::Text,
            120,
        )
        .kind_prop("media", "src")
        .look(NodeLook::of("lucide:type-outline")),
    )
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Shown {
    #[serde(flatten)]
    pub settings: TextMaskVideo,
    #[serde(default)]
    pub media: Option<MediaKind>,
    #[serde(default)]
    pub footage_urls: HashMap<String, String>,
}

fn reveal_clip(reveal: MaskReveal, hidden: f64) -> Option<ClipPath> {
    let cut = Length::percent(hidden);
    let edges = match reveal {
        MaskReveal::Left => Edges {
            top: Length::ZERO,
            right: cut,
            bottom: Length::ZERO,
            left: Length::ZERO,
        },
        MaskReveal::Right => Edges {
            top: Length::ZERO,
            right: Length::ZERO,
            bottom: Length::ZERO,
            left: cut,
        },
        MaskReveal::Up => Edges {
            top: Length::ZERO,
            right: Length::ZERO,
            bottom: cut,
            left: Length::ZERO,
        },
        MaskReveal::Down => Edges {
            top: cut,
            right: Length::ZERO,
            bottom: Length::ZERO,
            left: Length::ZERO,
        },
        MaskReveal::None => return None,
    };
    Some(ClipPath::Inset { edges, round: None })
}

fn fill_of(props: &Shown, cx: &Cx<'_>) -> Node {
    let mask = &props.settings;
    let fill = Style::default()
        .width(Length::percent(100.0))
        .height(Length::percent(100.0));
    if mask.src.is_empty() {
        return div(with_background(fill, &mask.gradient)).into();
    }
    if props.media == Some(MediaKind::Image) {
        return filled(mask.src.clone(), Fit::Cover, None);
    }
    match props.footage_urls.get("src") {
        Some(url) => filled(
            footage_frame_url(url, mask.start_from + cx.frame() / cx.fps()),
            Fit::Cover,
            None,
        ),
        None => div(with_background(fill, &mask.gradient)).into(),
    }
}

#[expect(
    clippy::suboptimal_flops,
    reason = "each step rounds on its own, and the recorded goldens hold what that draws"
)]
pub fn draw(props: &Shown, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let mask = &props.settings;
    let (video_width, video_height) = (cx.at.stage.width, cx.at.stage.height);
    let lines: Vec<&str> = mask
        .text
        .split('\n')
        .map(|l| l.strip_suffix('\r').unwrap_or(l))
        .collect();
    let count = float(lines.len());
    let longest = lines
        .iter()
        .map(|l| float(l.chars().count()))
        .fold(1.0f64, f64::max);
    let fitted_size = scale_font(mask.fit_max, video_width)
        .min(video_width * 0.9 / longest / GLYPH_WIDTH - mask.letter_spacing / GLYPH_WIDTH)
        .min(video_height * 0.85 / (count * mask.line_height * 1.25));
    let font_size = mask.font_size.unwrap_or_else(|| fitted_size.max(1.0));
    let width = opsis_scene::clock::js_round(
        (font_size * 6.2).max(longest * (font_size * GLYPH_WIDTH + mask.letter_spacing)),
    );
    let height = opsis_scene::clock::js_round(font_size * mask.line_height * count * 1.25);
    let duration = f64::from(mask.duration_in_frames).round().max(1.0);
    let frame = cx.frame();
    let enter_time = interpolate(
        frame,
        &[0.0, duration],
        &[0.0, 1.0],
        &Interpolation::clamped(),
    )?;
    let motion = interpolate(
        enter_time,
        &[0.0, 1.0],
        &[0.0, 1.0],
        &Interpolation::clamped().eased(ARRIVE),
    )?;
    let opacity = interpolate(
        enter_time,
        &[0.0, ENTER_OPACITY_LEAD],
        &[0.0, 1.0],
        &Interpolation::clamped().eased(ENTER),
    )?;
    let revealed = if mask.reveal == MaskReveal::None {
        1.0
    } else {
        motion
    };
    let hidden = ((1.0 - revealed) * 100.0).clamp(0.0, 100.0);
    let seconds = frame / cx.fps() * mask.drift_speed;
    let pan = (seconds * 0.62).sin() * mask.drift;
    let scale = 1.0 + mask.zoom * (0.5 + 0.5 * (seconds * 0.44 + 1.1).sin());
    let middle_x = width / 2.0;
    let block_height = (count - 1.0) * font_size * mask.line_height;
    let first_y = height / 2.0 - block_height / 2.0;
    let family = font_or(&mask.font_family, FontRole::Heading).join(", ");
    let mut clip = SvgElement::new("clipPath")
        .attr("id", CLIP_ID)
        .attr("clipPathUnits", "userSpaceOnUse");
    for (index, line) in lines.iter().enumerate() {
        clip = clip.child(
            SvgElement::new("text")
                .attr("x", middle_x)
                .attr("y", first_y + float(index) * font_size * mask.line_height)
                .attr("text-anchor", "middle")
                .attr("dominant-baseline", "central")
                .attr("font-size", font_size)
                .attr("font-weight", mask.font_weight)
                .attr("letter-spacing", mask.letter_spacing)
                .attr("font-family", family.as_str())
                .text(*line),
        );
    }
    let defs = svg(
        SvgElement::new("svg")
            .attr("width", 0.0)
            .attr("height", 0.0)
            .child(SvgElement::new("defs").child(clip)),
        Style::default()
            .position(Position::Absolute)
            .width(Length::ZERO)
            .height(Length::ZERO),
    );
    let pan_text = js_fixed(-pan, 2).parse::<f64>().unwrap_or(0.0);
    let scale_text = js_fixed(scale, 4).parse::<f64>().unwrap_or(1.0);
    let moving = div(Style::default()
        .position(Position::Absolute)
        .inset(Length::percent(OVERSCAN_PERCENT))
        .transform(vec![
            Transform::Translate(Length::px(pan_text), Length::ZERO, Length::ZERO),
            Transform::Scale(Num(scale_text), Num(scale_text), Num(1.0)),
        ]))
    .child(fill_of(props, cx));
    let mut revealing = Style::default()
        .position(Position::Absolute)
        .inset(Length::ZERO);
    if let Some(cut) = reveal_clip(mask.reveal, hidden) {
        revealing = revealing.clip_path(cut);
    }
    let masked = div(Style::default()
        .position(Position::Absolute)
        .inset(Length::ZERO)
        .clip_path(ClipPath::Url(format!("#{CLIP_ID}"))))
    .child(div(revealing).child(moving));
    Ok(div(absolute_fill()
        .align_items(Align::Center)
        .justify_content(Align::Center)
        .text_align(TextAlign::Center))
    .child(
        div(Style::default()
            .position(Position::Relative)
            .width(Length::px(width))
            .height(Length::px(height))
            .opacity(opacity))
        .child(defs)
        .child(masked),
    )
    .into())
}
