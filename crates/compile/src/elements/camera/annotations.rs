use opsis_scene::clock::js_round;
use opsis_scene::easing::Easing;
use opsis_scene::keyed::percent;
use opsis_scene::node::{Node, SvgElement, div, span, svg};
use opsis_scene::paint::{BlendMode, Shadow, Transform};
use opsis_scene::style::{
    BorderSide, BorderStyle, ContainerType, Display, Overflow, Position, Style, WhiteSpace,
};
use opsis_scene::values::{Angle, Edges, Length, Num};
use opsis_types::actions::{ActionLog, Rect, Viewport};
use opsis_types::recording::{AnnotationKind, AnnotationLook, AnnotationSpec, HighlightLook};

use crate::elements::anim::{Fade, draw_on, fade_window, is_faded_out};
use crate::elements::camera::click_log::rect_center;
use crate::elements::camera::window::lifted;
use crate::elements::draw::{DrawFailure, css_color};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::parts::hand_drawn::{MarkBox, MarkShape, hand_drawn_mark};
use crate::recording::POINTER_KINDS;

const FADE_S: f64 = 0.18;
const POP_SCALE: f64 = 1.2;
const SPOTLIGHT_RADIUS: f64 = 12.0;
const SPOTLIGHT_SPREAD: f64 = 9999.0;
const HIGHLIGHT_RADIUS: f64 = 10.0;
const HIGHLIGHT_BORDER: f64 = 3.0;
const HIGHLIGHT_FILL: f64 = 0.12;
const HIGHLIGHT_GROW_FROM: f64 = 0.96;
const TOOLTIP_RISE: f64 = 8.0;
const TOOLTIP_PAD_SHARE: f64 = 8.0 / 14.0;
const TOOLTIP_SHADOW_Y: f64 = 6.0;
const TOOLTIP_SHADOW_BLUR: f64 = 18.0;
const TOOLTIP_SHADOW: &str = "rgba(0,0,0,0.25)";
const CALLOUT_WIDTH: f64 = 140.0;
const CALLOUT_HEIGHT: f64 = 110.0;
const CALLOUT_LINE: &str = "M130 10 C 90 20, 40 40, 12 96";
const CALLOUT_HEAD: &str = "M4 82 L12 98 L28 88";
const CALLOUT_HEAD_FROM: f64 = 0.9;
const CALLOUT_TEXT_LEFT: f64 = 120.0;
const CALLOUT_TEXT_BOTTOM: f64 = 96.0;
const DEFAULT_ROUGHNESS: f64 = 1.0;
const STROKES: u32 = 2;
const MARKER_OPACITY: f64 = 0.5;

#[derive(Debug, Clone, PartialEq)]
pub struct Annotation {
    pub id: String,
    pub kind: AnnotationKind,
    pub rect: Rect,
    pub from_s: f64,
    pub to_s: f64,
    pub spec: AnnotationSpec,
    pub look: HighlightLook,
    pub roughness: f64,
    pub seed: u32,
}

pub fn resolve_annotations(log: &ActionLog, specs: &[AnnotationSpec]) -> Vec<Annotation> {
    let mut out: Vec<Annotation> = Vec::new();
    for (index, spec) in specs.iter().enumerate() {
        let needle = spec.match_text.trim().to_lowercase();
        for a in &log.actions {
            let Some(rect) = a.rect.filter(|_| POINTER_KINDS.contains(&a.kind)) else {
                continue;
            };
            if !needle.is_empty()
                && !a
                    .label
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(&needle)
            {
                continue;
            }
            let start = a.t_depart.unwrap_or(a.t) - spec.lead_ms;
            let end = a.t_end.unwrap_or(a.t) + spec.hold_ms;
            let seed = u32::try_from(out.len() + 1).unwrap_or(u32::MAX);
            out.push(Annotation {
                id: format!("{index}:{}", a.id),
                kind: spec.kind,
                rect,
                from_s: start.max(0.0) / 1000.0,
                to_s: end / 1000.0,
                spec: spec.clone(),
                look: spec.look.unwrap_or(HighlightLook::Clean),
                roughness: spec.roughness.unwrap_or(DEFAULT_ROUGHNESS),
                seed,
            });
        }
    }
    out
}

struct Mark<'a> {
    annotation: &'a Annotation,
    opacity: f64,
    k: f64,
    counter: f64,
    viewport: Viewport,
    area: (f64, f64),
    t: f64,
}

