use std::collections::BTreeSet;

use serde_json::{Map, Value};

use crate::elements::introspect::settings_in_frames;
use crate::elements::{ElementKind, ElementMeta};

type Settings = Map<String, Value>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Unit {
    Frames,
    Authored,
    Seconds,
}

#[derive(Clone, Copy)]
struct Timing {
    unit: Unit,
    end: fn(&Settings) -> f64,
    keeps_length: bool,
}

pub const AUTHORED_FPS: f64 = 30.0;
const CUT_OFF_HOLD: f64 = AUTHORED_FPS / 2.0;
const RELEASE_TEASER_SECONDS: f64 = 960.0 * 1001.0 / 60000.0;
const LAUNCH_ANYTHING_SECONDS: f64 = 1600.0 / 60.0;

fn finite(value: Option<&Value>) -> Option<f64> {
    value.and_then(Value::as_f64).filter(|n| n.is_finite())
}

fn num(s: &Settings, key: &str, fallback: f64) -> f64 {
    finite(s.get(key)).unwrap_or(fallback)
}

fn text<'a>(s: &'a Settings, key: &str) -> &'a str {
    s.get(key).and_then(Value::as_str).unwrap_or("")
}

fn list<'a>(s: &'a Settings, key: &str) -> &'a [Value] {
    s.get(key)
        .and_then(Value::as_array)
        .map_or(&[], Vec::as_slice)
}

fn field<'a>(row: &'a Value, key: &str) -> Option<&'a Value> {
    row.as_object().and_then(|r| r.get(key))
}

fn field_num(row: &Value, key: &str, fallback: f64) -> f64 {
    finite(field(row, key)).unwrap_or(fallback)
}

fn field_str<'a>(row: &'a Value, key: &str) -> &'a str {
    field(row, key).and_then(Value::as_str).unwrap_or("")
}

fn last_index(count: usize) -> f64 {
    count.saturating_sub(1) as f64
}

fn characters(text: &str) -> usize {
    text.chars().count()
}

fn words(text: &str) -> usize {
    text.split_whitespace().count()
}

fn lines(text: &str) -> usize {
    text.split('\n').count()
}

fn spaced(text: &str) -> usize {
    text.split(' ').count()
}

fn comma_list(text: &str) -> Vec<&str> {
    text.split(',')
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .collect()
}

fn max_of(values: impl IntoIterator<Item = f64>) -> f64 {
    values.into_iter().fold(0.0, f64::max)
}

fn word_push_end(text: &str, word_gap: f64, accel: f64) -> f64 {
    let count = words(text);
    if count == 0 {
        return 0.0;
    }
    let index = (count - 1) as f64;
    let reveal = if accel == 1.0 {
        index * word_gap
    } else {
        word_gap * (1.0 - accel.powf(index)) / (1.0 - accel)
    };
    reveal.ceil() + 12.0
}

const KEYSTROKE_HOLD: f64 = 8.0;
const KEYSTROKE_EXIT: f64 = 8.0;

fn keystroke_end(s: &Settings) -> f64 {
    let linger = num(s, "linger", 30.0);
    max_of(list(s, "steps").iter().filter_map(|step| {
        let at = finite(field(step, "at"))?;
        (!field_str(step, "keys").trim().is_empty())
            .then(|| at + field_num(step, "hold", KEYSTROKE_HOLD) + linger + KEYSTROKE_EXIT)
    }))
}

const TERMINAL_START: f64 = 10.0;
const TERMINAL_DELAY: f64 = 8.0;
const TERMINAL_ELLIPSIS_PAUSE: f64 = 18.0;

