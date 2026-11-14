use std::collections::HashMap;
use std::sync::LazyLock;

use opsis_scene::keyed::{lerp, percent, sample_keyed};
use opsis_scene::node::{Node, SvgElement, div, img, svg};
use opsis_scene::paint::{Filter, Shadow, Transform};
use opsis_scene::style::{BorderSide, BorderStyle, ObjectFit, Position, Style};
use opsis_scene::values::{Edges, Length, Num};
use opsis_types::actions::{ClickButton, CursorSample};
use opsis_types::recording::{CursorFeel, CursorLook, PointerProps, RippleLook};
use serde::Deserialize;

use crate::elements::anim::is_faded_out;
use crate::elements::camera::click_log::{ClickEvent, ClickLog};
use crate::elements::camera::track::clamp01;
use crate::elements::camera::window::ink;
use crate::elements::draw::css_color;

const CLICK_SQUASH_S: f64 = 0.12;
const CURSOR_FADE_S: f64 = 0.18;
const ARROW_SHAPE: &str = "default";
const HOT_X: f64 = 1.0;
const HOT_Y: f64 = 1.0;
const ARROW_W: f64 = 34.0;
const ARROW_H: f64 = 45.0;
const ARROW_VIEW_W: f64 = 24.0;
const ARROW_VIEW_H: f64 = 32.0;
const RIPPLE_D: f64 = 22.0;
const RIPPLE_RING: f64 = 0.47;
const RIPPLE_FILL: f64 = 0.085;
const RIPPLE_S: f64 = 0.35;
const RIPPLE_FROM: f64 = 0.3;
const RIPPLE_GROWTH: f64 = 1.05;
const ARROW_PATH: &str = "M1 1 L1 23 L6.5 17.8 L10.2 26.6 L13.7 25 L9.95 16.5 L17 16.3 Z";
const MAC_CURSOR_PX: f64 = 44.0;

pub const DEFAULT_CURSOR_FEEL: CursorFeel = CursorFeel {
    grow_with_zoom: 0.58,
    outline_width: 1.85,
    shadow: 0.34,
    press_squash: 0.9,
    hide_while_typing: true,
};

#[derive(Debug, Deserialize)]
struct MacPicture {
    src: String,
    hotspot: (f64, f64),
}

#[derive(Debug, Deserialize)]
struct MacCursors {
    #[serde(rename = "box")]
    size: f64,
    cursors: HashMap<String, MacPicture>,
    keywords: HashMap<String, String>,
}