impl Mark<'_> {
    fn width(&self) -> f64 {
        f64::from(self.viewport.width)
    }

    fn height(&self) -> f64 {
        f64::from(self.viewport.height)
    }

    const fn look(&self) -> Option<&AnnotationLook> {
        self.annotation.spec.mark_look.as_ref()
    }

    #[expect(
        clippy::suboptimal_flops,
        reason = "each step rounds on its own, and the recorded goldens hold what that draws"
    )]
    fn boxed(&self, style: Style) -> Style {
        let a = self.annotation;
        let pad = a.spec.padding;
        style
            .position(Position::Absolute)
            .left(percent((a.rect.x - pad) / self.width()))
            .top(percent((a.rect.y - pad) / self.height()))
            .width(percent((a.rect.w + pad * 2.0) / self.width()))
            .height(percent((a.rect.h + pad * 2.0) / self.height()))
    }
}

fn faded(color: &str, share: f64) -> String {
    let text = match percent(share) {
        Length::Percent(Num(p)) => opsis_scene::random::js_number_text(p),
        _ => "0".to_owned(),
    };
    format!("color-mix(in srgb, {color} {text}%, transparent)")
}

fn spotlight(mark: &Mark<'_>) -> Node {
    let look = mark.look();
    div(mark
        .boxed(Style::default())
        .border_radius_all(look.map_or(SPOTLIGHT_RADIUS, |l| l.radius) * mark.k)
        .box_shadow(vec![Shadow {
            inset: false,
            x: Length::ZERO,
            y: Length::ZERO,
            blur: Length::px(look.and_then(|l| l.softness).unwrap_or(0.0) * mark.k),
            spread: Length::px(SPOTLIGHT_SPREAD),
            color: css_color(&mark.annotation.spec.color),
        }])
        .opacity(mark.opacity))
    .into()
}

#[expect(
    clippy::suboptimal_flops,
    reason = "each step rounds on its own, and the recorded goldens hold what that draws"
)]
fn sketched(mark: &Mark<'_>) -> Result<Node, DrawFailure> {
    let a = mark.annotation;
    let look = mark.look();
    let marker = a.look == HighlightLook::Marker;
    let pad = a.spec.padding;
    let w = js_round((a.rect.w + pad * 2.0) / mark.width() * mark.area.0);
    let h = js_round((a.rect.h + pad * 2.0) / mark.height() * mark.area.1);
    let drawn = hand_drawn_mark(
        MarkShape {
            look: a.look,
            seed: a.seed.max(1),
            strokes: look
                .and_then(|l| l.strokes)
                .map_or(STROKES, opsis_types::numbers::whole),
            roughness: Some(a.roughness),
        },
        MarkBox {
            x: 0.0,
            y: 0.0,
            w,
            h,
        },
        draw_on(mark.t, a.from_s, look.and_then(|l| l.draw_s))?,
        &a.spec.color,
        look.and_then(|l| l.line_width).unwrap_or(HIGHLIGHT_BORDER) * mark.k * mark.counter,
    );
    let tracker: Node = span(
        Style::default()
            .display(Display::InlineBlock)
            .position(Position::Relative)
            .white_space(WhiteSpace::Pre),
    )
    .child(div(Style::default()
        .width(Length::px(
            (a.rect.w + pad * 2.0) / mark.width() * mark.area.0,
        ))
        .height(Length::px(
            (a.rect.h + pad * 2.0) / mark.height() * mark.area.1,
        ))))
    .into();
    let children = if marker {
        vec![drawn, tracker]
    } else {
        vec![tracker, drawn]
    };
    let mut style = Style::default()
        .position(Position::Absolute)
        .left(percent((a.rect.x - pad) / mark.width()))
        .top(percent((a.rect.y - pad) / mark.height()))
        .opacity(if marker {
            mark.opacity
                * look
                    .and_then(|l| l.marker_opacity)
                    .unwrap_or(MARKER_OPACITY)
        } else {
            mark.opacity
        });
    if marker {
        style = style.mix_blend_mode(BlendMode::Multiply);
    }
    Ok(div(style)
        .child(
            span(
                Style::default()
                    .display(Display::InlineBlock)
                    .position(Position::Relative),
            )
            .children(children),
        )
        .into())
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film grows the highlight with a separate multiply and add"
)]
fn highlight(mark: &Mark<'_>) -> Result<Node, DrawFailure> {
    let a = mark.annotation;
    if a.look != HighlightLook::Clean {
        return sketched(mark);
    }
    let look = mark.look();
    let grow = HIGHLIGHT_GROW_FROM + (1.0 - HIGHLIGHT_GROW_FROM) * mark.opacity;
    Ok(div(mark
        .boxed(Style::default())
        .border_radius_all(look.map_or(HIGHLIGHT_RADIUS, |l| l.radius) * mark.k)
        .border(Edges::all(BorderSide {
            width: Length::px(
                look.and_then(|l| l.line_width).unwrap_or(HIGHLIGHT_BORDER) * mark.k * mark.counter,
            ),
            style: BorderStyle::Solid,
            color: Some(css_color(&a.spec.color)),
        }))
        .background_color(css_color(&faded(
            &a.spec.color,
            look.and_then(|l| l.fill_opacity).unwrap_or(HIGHLIGHT_FILL),
        )))
        .opacity(mark.opacity)
        .transform(vec![Transform::Scale(Num(grow), Num(grow), Num(1.0))]))
    .into())
}