fn terminal_end(s: &Settings) -> f64 {
    let per_frame = (num(s, "chunkSize", 1.0) * num(s, "charsPerFrame", 1.0)).max(0.01);
    list(s, "lines").iter().fold(TERMINAL_START, |end, line| {
        let text = field_str(line, "text");
        let pause = match field(line, "pause").and_then(Value::as_f64) {
            Some(p) => p,
            None if text.trim_end().ends_with("...") => TERMINAL_ELLIPSIS_PAUSE,
            None => 0.0,
        };
        let typing = (text.encode_utf16().count() as f64 / per_frame).ceil();
        end + field_num(line, "delay", TERMINAL_DELAY) + typing + pause
    })
}

const MORPH_BASE: f64 = 36.0;
const MORPH_ENTER_START: f64 = 20.0;
const MORPH_RISE: f64 = 8.0;
const MORPH_SHOW: f64 = 24.0;
const MORPH_FALL: f64 = 14.0;

fn code_morph_end(s: &Settings) -> f64 {
    let morph = num(s, "morphDuration", MORPH_BASE).clamp(12.0, 120.0);
    let mut ats: Vec<f64> = list(s, "steps")
        .iter()
        .map(|step| field_num(step, "at", 0.0).max(0.0))
        .collect();
    ats.sort_by(f64::total_cmp);
    let mut ready = 0.0;
    let mut last = None;
    for at in ats.iter().skip(1) {
        let start = at.max(ready);
        last = Some(start);
        ready = start + morph;
    }
    let Some(last) = last else { return 0.0 };
    let highlight = last + MORPH_ENTER_START * morph / MORPH_BASE + MORPH_RISE + MORPH_SHOW;
    if s.get("highlight") == Some(&Value::Bool(false)) {
        last + morph
    } else {
        (last + morph).max(highlight + MORPH_FALL)
    }
}

fn handwrite_frames(text: &str, per_step: f64, step: f64) -> f64 {
    (characters(text) as f64 / per_step).ceil() * step + step
}

fn check_list_end(s: &Settings) -> f64 {
    let step = num(s, "step", 3.0);
    let item_gap = num(s, "itemGap", step * 6.0);
    let per_step = num(s, "perStep", 1.6);
    let close_gap = num(s, "closeGap", step * 3.0);
    let items: Vec<(&str, bool)> = list(s, "items")
        .iter()
        .map(|item| match item.as_str() {
            Some(text) => (text, true),
            None => (
                field_str(item, "text"),
                field(item, "checked") != Some(&Value::Bool(false)),
            ),
        })
        .collect();
    let enter_end = max_of(items.iter().enumerate().map(|(i, (text, _))| {
        i as f64 * item_gap + step * 2.0 + handwrite_frames(text, per_step, step)
    }));
    let checked = items.iter().filter(|(_, c)| *c).count();
    let strike_end = if checked > 0 {
        enter_end + (checked - 1) as f64 * close_gap + step * 4.0
    } else {
        0.0
    };
    enter_end.max(strike_end)
}

const AGENT_PLAN_PER_WORD: f64 = 0.72;
const AGENT_ANSWER_PER_WORD: f64 = 1.0;
const AGENT_ANSWER_PER_LINE: f64 = 1.0;

fn agent_run_end(s: &Settings) -> f64 {
    let per_frame = num(s, "compression", 3.0).clamp(0.25, 20.0) / AUTHORED_FPS;
    let plan: Vec<&str> = list(s, "runPlan")
        .iter()
        .map(|v| v.as_str().unwrap_or(""))
        .collect();
    let plan = plan.join("\n");
    let answer = text(s, "runAnswer");
    let think = 16.0 + 14.0 + words(&plan) as f64 * AGENT_PLAN_PER_WORD + 8.0;
    let tools: f64 = list(s, "runSteps")
        .iter()
        .map(|step| {
            5.0 + (field_num(step, "duration", 0.0) / per_frame)
                .round()
                .clamp(12.0, 120.0)
        })
        .sum();
    let answering =
        words(answer) as f64 * AGENT_ANSWER_PER_WORD + lines(answer) as f64 * AGENT_ANSWER_PER_LINE;
    think + tools + 8.0 + answering + 16.0
}