static MAC_CURSORS: LazyLock<MacCursors> = LazyLock::new(|| {
    serde_json::from_str(include_str!("mac-cursors.generated.json"))
        .expect("the bundled Mac cursors")
});

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ring {
    pub progress: f64,
    pub hollow: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CursorState {
    pub x: f64,
    pub y: f64,
    pub squash: f64,
    pub rings: Vec<Ring>,
    pub opacity: f64,
    pub shape: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CursorMotion {
    pub press_squash: f64,
    pub hide_while_typing: bool,
    pub ripple_s: f64,
}

fn cursor_opacity(clicks: &[ClickEvent], t: f64) -> f64 {
    let mut opacity: f64 = 1.0;
    for c in clicks {
        let Some(end) = c.type_end_ms else {
            continue;
        };
        let from = c.t_ms / 1000.0;
        let to = end / 1000.0;
        if t <= from - CURSOR_FADE_S || t >= to + CURSOR_FADE_S {
            continue;
        }
        let fading_out = clamp01((from - t) / CURSOR_FADE_S);
        let fading_in = clamp01((t - to) / CURSOR_FADE_S);
        opacity = opacity.min(fading_out.max(fading_in));
    }
    opacity
}

fn shape_at(track: &[CursorSample], t: f64) -> String {
    let end = track.partition_point(|s| s.t <= t * 1000.0);
    track[..end]
        .iter()
        .rev()
        .find_map(|s| s.c.clone())
        .unwrap_or_else(|| ARROW_SHAPE.to_owned())
}

fn squash_after(released: f64, pressed: f64) -> f64 {
    if released <= 0.0 {
        return pressed;
    }
    if released > CLICK_SQUASH_S {
        return 1.0;
    }
    lerp(pressed, 1.0, released / CLICK_SQUASH_S)
}

pub fn cursor_at(
    track: &[CursorSample],
    clicks: &[ClickEvent],
    t: f64,
    motion: CursorMotion,
) -> Option<CursorState> {
    let (x, y) = sample_keyed(
        track,
        t * 1000.0,
        |s| s.t,
        |s| (s.x, s.y),
        |a, b, p| (lerp(a.x, b.x, p), lerp(a.y, b.y, p)),
    )?;
    let mut squash: f64 = 1.0;
    let mut rings = Vec::new();
    for c in clicks {
        for press in [c.t_down_ms, c.t_second_down_ms].into_iter().flatten() {
            let since = t - press / 1000.0;
            if since < 0.0 {
                continue;
            }
            let held = c.t_up_ms.map_or(0.0, |up| ((up - press) / 1000.0).max(0.0));
            squash = squash.min(squash_after(since - held, motion.press_squash));
            if since <= motion.ripple_s && motion.ripple_s > 0.0 {
                rings.push(Ring {
                    progress: clamp01(since / motion.ripple_s),
                    hollow: c.button == Some(ClickButton::Right),
                });
            }
        }
    }
    Some(CursorState {
        x,
        y,
        squash,
        rings,
        opacity: if motion.hide_while_typing {
            cursor_opacity(clicks, t)
        } else {
            1.0
        },
        shape: shape_at(track, t),
    })
}

fn faded(color: &str, share: f64) -> String {
    format!(
        "color-mix(in srgb, {color} {}, transparent)",
        percent_text(share)
    )
}

fn percent_text(share: f64) -> String {
    match percent(share) {
        Length::Percent(Num(p)) => format!("{}%", opsis_scene::random::js_number_text(p)),
        _ => "0%".to_owned(),
    }
}

#[expect(
    clippy::suboptimal_flops,
    reason = "each step rounds on its own, and the recorded goldens hold what that draws"
)]
fn ripple(ring: Ring, k: f64, look: &RippleLook) -> Node {
    let size = RIPPLE_D * look.size * k;
    let fill = if ring.hollow {
        css_color("transparent")
    } else {
        css_color(&faded(
            &look.color,
            look.fill_opacity.unwrap_or(RIPPLE_FILL),
        ))
    };
    let grow = RIPPLE_FROM + ring.progress * RIPPLE_GROWTH;
    div(Style::default()
        .position(Position::Absolute)
        .left(Length::ZERO)
        .top(Length::ZERO)
        .width(Length::px(size))
        .height(Length::px(size))
        .margin_left(Length::px(-size / 2.0))
        .margin_top(Length::px(-size / 2.0))
        .border_radius_all(Length::percent(50.0))
        .border(Edges::all(BorderSide {
            width: Length::px(look.width * k),
            style: BorderStyle::Solid,
            color: Some(css_color(&faded(
                &look.color,
                look.ring_opacity.unwrap_or(RIPPLE_RING),
            ))),
        }))
        .background_color(fill)
        .transform(vec![Transform::Scale(Num(grow), Num(grow), Num(1.0))])
        .opacity(1.0 - ring.progress))
    .into()
}