fn label_font(a: &Annotation) -> Vec<String> {
    a.spec.font.as_deref().map_or_else(
        || font_or("", FontRole::Main),
        |f| font_or(f, FontRole::Main),
    )
}

fn tooltip(mark: &Mark<'_>) -> Option<Node> {
    let a = mark.annotation;
    let (look, size) = (a.spec.label_look.as_ref()?, a.spec.font_size?);
    let k = mark.k;
    let mut bubble = Style::default()
        .background_color(css_color(&a.spec.color))
        .font_family(label_font(a))
        .font_weight(look.font_weight)
        .font_size(size * k)
        .padding(Edges::axes(
            Length::px(look.box_padding * TOOLTIP_PAD_SHARE * k),
            Length::px(look.box_padding * k),
        ))
        .border_radius_all(look.radius * k)
        .white_space(WhiteSpace::Nowrap);
    if let Some(color) = &a.spec.text_color {
        bubble = bubble.color(css_color(color));
    }
    if look.shadow {
        bubble = bubble.box_shadow(vec![Shadow {
            inset: false,
            x: Length::ZERO,
            y: Length::px(TOOLTIP_SHADOW_Y * k),
            blur: Length::px(TOOLTIP_SHADOW_BLUR * k),
            spread: Length::ZERO,
            color: css_color(TOOLTIP_SHADOW),
        }]);
    }
    let border = |color: &str| BorderSide {
        width: Length::px(look.pointer_size * k),
        style: BorderStyle::Solid,
        color: Some(css_color(color)),
    };
    let pointer = div(Style::default()
        .width(Length::ZERO)
        .height(Length::ZERO)
        .margin(Edges::axes(Length::ZERO, Length::Auto))
        .border(Edges {
            top: border(&a.spec.color),
            right: border("transparent"),
            bottom: BorderSide {
                width: Length::ZERO,
                style: BorderStyle::None,
                color: None,
            },
            left: border("transparent"),
        }));
    let pop = mark.counter * POP_SCALE;
    Some(
        div(Style::default()
            .position(Position::Absolute)
            .left(percent(rect_center(a.rect).x / mark.width()))
            .top(percent((a.rect.y - a.spec.padding) / mark.height()))
            .transform(vec![
                Transform::Translate(
                    Length::percent(-50.0),
                    Length::percent(-100.0),
                    Length::ZERO,
                ),
                Transform::Translate(
                    Length::ZERO,
                    Length::px((1.0 - mark.opacity) * TOOLTIP_RISE * k),
                    Length::ZERO,
                ),
                Transform::Scale(Num(pop), Num(pop), Num(1.0)),
            ])
            .transform_origin((Length::percent(50.0), Length::percent(100.0), Length::ZERO))
            .opacity(mark.opacity))
        .child(div(bubble).child(a.spec.text.clone()))
        .child(pointer)
        .into(),
    )
}