fn logo_wall_end(s: &Settings) -> f64 {
    let count = list(s, "logos").len();
    let columns = num(s, "columns", 4.0).round().max(1.0) as usize;
    let start_at = num(s, "startAtSeconds", 0.32);
    let diagonal = (0..count)
        .map(|i| (i / columns + i % columns) as f64)
        .fold(0.0, f64::max);
    let arrived = start_at + last_index(count) * num(s, "arriveStaggerSeconds", 0.06) + 0.36;
    let warmed =
        start_at + 0.3 + diagonal * num(s, "staggerSeconds", 0.24) + num(s, "warmSeconds", 0.6);
    arrived.max(warmed)
}

fn end_card_end(s: &Settings) -> f64 {
    let url = text(s, "url");
    [
        0.81,
        if text(s, "subtitle").is_empty() {
            0.0
        } else {
            1.1
        },
        if text(s, "cta").is_empty() { 0.0 } else { 2.75 },
        if url.is_empty() {
            0.0
        } else {
            1.24 + url.encode_utf16().count() as f64 / 26.0
        },
        if list(s, "handles").is_empty() {
            0.0
        } else {
            2.1
        },
    ]
    .into_iter()
    .fold(f64::NEG_INFINITY, f64::max)
}

fn code_reveal_end(s: &Settings) -> f64 {
    let code = text(s, "code").trim_end();
    let code = code.trim_start_matches(['\r', '\n']);
    let rows: Vec<&str> = code
        .split('\n')
        .map(|r| r.strip_suffix('\r').unwrap_or(r))
        .collect();
    let typed: usize = rows
        .iter()
        .map(|r| r.trim_start().encode_utf16().count())
        .sum();
    0.4 + typed as f64 / 55.0 + last_index(rows.len()) * 0.07 + 0.84
}

fn file_tree_rows(paths: &[&str]) -> usize {
    let mut rows = BTreeSet::new();
    for path in paths {
        let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
        for depth in 1..=parts.len() {
            rows.insert(parts[..depth].join("/"));
        }
    }
    rows.len()
}

fn notification_end(s: &Settings) -> f64 {
    let start_at = num(s, "startAtSeconds", 0.3);
    let stagger = num(s, "staggerSeconds", 0.62);
    let life = num(s, "lifeSeconds", 2.3);
    list(s, "toasts")
        .iter()
        .enumerate()
        .fold(0.0, |end, (i, toast)| {
            let spawn = field_num(toast, "atSeconds", start_at + stagger * i as f64);
            end.max(spawn + field_num(toast, "holdFor", life) + 0.3)
        })
}

fn punch_lines_end(s: &Settings) -> f64 {
    let cards = text(s, "script")
        .split('|')
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .count();
    let holds: Vec<f64> = comma_list(text(s, "holds"))
        .iter()
        .filter_map(|h| h.parse::<f64>().ok())
        .filter(|h| h.is_finite())
        .collect();
    (0..cards)
        .map(|i| {
            if holds.is_empty() {
                48.0
            } else {
                holds[i.min(holds.len() - 1)]
            }
            .max(1.0)
        })
        .sum()
}

fn authored(end: fn(&Settings) -> f64) -> Timing {
    Timing {
        unit: Unit::Authored,
        end,
        keeps_length: false,
    }
}

fn frames(end: fn(&Settings) -> f64) -> Timing {
    Timing {
        unit: Unit::Frames,
        end,
        keeps_length: false,
    }
}

fn seconds(end: fn(&Settings) -> f64) -> Timing {
    Timing {
        unit: Unit::Seconds,
        end,
        keeps_length: false,
    }
}

const KEEPS_ITS_LENGTH: Timing = Timing {
    unit: Unit::Authored,
    end: |_| 0.0,
    keeps_length: true,
};

