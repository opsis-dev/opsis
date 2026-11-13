use opsis_scene::clock::js_round;
use opsis_scene::easing::Easing;
use opsis_scene::interpolate::{Interpolation, interpolate};
use opsis_scene::node::{Node, div, span};
use opsis_scene::paint::{Shadow, Transform};
use opsis_scene::style::{Align, Display, LineHeight, Overflow, Style, TextOverflow, WhiteSpace};
use opsis_scene::timeline::{Stage, absolute_fill};
use opsis_scene::values::{Edges, Length, Num};
use opsis_types::actions::ActionLog;
use opsis_types::recording::{KeyNames, KeystrokeLook, KeystrokePosition};

use crate::elements::anim::{Fade, fade_window};
use crate::elements::camera::click_log::KeyEvent;
use crate::elements::camera::track::clamp01;
use crate::elements::draw::{DrawFailure, css_color};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::overlay_frame::{Insets, overlay_frame, overlay_padding};
use crate::elements::units::{utf16_len, utf16_slice};
use crate::nodes::browser::chord_keys;

const HOLD_S: f64 = 1.1;
pub const KEYSTROKE_EDGE: f64 = 56.0;
const IN_S: f64 = 0.13;
const OUT_S: f64 = 0.17;
const BEZIER: Easing = Easing::Bezier(0.15, 0.9, 0.75, 0.95);
const KEYCAP_HEIGHT: f64 = 103.0;
const KEYCAP_MIN_WIDTH: f64 = 216.0;
const KEYCAP_PAD_X: f64 = 46.0;
const KEYCAP_EDGE_LIFT: f64 = 3.0;
const KEYCAP_RADIUS: f64 = 26.0;
const KEYCAP_GLYPH: f64 = 44.0;
const KEYCAP_GLYPH_GAP: f64 = 0.28;
const KEYCAP_RISE: f64 = 10.0;
pub const TYPED_LINGER_S: f64 = 0.6;
const TYPED_FALLBACK_MS: f64 = 1000.0;
const TYPED_FADE_IN_S: f64 = 0.15;
const TYPED_FADE_OUT_S: f64 = 0.2;
const CARET_BLINKS_PER_S: f64 = 2.5;
const CHIP_PAD_Y: f64 = 14.0;
const CHIP_PAD_X: f64 = 26.0;
const CHIP_FONT_SIZE: f64 = 30.0;
const CHIP_SHADOW_Y: f64 = 8.0;
const CHIP_SHADOW_BLUR: f64 = 24.0;
const CHIP_SHADOW: f64 = 0.35;

const GLYPHS: [(&str, &str); 20] = [
    ("alt", "⌥"),
    ("option", "⌥"),
    ("meta", "⌘"),
    ("cmd", "⌘"),
    ("command", "⌘"),
    ("control", "⌃"),
    ("ctrl", "⌃"),
    ("shift", "⇧"),
    ("enter", "⏎"),
    ("return", "⏎"),
    ("tab", "⇥"),
    ("escape", "esc"),
    ("esc", "esc"),
    ("backspace", "⌫"),
    ("delete", "⌦"),
    ("arrowup", "↑"),
    ("arrowdown", "↓"),
    ("arrowleft", "←"),
    ("arrowright", "→"),
    ("space", "space"),
];

const WORDS: [(&str, &str); 20] = [
    ("alt", "Alt"),
    ("option", "Option"),
    ("meta", "Cmd"),
    ("cmd", "Cmd"),
    ("command", "Cmd"),
    ("control", "Ctrl"),
    ("ctrl", "Ctrl"),
    ("shift", "Shift"),
    ("enter", "Enter"),
    ("return", "Return"),
    ("tab", "Tab"),
    ("escape", "Esc"),
    ("esc", "Esc"),
    ("backspace", "Backspace"),
    ("delete", "Delete"),
    ("arrowup", "↑"),
    ("arrowdown", "↓"),
    ("arrowleft", "←"),
    ("arrowright", "→"),
    ("space", "Space"),
];

const ALIASES: [(&str, &str); 11] = [
    ("esc", "escape"),
    ("return", "enter"),
    ("cmd", "meta"),
    ("command", "meta"),
    ("ctrl", "control"),
    ("option", "alt"),
    ("del", "delete"),
    ("up", "arrowup"),
    ("down", "arrowdown"),
    ("left", "arrowleft"),
    ("right", "arrowright"),
];