fn callout(mark: &Mark<'_>) -> Option<Node> {
    let a = mark.annotation;
    let (look, size_px) = (a.spec.label_look.as_ref()?, a.spec.font_size?);
    let k = mark.k;
    let size = look.arrow_size * k;
    let line = SvgElement::new("path")
        .attr("d", CALLOUT_LINE)
        .attr("fill", "none")
        .attr("stroke", a.spec.color.as_str())
        .attr("stroke-width", look.line_width)
        .attr("stroke-linecap", "round")
        .attr("pathLength", 1.0)
        .attr("stroke-dasharray", 1.0)
        .attr("stroke-dashoffset", 1.0 - mark.opacity);
    let head = SvgElement::new("path")
        .attr("d", CALLOUT_HEAD)
        .attr("fill", "none")
        .attr("stroke", a.spec.color.as_str())
        .attr("stroke-width", look.line_width)
        .attr("stroke-linecap", "round")
        .attr("stroke-linejoin", "round")
        .attr(
            "opacity",
            if mark.opacity > CALLOUT_HEAD_FROM {
                1.0
            } else {
                0.0
            },
        );
    let pop = mark.counter * POP_SCALE;
    Some(
        div(Style::default()
            .position(Position::Absolute)
            .left(percent((a.rect.x + a.rect.w) / mark.width()))
            .top(percent(a.rect.y / mark.height()))
            .transform(vec![Transform::Scale(Num(pop), Num(pop), Num(1.0))])
            .transform_origin((Length::ZERO, Length::percent(100.0), Length::ZERO))
            .opacity(mark.opacity))
        .child(svg(
            SvgElement::new("svg")
                .attr("width", CALLOUT_WIDTH * size)
                .attr("height", CALLOUT_HEIGHT * size)
                .attr("viewBox", format!("0 0 {CALLOUT_WIDTH} {CALLOUT_HEIGHT}"))
                .child(line)
                .child(head),
            Style::default()
                .position(Position::Absolute)
                .left(Length::ZERO)
                .bottom(Length::ZERO)
                .width(Length::px(CALLOUT_WIDTH * size))
                .height(Length::px(CALLOUT_HEIGHT * size))
                .overflow(Overflow::Visible),
        ))
        .child(
            div(Style::default()
                .position(Position::Absolute)
                .left(Length::px(CALLOUT_TEXT_LEFT * size))
                .bottom(Length::px(CALLOUT_TEXT_BOTTOM * size))
                .color(css_color(&a.spec.color))
                .font_family(label_font(a))
                .font_weight(look.font_weight)
                .font_size(size_px * k)
                .white_space(WhiteSpace::Nowrap)
                .transform(vec![Transform::Rotate(Angle::deg(look.tilt))]))
            .child(a.spec.text.clone()),
        )
        .into(),
    )
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shown {
    pub viewport: Viewport,
    pub area: (f64, f64),
    pub t: f64,
    pub camera_scale: f64,
    pub k: f64,
    pub lift: bool,
}

pub fn annotations(list: &[Annotation], shown: &Shown) -> Result<Option<Node>, DrawFailure> {
    let (viewport, t, camera_scale, k) = (shown.viewport, shown.t, shown.camera_scale, shown.k);
    if list.is_empty() {
        return Ok(None);
    }
    let counter = 1.0 / camera_scale.max(1.0);
    let mut marks = Vec::new();
    for annotation in list {
        let fade_s = annotation
            .spec
            .mark_look
            .as_ref()
            .map_or(FADE_S, |l| l.fade_s);
        let opacity = fade_window(
            t,
            annotation.from_s,
            annotation.to_s,
            &Fade {
                fade_in: fade_s,
                fade_out: fade_s,
                ease_in: Some(Easing::ease_out(Easing::Cubic)),
                ease_out: Some(Easing::ease_in(Easing::Cubic)),
            },
        )?;
        if is_faded_out(opacity) {
            continue;
        }
        let mark = Mark {
            annotation,
            opacity,
            k,
            counter,
            viewport,
            area: shown.area,
            t,
        };
        let drawn = match annotation.kind {
            AnnotationKind::Spotlight => Some(spotlight(&mark)),
            AnnotationKind::Highlight => Some(highlight(&mark)?),
            AnnotationKind::Tooltip => tooltip(&mark),
            AnnotationKind::Callout => callout(&mark),
        };
        marks.extend(drawn.map(|mark| if shown.lift { lifted(mark) } else { mark }));
    }
    if marks.is_empty() {
        return Ok(None);
    }
    Ok(Some(
        div(Style::default()
            .position(Position::Absolute)
            .inset(Length::ZERO)
            .container_type(ContainerType::Size))
        .children(marks)
        .into(),
    ))
}