fn element_timing(key: &str) -> Option<Timing> {
    Some(match key {
        "remocn/soft-blur-in" => authored(|s| last_index(characters(text(s, "text"))) + 27.0),
        "remocn/typewriter" => authored(|s| {
            characters(text(s, "text")) as f64 / num(s, "charsPerSecond", 22.0).max(0.1)
                * AUTHORED_FPS
        }),
        "remocn/matrix-decode" => authored(|s| num(s, "revealDuration", 60.0)),
        "remocn/slot-machine-roll" => authored(|s| {
            last_index(
                text(s, "from")
                    .encode_utf16()
                    .count()
                    .max(text(s, "to").encode_utf16().count()),
            ) * 4.0
        }),
        "remocn/staggered-fade-up" => {
            authored(|s| last_index(spaced(text(s, "text"))) * num(s, "staggerDelay", 4.0) + 12.0)
        }
        "remocn/kinetic-center-build" => {
            authored(|s| 10.0 + last_index(spaced(text(s, "text"))) * 13.0)
        }
        "remocn/mask-reveal-up" => authored(|s| last_index(lines(text(s, "text"))) * 5.0),
        "remocn/line-by-line-slide" => authored(|s| last_index(lines(text(s, "text"))) * 6.0),
        "remocn/word-push" => authored(|s| {
            word_push_end(
                text(s, "text"),
                num(s, "wordGap", 10.0),
                num(s, "accel", 0.8),
            )
        }),
        "remocn/rgb-glitch-text" => {
            authored(|s| num(s, "glitchAt", 20.0) + num(s, "glitchDuration", 8.0))
        }
        "remocn/ink-underline" => authored(|s| num(s, "durationSteps", 5.0) * num(s, "step", 3.0)),
        "remocn/scribble-circle" => {
            authored(|s| num(s, "durationSteps", 10.0) * num(s, "step", 3.0))
        }
        "remocn/ink-arrow" => {
            authored(|s| num(s, "drawDur", 36.0) + num(s, "headDur", num(s, "step", 3.0) * 4.0))
        }
        "remocn/confetti" => authored(|s| num(s, "lifetime", 90.0)),
        "remocn/keystroke" => authored(keystroke_end),
        "remocn/terminal-simulator" => authored(terminal_end),
        "remocn/glass-code-block" => {
            authored(|s| last_index(lines(text(s, "code"))) * num(s, "staggerFrames", 4.0) + 8.0)
        }
        "remocn/code-morph" => authored(code_morph_end),
        "remocn/logo-enter" => {
            authored(|s| last_index(list(s, "logos").len()) * num(s, "stagger", 7.0))
        }
        "remocn/check-list" => authored(check_list_end),
        "remocn/search-reveal" => authored(|s| {
            let step = num(s, "framesPerCharacter", 4.0).clamp(1.0, 12.0);
            (54.0 + characters(text(s, "text")) as f64 * step + 10.0).max(90.0)
        }),
        "remocn/agent-run" => authored(agent_run_end),
        "remocn/signup-flow" => authored(|_| 348.0),
        "remocn/command-menu" => authored(|s| {
            if s.get("autoplay") == Some(&Value::Bool(false)) {
                0.0
            } else {
                24.0 + characters(text(s, "query")) as f64 * 2.5 + 8.0 + 18.0 + 6.0
            }
        }),
        "remocn/release-teaser" => seconds(|_| RELEASE_TEASER_SECONDS),
        "remocn/launch-anything" => seconds(|_| LAUNCH_ANYTHING_SECONDS),
        "remocn/animated-bar-chart" => {
            authored(|s| last_index(list(s, "bars").len()) * num(s, "staggerFrames", 6.0))
        }
        "remotion-ui/metric-ticker" => frames(|s| {
            (list(s, "metrics").len() as f64).min(num(s, "maxCards", 4.0).max(1.0)) * 8.0
        }),
        "remotion-ui/funnel-chart" => frames(|s| {
            last_index(list(s, "stages").len()) * num(s, "staggerInFrames", 12.0)
                + num(s, "durationInFrames", 22.0)
        }),
        "remotion-ui/donut-chart" => frames(|s| {
            last_index(list(s, "segments").len()) * num(s, "staggerInFrames", 9.0)
                + num(s, "durationInFrames", 26.0)
        }),
        "remotion-ui/comparison-bars" => frames(|s| {
            last_index(list(s, "rows").len()) * num(s, "staggerInFrames", 12.0)
                + num(s, "pairOffsetInFrames", 6.0)
                + num(s, "durationInFrames", 30.0)
        }),
        "remotion-ui/bar-chart-race" => frames(|s| {
            let steps = list(s, "series")
                .iter()
                .map(|row| {
                    field(row, "values")
                        .and_then(Value::as_array)
                        .map_or(0, Vec::len)
                })
                .max();
            last_index(steps.unwrap_or(0)) * num(s, "framesPerStep", 26.0)
        }),
        "remotion-ui/gauge-dial" => frames(|s| num(s, "durationInFrames", 44.0)),
        "remotion-ui/handwriting-text" => frames(|s| {
            last_index(characters(text(s, "text"))) * num(s, "staggerInFrames", 3.0)
                + num(s, "durationInFrames", 10.0)
        }),
        "remotion-ui/neon-flicker-text" => {
            frames(|s| last_index(characters(text(s, "text"))) * num(s, "staggerInFrames", 0.0))
        }
        "remotion-ui/text-mask-video" => frames(|s| num(s, "durationInFrames", 0.0)),
        "remotion-ui/pricing-card" => seconds(|s| {
            (num(s, "priceAtSeconds", 0.42) + num(s, "rollSeconds", 0.9)).max(
                num(s, "featuresAtSeconds", 1.05)
                    + last_index(list(s, "features").len()) * num(s, "featureStaggerSeconds", 0.22)
                    + 0.58,
            )
        }),
        "remotion-ui/comparison-table" => seconds(|s| {
            let rows = list(s, "rows");
            let cells = rows
                .iter()
                .map(|row| {
                    field(row, "cells")
                        .and_then(Value::as_array)
                        .map_or(0, Vec::len)
                })
                .max()
                .unwrap_or(0);
            num(s, "startAtSeconds", 0.5)
                + last_index(rows.len()) * num(s, "rowStaggerSeconds", 0.26)
                + (0.39 + last_index(cells) * 0.07).max(0.4)
        }),
        "remotion-ui/changelog-entry" => seconds(|s| {
            (num(s, "startAtSeconds", 0.66)
                + last_index(list(s, "changes").len()) * num(s, "staggerSeconds", 0.28)
                + 0.46)
                .max(0.84)
        }),
        "remotion-ui/logo-wall" => seconds(logo_wall_end),
        "remotion-ui/end-card" => seconds(end_card_end),
        "remotion-ui/code-reveal" => seconds(code_reveal_end),
        "remotion-ui/code-diff-wipe" => seconds(|s| 0.7 + num(s, "wipeSeconds", 1.7) + 1.4),
        "remotion-ui/kanban-move" => seconds(|s| {
            let cards = list(s, "cards");
            let moves = cards.iter().filter_map(|card| {
                let at = field(card, "moveAtSeconds").and_then(Value::as_f64)?;
                field(card, "moveTo")
                    .is_some()
                    .then(|| at + num(s, "moveSeconds", 0.62))
            });
            moves.fold(
                0.18 + last_index(cards.len()) * num(s, "dealStaggerSeconds", 0.08) + 0.42,
                f64::max,
            )
        }),
        "remotion-ui/file-tree-reveal" => seconds(|s| {
            let paths: Vec<&str> = list(s, "nodes")
                .iter()
                .map(|v| v.as_str().unwrap_or(""))
                .collect();
            0.42 + last_index(file_tree_rows(&paths)) * num(s, "rowStagger", 0.18)
                + 0.34
                + if text(s, "selectedPath").is_empty() {
                    0.0
                } else {
                    0.64
                }
        }),
        "remotion-ui/commit-graph" => seconds(|s| {
            num(s, "startAtSeconds", 0.34)
                + (list(s, "commits").len() as f64 - 1.0).max(1.0) * num(s, "stepSeconds", 0.36)
        }),
        "remotion-ui/notification-stack" => seconds(notification_end),
        "snapcn/punch-lines" => authored(punch_lines_end),
        "snapcn/status-cycle" => authored(|s| {
            num(s, "startAt", 0.0)
                + num(s, "introFrames", 24.0)
                + last_index(comma_list(text(s, "statuses")).len()) * num(s, "statusHold", 30.0)
        }),
        "snapcn/text-highlight" => authored(|s| {
            num(s, "startAt", 6.0)
                + num(s, "drawDuration", 14.0)
                + num(s, "holdDuration", 12.0)
                + num(s, "rushDuration", 24.0)
        }),
        "snapcn/logo-drift" => seconds(|s| num(s, "exitAt", 3.13) + num(s, "exitDuration", 0.0)),
        "snapcn/screen-recording" => authored(|s| {
            max_of(
                list(s, "camera")
                    .iter()
                    .map(|m| field_num(m, "at", 0.0) + field_num(m, "duration", 25.0)),
            )
        }),
        _ => return None,
    })
}

