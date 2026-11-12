use opsis_macros::settings;
use opsis_scene::color::Color;
use opsis_scene::easing::Easing;
use opsis_scene::interpolate::{Interpolation, interpolate};
use opsis_scene::node::{Node, div, span};
use opsis_scene::paint::{
    ColorStop, Filter, Gradient as Painted, MaskLayer, MaskMode, Repeat, Transform,
};
use opsis_scene::style::{Display, LineHeight, Overflow, Position, Style, WhiteSpace};
use opsis_scene::values::{Angle, Length, Num};
use opsis_types::looks::NodeLook;

use crate::elements::anim::{clamped, clamped_eased};
use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::shared::FontWeight;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct GradientScaleCutText {
    #[setting("Text", default = "Introducing")]
    pub text: String,
    #[setting(
        "Giant font size",
        default = 520,
        range(min = 160, max = 720),
        step = 4
    )]
    pub giant_font_size: u32,
    #[setting(
        "Compact font size",
        default = 132,
        range(min = 40, max = 240),
        step = 2
    )]
    pub compact_font_size: u32,
    #[setting(
        "Final horizontal anchor offset",
        default = -16,
        range(min = -400, max = 400),
        step = 4,
        advanced
    )]
    pub anchor_offset_x: i32,
    #[setting(
        "Giant phase left travel",
        default = 400,
        range(min = 0, max = 800),
        step = 4,
        advanced
    )]
    pub giant_travel: u32,
    #[setting(
        "Compact entrance travel",
        default = 160,
        range(min = 0, max = 400),
        step = 4,
        advanced
    )]
    pub settle_travel: u32,
    #[setting(
        "Hard cut at",
        default = 0.4333,
        range(min = 0.1333, max = 0.8),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub cut_frame: f64,
    #[setting(
        "Reveal edge softness",
        default = 14,
        range(min = 0, max = 40),
        step = 1,
        advanced
    )]
    pub reveal_softness: u32,
    #[setting(
        "Blur after the cut",
        default = 12,
        range(min = 0, max = 40),
        step = 1,
        advanced
    )]
    pub compact_blur: u32,
    #[setting("Gradient start", default = "#f04a14", check = colour)]
    pub gradient_start: String,
    #[setting("Gradient end", default = "#f3f1f1", check = colour)]
    pub gradient_end: String,
    #[setting("Unrevealed text color", default = "#170b0a", check = colour, advanced)]
    pub ghost_color: String,
    #[setting("Background color", default = "#000000", check = colour)]
    pub background_color: String,
    #[setting("Font weight", default = FontWeight::W700)]
    pub font_weight: FontWeight,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project."
    )]
    pub font_family: String,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4), step = 0.25)]
    pub speed: f64,
    #[setting(
        "Reveal length",
        default = 1.1,
        range(min = 0.3333, max = 3),
        step = 0.0333,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub reveal_end: f64,
    #[setting(
        "Compact text width",
        default = 87.0,
        range(min = 50, max = 120),
        unit = "%",
        advanced
    )]
    pub compact_width: f64,
    #[setting(
        "Gradient spread",
        default = 26.0,
        range(min = 5, max = 100),
        unit = "%",
        advanced
    )]
    pub gradient_spread: f64,
    #[setting("Letter spacing", default = 0.0, range(min = -0.2, max = 0.5), unit = "em", advanced)]
    pub letter_spacing: f64,
}

pub fn definition() -> ElementMeta {
    meta::<GradientScaleCutText>(
            element(
                ElementKind::Component,
                "gradient-scale-cut-text",
                "Gradient scale cut text",
                "An oversized gradient reveal snaps to a separate compact blurred line that settles left while the reveal continues.",
                ElementCategory::Text,
                36,
            )
                .authored_clock()
                .design(1920, 1080)
                .look(NodeLook::of("lucide:type")),
        )
}

const SETTLE_FRAMES: f64 = 8.0;
const GIANT_Y: f64 = -28.0;
const GIANT_X_CORRECTION: f64 = -48.0;
const COMPACT_Y: f64 = 5.0;
const COMPACT_START_X_CORRECTION: f64 = 28.0;
const LINE_HEIGHT: f64 = 1.12;
const START_HOLD_PERCENT: f64 = 4.0;

struct Layer<'a> {
    text: &'a str,
    face: &'a [String],
    font_size: f64,
    font_weight: f64,
    gradient: (Color, Color),
    ghost: Color,
    mask: MaskLayer,
    spread: f64,
    letter_spacing: f64,
}

