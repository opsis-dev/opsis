use opsis_scene::measure::Measure;
use opsis_scene::node::{Node, SvgElement, div, svg};
use opsis_scene::paint::{Filter, Transform};
use opsis_scene::style::{Overflow, Position, Style};
use opsis_scene::values::{Length, Num};
use opsis_types::recording::TerminalEffects;
use opsis_types::terminal::TerminalLook;
use opsis_types::terminal_frames::{FrameRecord, Frames};

use crate::elements::terminal::effects::{
    effect_defs, effect_layers, grain, has_lens, lensed, soften_px, trail_weights,
};
use crate::elements::terminal::view::{View, lighten_layer, screen_size, terminal_view};

const BLINKS_PER_S: f64 = 1.0;
const SCREEN_ID: &str = "terminal";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fit {
    Fill,
    Cover,
    Contain,
}

#[derive(Debug, Clone, Copy)]
pub struct TerminalScreen<'a> {
    pub frames_url: &'a str,
    pub look: &'a TerminalLook,
    pub effects: Option<&'a TerminalEffects>,
    pub time_s: f64,
    pub fit: Fit,
    pub area: (f64, f64),
}

fn frame_index(frames: &Frames, time_s: f64) -> Option<usize> {
    let last = frames.frames.len().checked_sub(1)?;
    let wanted = opsis_scene::clock::js_round(time_s * f64::from(frames.fps)).max(0.0);
    let index = usize::try_from(opsis_types::numbers::whole(wanted)).unwrap_or(usize::MAX);
    Some(index.min(last))
}

fn frame_at(frames: &Frames, time_s: f64) -> Option<(usize, &FrameRecord)> {
    let index = frame_index(frames, time_s)?;
    frames.frames.get(index).map(|f| (index, f))
}

fn cursor_shown(time_s: f64, blink: bool) -> bool {
    !blink || (time_s * BLINKS_PER_S * 2.0).floor().rem_euclid(2.0) == 0.0
}

#[expect(
    clippy::suboptimal_flops,
    reason = "each step rounds on its own, and the recorded goldens hold what that draws"
)]
fn fitted(fit: Fit, (width, height): (f64, f64), (area_w, area_h): (f64, f64)) -> Vec<Transform> {
    let (sx, sy) = match fit {
        Fit::Fill => (area_w / width, area_h / height),
        Fit::Cover => {
            let s = (area_w / width).max(area_h / height);
            (s, s)
        }
        Fit::Contain => {
            let s = (area_w / width).min(area_h / height);
            (s, s)
        }
    };
    vec![
        Transform::Translate(
            Length::px((area_w - width * sx) / 2.0),
            Length::px((area_h - height * sy) / 2.0),
            Length::ZERO,
        ),
        Transform::Scale(Num(sx), Num(sy), Num(1.0)),
    ]
}

pub fn terminal_screen(screen: &TerminalScreen<'_>, measure: &dyn Measure) -> Option<Node> {
    let frames = measure.terminal(screen.frames_url)?;
    let (index, frame) = frame_at(&frames, screen.time_s)?;
    let size = screen_size(screen.look, frame.cols, frame.rows);
    let cursor_on = cursor_shown(screen.time_s, screen.look.cursor_blink);
    let fps = f64::from(frames.fps);
    let view = |frame: &FrameRecord, cursor_on: bool, id: &str| {
        terminal_view(&View {
            frames: &frames,
            frame,
            look: screen.look,
            cursor_on,
            measure,
            id,
        })
    };
    let mut content: Vec<Node> = vec![view(frame, cursor_on, SCREEN_ID)];
    for (k, weight) in trail_weights(screen.effects).into_iter().enumerate() {
        let earlier = opsis_types::numbers::float(k) + 1.0;
        let Some((earlier_index, earlier_frame)) = frame_at(&frames, screen.time_s - earlier / fps)
        else {
            continue;
        };
        if earlier_index != index {
            content.push(lighten_layer(
                weight,
                view(earlier_frame, false, &format!("{SCREEN_ID}-trail-{k}")),
            ));
        }
    }
    let blur = soften_px(screen.effects, size);
    let mut softened = Style::default().position(Position::Relative);
    if blur > 0.0 {
        softened = softened.filter(vec![Filter::Blur(Length::px(blur))]);
    }
    let placed = Style::default()
        .position(Position::Absolute)
        .left(Length::ZERO)
        .top(Length::ZERO)
        .width(Length::px(size.width))
        .height(Length::px(size.height));
    let screen_content: Node = div(softened).children(content).into();
    let lensed_content = match screen.effects {
        Some(effects) if has_lens(Some(effects)) => lensed(effects, size, screen_content),
        _ => screen_content,
    };
    let mut layers: Vec<Node> = vec![div(placed.clone()).child(lensed_content).into()];
    if let Some(effects) = screen.effects {
        let frame_number = opsis_scene::clock::js_round(screen.time_s * fps);
        let mut root = SvgElement::new("svg")
            .attr("width", size.width)
            .attr("height", size.height)
            .child(effect_defs(SCREEN_ID, effects, size));
        for layer in effect_layers(SCREEN_ID, effects, size) {
            root = root.child(layer);
        }
        layers.push(
            svg(
                root,
                Style::default()
                    .position(Position::Absolute)
                    .left(Length::ZERO)
                    .top(Length::ZERO)
                    .width(Length::px(size.width))
                    .height(Length::px(size.height)),
            )
            .into(),
        );
        if effects.grain > 0.0 {
            layers.push(div(placed).child(grain(effects, frame_number)).into());
        }
    }
    Some(
        div(Style::default()
            .position(Position::Absolute)
            .inset(Length::ZERO)
            .overflow(Overflow::Hidden))
        .child(
            div(Style::default()
                .position(Position::Absolute)
                .left(Length::ZERO)
                .top(Length::ZERO)
                .width(Length::px(size.width))
                .height(Length::px(size.height))
                .transform_origin((Length::ZERO, Length::ZERO, Length::ZERO))
                .transform(fitted(screen.fit, (size.width, size.height), screen.area)))
            .children(layers),
        )
        .into(),
    )
}