const SCENE_TEMPLATES: [&str; 2] = ["remocn/release-teaser", "remocn/launch-anything"];

fn timing_of(meta: &ElementMeta) -> Option<Timing> {
    if let Some(timing) = element_timing(&meta.key) {
        return Some(timing);
    }
    let own_clock = meta.kind == ElementKind::Component
        && (meta.key.starts_with("remocn/") || meta.key.starts_with("snapcn/"))
        && !SCENE_TEMPLATES.contains(&meta.key.as_str());
    own_clock.then_some(KEEPS_ITS_LENGTH)
}

pub fn fits_its_animation(meta: &ElementMeta) -> bool {
    timing_of(meta).is_some()
}

fn end_frames(timing: Timing, settings: &Settings, fps: f64) -> f64 {
    let end = (timing.end)(settings);
    match timing.unit {
        Unit::Seconds => end * fps,
        Unit::Authored => end * fps / AUTHORED_FPS,
        Unit::Frames => end,
    }
}

fn merged(base: &Settings, over: &Settings) -> Settings {
    let mut out = base.clone();
    for (k, v) in over {
        out.insert(k.clone(), v.clone());
    }
    out
}

pub fn animation_length(meta: &ElementMeta, params: &Settings, fps: f64) -> f64 {
    let Some(timing) = timing_of(meta) else {
        return meta.default_duration_in_frames;
    };
    let defaults = merged(
        &settings_in_frames(&meta.schema, &meta.default_props),
        &meta.fixed_props,
    );
    let settings = merged(
        &settings_in_frames(&meta.schema, &merged(&meta.default_props, params)),
        &meta.fixed_props,
    );
    let designed_hold = meta.default_duration_in_frames
        - end_frames(timing, &defaults, AUTHORED_FPS) / num(&defaults, "speed", 1.0);
    let hold = if designed_hold < 0.0 {
        CUT_OFF_HOLD
    } else {
        designed_hold
    };
    let scaled_hold = if timing.unit == Unit::Frames {
        hold
    } else {
        hold * fps / AUTHORED_FPS
    };
    let speed = num(&settings, "speed", 1.0);
    if speed <= 0.0 {
        return 1.0;
    }
    if timing.keeps_length {
        return (scaled_hold / speed).ceil().max(1.0);
    }
    (end_frames(timing, &settings, fps) / speed + scaled_hold)
        .ceil()
        .max(1.0)
}