fn looked_up(table: &[(&str, &str)], key: &str) -> Option<String> {
    table
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| (*v).to_owned())
}

pub fn chord_glyphs(chord: &str, names: KeyNames) -> String {
    let table: &[(&str, &str)] = if names == KeyNames::Words {
        &WORDS
    } else {
        &GLYPHS
    };
    let keys: Vec<String> = chord_keys(chord)
        .iter()
        .map(|key| looked_up(table, &key.to_lowercase()).unwrap_or_else(|| key.to_uppercase()))
        .collect();
    keys.join(if names == KeyNames::Words { " + " } else { "" })
}

fn chord_id(chord: &str) -> String {
    let mut keys: Vec<String> = chord_keys(chord)
        .iter()
        .map(|key| {
            let lower = key.to_lowercase();
            looked_up(&ALIASES, &lower).unwrap_or(lower)
        })
        .collect();
    keys.sort();
    keys.join("+")
}

pub fn shown_keys(keys: &[KeyEvent], left_out: &[String]) -> Vec<KeyEvent> {
    if left_out.is_empty() {
        return keys.to_vec();
    }
    let hidden: Vec<String> = left_out.iter().map(|c| chord_id(c)).collect();
    keys.iter()
        .filter(|key| !hidden.contains(&chord_id(&key.chord)))
        .cloned()
        .collect()
}

struct Active<'a> {
    chord: &'a str,
    in_p: f64,
    out_p: f64,
}

fn keycap_at(keys: &[KeyEvent], t: f64, hold: f64) -> Option<Active<'_>> {
    keys.iter().rev().find_map(|key| {
        let start = key.t_ms / 1000.0;
        let end = start + hold;
        (t >= start && t < end).then(|| Active {
            chord: &key.chord,
            in_p: clamp01((t - start) / IN_S),
            out_p: clamp01((end - t) / OUT_S),
        })
    })
}

fn ease(u: f64) -> f64 {
    BEZIER.at(clamp01(u))
}