fn text_layer(layer: Layer<'_>) -> Node {
    let text_style = Style::default()
        .display(Display::Block)
        .font_family(layer.face.to_vec())
        .font_size(layer.font_size)
        .font_weight(layer.font_weight)
        .letter_spacing(Length::em(layer.letter_spacing))
        .line_height(LineHeight::Number(Num(LINE_HEIGHT)))
        .white_space(WhiteSpace::Nowrap);
    let (start, end) = layer.gradient;
    let paint = Painted::linear(
        Angle::deg(90.0),
        vec![
            ColorStop::at(start.clone(), Length::percent(0.0)),
            ColorStop::at(start, Length::percent(START_HOLD_PERCENT)),
            ColorStop::at(end.clone(), Length::percent(layer.spread)),
            ColorStop::at(end, Length::percent(100.0)),
        ],
    );
    div(Style::default().position(Position::Relative))
        .child(span(text_style.clone().color(layer.ghost)).child(layer.text))
        .child(
            span(
                text_style
                    .position(Position::Absolute)
                    .inset(Length::ZERO)
                    .color(Color::TRANSPARENT)
                    .background(vec![paint.layer()])
                    .background_clip_text(true)
                    .mask(vec![layer.mask])
                    .opacity(1.0),
            )
            .child(layer.text),
        )
        .into()
}

fn reveal_mask(solid: f64, fade: f64) -> MaskLayer {
    let mut layer = Painted::linear(
        Angle::deg(90.0),
        vec![
            ColorStop::at(Color::BLACK, Length::percent(0.0)),
            ColorStop::at(Color::BLACK, Length::percent(solid)),
            ColorStop::at(Color::TRANSPARENT, Length::percent(fade)),
            ColorStop::at(Color::TRANSPARENT, Length::percent(fade)),
        ],
    )
    .layer();
    layer.repeat = (Repeat::NoRepeat, Repeat::NoRepeat);
    MaskLayer {
        layer,
        mode: MaskMode::Alpha,
    }
}

fn centred(offset: f64) -> Length {
    Length::sum([Length::percent(50.0), Length::px(offset)])
}

pub fn draw(props: &GradientScaleCutText, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let face = font_or(&props.font_family, FontRole::Main);
    let frame = cx.authored_frame() * props.speed.max(0.0);
    let cut = props.cut_frame.max(1.0);
    let softness = f64::from(props.reveal_softness);
    let blur = f64::from(props.compact_blur);
    let reveal = clamped_eased(
        frame,
        [0.0, props.reveal_end],
        [0.02, 1.0],
        Easing::Bezier(0.22, 0.74, 0.24, 1.0),
    )?;
    let edge = reveal * (100.0 + softness / 2.0);
    let solid_until = (edge - softness / 2.0).max(0.0);
    let fade_until = edge + softness / 2.0;
    let out_cubic = || Easing::ease_out(Easing::Cubic);
    let giant = clamped_eased(frame, [0.0, (cut - 1.0).max(1.0)], [0.0, 1.0], out_cubic())?;
    let settle = clamped_eased(frame, [cut, cut + SETTLE_FRAMES], [0.0, 1.0], out_cubic())?;
    let moved = interpolate(
        frame,
        &[cut + 1.0, cut + 2.0, cut + 4.0, cut + SETTLE_FRAMES],
        &[0.0, 0.58, 0.875, 1.0],
        &Interpolation::clamped(),
    )?;
    let compact_x = COMPACT_START_X_CORRECTION.mul_add(
        1.0 - moved,
        clamped(moved, [0.0, 1.0], [f64::from(props.settle_travel), 0.0])?,
    );
    let anchor = f64::from(props.anchor_offset_x);
    let giant_shown = frame < cut;
    let layer = Layer {
        text: &props.text,
        face: &face,
        font_size: f64::from(if giant_shown {
            props.giant_font_size.max(1)
        } else {
            props.compact_font_size.max(1)
        }),
        font_weight: props.font_weight.value(),
        gradient: (
            cx.color(&props.gradient_start),
            cx.color(&props.gradient_end),
        ),
        ghost: cx.color(&props.ghost_color),
        mask: reveal_mask(solid_until, fade_until),
        spread: props.gradient_spread,
        letter_spacing: props.letter_spacing,
    };
    let holder = if giant_shown {
        Style::default()
            .position(Position::Absolute)
            .top(centred(GIANT_Y))
            .left(centred(f64::mul_add(
                f64::from(props.giant_travel),
                -giant,
                anchor + GIANT_X_CORRECTION,
            )))
            .opacity(1.0)
            .transform(vec![Transform::Translate(
                Length::ZERO,
                Length::percent(-50.0),
                Length::ZERO,
            )])
    } else {
        Style::default()
            .position(Position::Absolute)
            .top(centred(COMPACT_Y))
            .left(centred(anchor + compact_x))
            .filter(vec![Filter::Blur(Length::px(blur * (1.0 - settle)))])
            .opacity(1.0)
            .transform(vec![
                Transform::Translate(Length::percent(-50.0), Length::percent(-50.0), Length::ZERO),
                Transform::Scale(Num(props.compact_width / 100.0), Num(1.0), Num(1.0)),
            ])
            .transform_origin((Length::percent(50.0), Length::percent(50.0), Length::ZERO))
    };
    Ok(div(Style::default()
        .position(Position::Absolute)
        .inset(Length::ZERO)
        .overflow(Overflow::Hidden)
        .background_color(cx.color(&props.background_color)))
    .child(div(holder).child(text_layer(layer)))
    .into())
}