fn arrow(squash: f64, k: f64, fill: &str, edge: &str, feel: &CursorFeel) -> Node {
    let (width, height) = (ARROW_W * k, ARROW_H * k);
    let mut style = Style::default()
        .position(Position::Absolute)
        .left(Length::px(-HOT_X / ARROW_VIEW_W * width))
        .top(Length::px(-HOT_Y / ARROW_VIEW_H * height))
        .width(Length::px(width))
        .height(Length::px(height))
        .transform(vec![Transform::Scale(Num(squash), Num(squash), Num(1.0))])
        .transform_origin((
            percent(HOT_X / ARROW_VIEW_W),
            percent(HOT_Y / ARROW_VIEW_H),
            Length::ZERO,
        ));
    if feel.shadow > 0.0 {
        style = style.filter(vec![Filter::DropShadow(Shadow {
            inset: false,
            x: Length::ZERO,
            y: Length::px(1.5 * k),
            blur: Length::px(2.5 * k),
            spread: Length::ZERO,
            color: css_color(&ink(feel.shadow)),
        })]);
    }
    svg(
        SvgElement::new("svg")
            .attr("width", width)
            .attr("height", height)
            .attr("viewBox", format!("0 0 {ARROW_VIEW_W} {ARROW_VIEW_H}"))
            .child(
                SvgElement::new("path")
                    .attr("d", ARROW_PATH)
                    .attr("fill", fill)
                    .attr("stroke", edge)
                    .attr("stroke-width", feel.outline_width)
                    .attr("stroke-linejoin", "round")
                    .attr("paint-order", "stroke"),
            ),
        style,
    )
    .into()
}

fn mac_cursor(shape: &str, squash: f64, k: f64) -> Option<Node> {
    if shape == "none" {
        return None;
    }
    let cursors = &*MAC_CURSORS;
    let name = cursors
        .keywords
        .get(shape)
        .or_else(|| cursors.keywords.get("default"))?;
    let picture = cursors.cursors.get(name)?;
    let size = MAC_CURSOR_PX * k;
    let (x, y) = picture.hotspot;
    let scale = size / cursors.size;
    Some(
        img(
            picture.src.clone(),
            Style::default()
                .position(Position::Absolute)
                .left(Length::px(-x * scale))
                .top(Length::px(-y * scale))
                .width(Length::px(size))
                .height(Length::px(size))
                .object_fit(ObjectFit::Fill)
                .transform(vec![Transform::Scale(Num(squash), Num(squash), Num(1.0))])
                .transform_origin((Length::px(x * scale), Length::px(y * scale), Length::ZERO)),
        )
        .into(),
    )
}

pub fn cursor(
    log: &ClickLog,
    pointer: &PointerProps,
    t: f64,
    camera_scale: f64,
    k: f64,
) -> Option<Node> {
    let feel = pointer.feel.clone().unwrap_or(DEFAULT_CURSOR_FEEL);
    let motion = CursorMotion {
        press_squash: feel.press_squash,
        hide_while_typing: feel.hide_while_typing,
        ripple_s: pointer
            .ripple
            .as_ref()
            .and_then(|r| r.duration_s)
            .unwrap_or(RIPPLE_S),
    };
    if !pointer.shown {
        return None;
    }
    let state = cursor_at(
        log.cursor_track.as_deref().unwrap_or_default(),
        &log.clicks,
        t,
        motion,
    )?;
    if is_faded_out(state.opacity) {
        return None;
    }
    let (width, height) = (
        f64::from(log.viewport.width),
        f64::from(log.viewport.height),
    );
    let counter = camera_scale.powf(feel.grow_with_zoom - 1.0) * pointer.scale;
    let mut out = div(Style::default()
        .position(Position::Absolute)
        .left(percent(state.x / width))
        .top(percent(state.y / height))
        .width(Length::ZERO)
        .height(Length::ZERO)
        .opacity(state.opacity)
        .transform(vec![Transform::Scale(Num(counter), Num(counter), Num(1.0))])
        .transform_origin((Length::ZERO, Length::ZERO, Length::ZERO)));
    if let Some(look) = &pointer.ripple {
        out = out.children(state.rings.iter().map(|ring| ripple(*ring, k, look)));
    }
    let shown = match &pointer.look {
        CursorLook::Mac => mac_cursor(&state.shape, state.squash, k),
        CursorLook::Arrow { fill, edge } => Some(arrow(state.squash, k, fill, edge, &feel)),
    };
    if let Some(shown) = shown {
        out = out.child(shown);
    }
    Some(out.into())
}