pub fn keycap_hud(
    keys: &[KeyEvent],
    t: f64,
    look: &KeystrokeLook,
    k_frame: f64,
    stage: Stage,
) -> Option<Node> {
    let k = k_frame * look.size;
    let insets = overlay_frame(stage).insets;
    let active = keycap_at(keys, t, look.chord_hold_s.unwrap_or(HOLD_S))?;
    let edge = (look.edge.unwrap_or(KEYSTROKE_EDGE) - KEYCAP_EDGE_LIFT) * k_frame;
    let appear = ease(active.in_p);
    let opacity = appear.min(ease(active.out_p));
    let top = look.position == KeystrokePosition::Top;
    let rise = (1.0 - appear) * KEYCAP_RISE * k * if top { -1.0 } else { 1.0 };
    let mut cap = Style::default()
        .display(Display::Flex)
        .align_items(Align::Center)
        .justify_content(Align::Center)
        .min_width(Length::px(KEYCAP_MIN_WIDTH * k))
        .height(Length::px(KEYCAP_HEIGHT * k))
        .padding(Edges::axes(Length::ZERO, Length::px(KEYCAP_PAD_X * k)))
        .border_radius_all(look.radius.unwrap_or(KEYCAP_RADIUS) * k)
        .background_color(css_color(&look.keycap))
        .color(css_color(&look.ink))
        .font_size(KEYCAP_GLYPH * k)
        .gap(KEYCAP_GLYPH_GAP * KEYCAP_GLYPH * k)
        .line_height(LineHeight::Number(Num(1.0)))
        .opacity(opacity)
        .transform(vec![Transform::Translate(
            Length::ZERO,
            Length::px(rise),
            Length::ZERO,
        )]);
    if let Some(font) = look.font.as_deref().filter(|f| !f.is_empty()) {
        cap = cap.font_family(font_or(font, FontRole::Main));
    }
    Some(
        div(absolute_fill()
            .align_items(Align::Center)
            .justify_content(if top {
                Align::FlexStart
            } else {
                Align::FlexEnd
            })
            .padding_top(Length::px(if top { edge + insets.top } else { 0.0 }))
            .padding_bottom(Length::px(if top { 0.0 } else { edge + insets.bottom })))
        .child(div(cap).child(chord_glyphs(
            active.chord,
            look.key_names.unwrap_or_default(),
        )))
        .into(),
    )
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypedBeat {
    pub text: String,
    pub from_s: f64,
    pub to_s: f64,
}

pub fn typed_beats(log: &ActionLog, linger: f64) -> Vec<TypedBeat> {
    log.actions
        .iter()
        .filter(|a| a.kind == opsis_types::actions::ActionKind::Type)
        .filter_map(|a| {
            let text = a.text.as_ref().filter(|t| !t.is_empty())?;
            Some(TypedBeat {
                text: text.clone(),
                from_s: a.t / 1000.0,
                to_s: a.t_end.unwrap_or(a.t + TYPED_FALLBACK_MS) / 1000.0 + linger,
            })
        })
        .collect()
}

fn vertical(insets: Insets) -> Insets {
    Insets {
        top: insets.top,
        bottom: insets.bottom,
        ..Insets::default()
    }
}

pub fn typed_chip(
    beats: &[TypedBeat],
    t: f64,
    look: &KeystrokeLook,
    stage: Stage,
) -> Result<Option<Node>, DrawFailure> {
    let frame = overlay_frame(stage);
    let k = frame.k * look.size;
    let Some(beat) = beats.iter().find(|b| t >= b.from_s && t <= b.to_s) else {
        return Ok(None);
    };
    let typing_end = beat.to_s - look.typed_hold_s.unwrap_or(TYPED_LINGER_S);
    let length = opsis_types::numbers::float(utf16_len(&beat.text));
    let shown = js_round(interpolate(
        t,
        &[beat.from_s, typing_end],
        &[0.0, length],
        &Interpolation::clamped(),
    )?);
    let opacity = fade_window(
        t,
        beat.from_s,
        beat.to_s,
        &Fade {
            fade_in: TYPED_FADE_IN_S,
            fade_out: TYPED_FADE_OUT_S,
            ease_in: None,
            ease_out: None,
        },
    )?;
    let justify = match look.position {
        KeystrokePosition::Top => Align::FlexStart,
        KeystrokePosition::Bottom => Align::FlexEnd,
    };
    let font = look.font.as_deref().filter(|f| !f.is_empty()).map_or_else(
        || font_or("", FontRole::Code),
        |f| font_or(f, FontRole::Code),
    );
    let count = usize::try_from(opsis_types::numbers::whole(shown.max(0.0))).unwrap_or(0);
    let mut chip = div(Style::default()
        .opacity(opacity)
        .max_width(Length::percent(70.0))
        .padding(Edges::axes(
            Length::px(CHIP_PAD_Y * k),
            Length::px(CHIP_PAD_X * k),
        ))
        .border_radius_all(999.0)
        .background_color(css_color(&format!(
            "color-mix(in srgb, {} 82%, transparent)",
            look.keycap
        )))
        .color(css_color(&look.ink))
        .font_family(font)
        .font_size(CHIP_FONT_SIZE * k)
        .white_space(WhiteSpace::Nowrap)
        .overflow(Overflow::Hidden)
        .text_overflow(TextOverflow::Ellipsis)
        .box_shadow(vec![Shadow {
            inset: false,
            x: Length::ZERO,
            y: Length::px(CHIP_SHADOW_Y * k),
            blur: Length::px(CHIP_SHADOW_BLUR * k),
            spread: Length::ZERO,
            color: css_color(&format!(
                "rgba(0,0,0,{})",
                opsis_scene::random::js_number_text(look.shadow.unwrap_or(CHIP_SHADOW))
            )),
        }]))
    .child(utf16_slice(&beat.text, 0, count));
    if look.caret != Some(false) {
        let blink = (t * CARET_BLINKS_PER_S).floor() % 2.0;
        chip = chip.child(
            span(Style::default().opacity(if blink == 0.0 { 1.0 } else { 0.0 }))
                .child("▍".to_owned()),
        );
    }
    Ok(Some(
        div(overlay_padding(
            absolute_fill()
                .justify_content(justify)
                .align_items(Align::Center),
            look.edge.unwrap_or(KEYSTROKE_EDGE) * frame.k,
            vertical(frame.insets),
        ))
        .child(chip)
        .into(),
    ))
}
