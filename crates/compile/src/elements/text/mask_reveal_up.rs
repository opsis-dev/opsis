use opsis_macros::settings;
use opsis_scene::easing::Easing;
use opsis_scene::node::{Node, div, span};
use opsis_scene::paint::Filter;
use opsis_scene::style::{Display, LineHeight, Style};
use opsis_scene::values::{Length, Num};
use opsis_types::looks::NodeLook;
use opsis_types::numbers::float;

use crate::elements::anim::clamped_eased;
use crate::elements::backgrounds::centered_stage;
use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::shared::{FontWeight, TextAlign};
use crate::elements::text::typewriter::text_align;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct MaskRevealUp {
    #[setting(
        "Text",
        default = "Designed to move.\nBuilt to focus.",
        widget = "textarea"
    )]
    pub text: String,
    #[setting(
        "Distance",
        default = 30,
        range(min = 0, max = 120),
        step = 1,
        advanced
    )]
    pub distance: u32,
    #[setting("Font size", default = 72, range(min = 12, max = 160), step = 1)]
    pub font_size: u32,
    #[setting("Color", default = "#171717", check = colour)]
    pub color: String,
    #[setting("Font weight", default = FontWeight::W600)]
    pub font_weight: FontWeight,
    #[setting(
        "Background",
        default = "white",
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
    #[setting(
        "Align",
        default = TextAlign::Center
    )]
    pub align: TextAlign,
    #[setting(
        "Line stagger",
        default = 0.1,
        range(min = 0, max = 1),
        step = 0.0333,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub line_stagger: f64,
    #[setting(
        "Exit distance",
        default = 22,
        range(min = 0, max = 120),
        step = 1,
        unit = "px",
        advanced
    )]
    pub exit_distance: u32,
    #[setting(
        "Blur",
        default = 6,
        range(min = 0, max = 30),
        step = 1,
        unit = "px",
        advanced
    )]
    pub blur: u32,
    #[setting("Line height", default = 1.1, range(min = 0.7, max = 2), advanced)]
    pub line_height: f64,
}

pub fn definition() -> ElementMeta {
    meta::<MaskRevealUp>(
        element(
            ElementKind::Component,
            "mask-reveal-up",
            "Mask reveal up",
            "Lines reveal upward with a soft masked feel and compact stagger.",
            ElementCategory::Text,
            90,
        )
        .authored_clock()
        .look(NodeLook::of("lucide:type")),
    )
}

const ENTER_FRAMES: f64 = 23.0;
const ENTER_TRAVEL: f64 = 11.0;
const EXIT_FRAMES: f64 = 16.0;
const EXIT_TRAVEL_FROM: f64 = 8.0;
const EXIT_STAGGER: f64 = 2.0;
const LETTER_SPACING_EM: f64 = -0.03;

const fn enter_easing() -> Easing {
    Easing::Bezier(0.22, 1.0, 0.36, 1.0)
}

const fn exit_easing() -> Easing {
    Easing::Bezier(0.64, 0.0, 0.78, 0.0)
}

pub fn draw(props: &MaskRevealUp, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let frame = cx.authored_frame() * props.speed;
    let duration = cx.authored_duration() * props.speed;
    let lines: Vec<&str> = props.text.split('\n').collect();
    let extra = float(lines.len().saturating_sub(1));
    let enter_end = extra.mul_add(props.line_stagger, ENTER_FRAMES);
    let exit_start = enter_end.max(extra.mul_add(-EXIT_STAGGER, duration - EXIT_FRAMES));
    let softness = f64::from(props.blur);
    let spans = lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let enter_local = float(i).mul_add(-props.line_stagger, frame);
            let exit_local = float(i).mul_add(-EXIT_STAGGER, frame - exit_start);
            let enter =
                clamped_eased(enter_local, [0.0, ENTER_FRAMES], [0.0, 1.0], enter_easing())?;
            let exit = clamped_eased(exit_local, [0.0, EXIT_FRAMES], [0.0, 1.0], exit_easing())?;
            let y_enter = clamped_eased(
                enter_local,
                [0.0, ENTER_TRAVEL],
                [f64::from(props.distance), 0.0],
                enter_easing(),
            )?;
            let y_exit = clamped_eased(
                exit_local,
                [EXIT_TRAVEL_FROM, EXIT_FRAMES],
                [0.0, -f64::from(props.exit_distance)],
                exit_easing(),
            )?;
            let blur_enter = clamped_eased(
                enter_local,
                [0.0, ENTER_FRAMES],
                [softness, 0.0],
                enter_easing(),
            )?;
            let blur_exit = clamped_eased(
                exit_local,
                [0.0, EXIT_FRAMES],
                [0.0, softness],
                exit_easing(),
            )?;
            Ok(span(
                Style::default()
                    .display(Display::Block)
                    .transform_origin((Length::percent(50.0), Length::percent(50.0), Length::ZERO))
                    .opacity(enter * (1.0 - exit))
                    .translate(Length::ZERO, Length::px(y_enter + y_exit))
                    .filter(vec![Filter::Blur(Length::px(blur_enter + blur_exit))]),
            )
            .child(*line)
            .into())
        })
        .collect::<Result<Vec<Node>, DrawFailure>>()?;
    Ok(div(centered_stage(&props.background))
        .child(
            span(
                Style::default()
                    .font_size(f64::from(props.font_size))
                    .font_weight(props.font_weight.value())
                    .color(cx.color(&props.color))
                    .letter_spacing(Length::em(LETTER_SPACING_EM))
                    .line_height(LineHeight::Number(Num(props.line_height)))
                    .text_align(text_align(&props.align))
                    .font_family(font_or(&props.font_family, FontRole::Main)),
            )
            .children(spans),
        )
        .into())
}
