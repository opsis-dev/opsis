use std::collections::BTreeSet;

use opsis_types::numbers;
use opsis_types::numbers::whole;
use serde_json::{Map, Value};

use crate::elements::ElementMeta;
use crate::elements::introspect::settings_in_frames;

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

pub const AUTHORED_FPS: u32 = 30;
const CUT_OFF_SECONDS: f64 = 0.5;
const EXIT_STAGGER: f64 = 2.0;
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
    numbers::float(count.saturating_sub(1))
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

fn word_push_end(text: &str, word_gap: f64, accel: f64, push: f64) -> f64 {
    let count = words(text);
    if count == 0 {
        return 0.0;
    }
    let index = numbers::float(count - 1);
    let reveal = if accel == 1.0 {
        index * word_gap
    } else {
        word_gap * (1.0 - accel.powf(index)) / (1.0 - accel)
    };
    reveal.ceil() + push
}

const KEYSTROKE_HOLD: f64 = 8.0;
const KEYSTROKE_EXIT: f64 = 8.0;

fn keystroke_end(s: &Settings) -> f64 {
    let linger = num(s, "linger", 30.0);
    let hold = num(s, "hold", KEYSTROKE_HOLD);
    max_of(list(s, "steps").iter().filter_map(|step| {
        let at = finite(field(step, "at"))?;
        (!field_str(step, "keys").trim().is_empty())
            .then(|| at + field_num(step, "hold", hold) + linger + KEYSTROKE_EXIT)
    }))
}

const TERMINAL_START: f64 = 10.0;
const TERMINAL_DELAY: f64 = 8.0;
const TERMINAL_ELLIPSIS_PAUSE: f64 = 18.0;

fn terminal_end(s: &Settings) -> f64 {
    let per_frame = (num(s, "chunkSize", 1.0) * num(s, "charsPerFrame", 1.0)).max(0.01);
    let ellipsis = num(s, "ellipsisPause", TERMINAL_ELLIPSIS_PAUSE);
    list(s, "lines")
        .iter()
        .fold(num(s, "startAt", TERMINAL_START), |end, line| {
            let text = field_str(line, "text");
            let pause = match field(line, "pause").and_then(Value::as_f64) {
                Some(p) => p,
                None if text.trim_end().ends_with("...") => ellipsis,
                None => 0.0,
            };
            let typing = (numbers::float(text.encode_utf16().count()) / per_frame).ceil();
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
    let highlight = last
        + MORPH_ENTER_START * morph / MORPH_BASE
        + MORPH_RISE
        + num(s, "highlightHold", MORPH_SHOW);
    if s.get("highlight") == Some(&Value::Bool(false)) {
        last + morph
    } else {
        (last + morph).max(highlight + MORPH_FALL)
    }
}

fn handwrite_frames(text: &str, per_step: f64, step: f64) -> f64 {
    (numbers::float(characters(text)) / per_step).ceil() * step + step
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
        numbers::float(i) * item_gap + step * 2.0 + handwrite_frames(text, per_step, step)
    }));
    let checked = items.iter().filter(|(_, c)| *c).count();
    let strike_end = if checked > 0 {
        enter_end + numbers::float(checked - 1) * close_gap + step * 4.0
    } else {
        0.0
    };
    enter_end.max(strike_end)
}

const AGENT_PLAN_PER_WORD: f64 = 0.72;
const AGENT_ANSWER_PER_WORD: f64 = 1.0;
const AGENT_ANSWER_PER_LINE: f64 = 1.0;

fn agent_run_end(s: &Settings) -> f64 {
    let per_frame = num(s, "compression", 3.0).clamp(0.25, 20.0) / f64::from(AUTHORED_FPS);
    let plan: Vec<&str> = list(s, "runPlan")
        .iter()
        .map(|v| v.as_str().unwrap_or(""))
        .collect();
    let plan = plan.join("\n");
    let answer = text(s, "runAnswer");
    let think = 16.0 + 14.0 + numbers::float(words(&plan)) * AGENT_PLAN_PER_WORD + 8.0;
    let tools: f64 = list(s, "runSteps")
        .iter()
        .map(|step| {
            5.0 + (field_num(step, "duration", 0.0) / per_frame)
                .round()
                .clamp(12.0, 120.0)
        })
        .sum();
    let answering = numbers::float(words(answer)) * AGENT_ANSWER_PER_WORD
        + numbers::float(lines(answer)) * AGENT_ANSWER_PER_LINE;
    think + tools + 8.0 + answering + 16.0
}

fn logo_wall_end(s: &Settings) -> f64 {
    let count = list(s, "logos").len();
    let columns = numbers::truncate::<usize>(num(s, "columns", 4.0).round().max(1.0));
    let start_at = num(s, "startAtSeconds", 0.32);
    let diagonal = (0..count)
        .map(|i| numbers::float(i / columns + i % columns))
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
            1.24 + numbers::float(url.encode_utf16().count()) / 26.0
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
    0.4 + numbers::float(typed) / 55.0 + last_index(rows.len()) * 0.07 + 0.84
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
            let spawn = field_num(toast, "atSeconds", start_at + stagger * numbers::float(i));
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
        "soft-blur-in" => {
            authored(|s| last_index(characters(text(s, "text"))) * num(s, "stagger", 1.0) + 27.0)
        }
        "typewriter" => authored(|s| {
            numbers::float(characters(text(s, "text"))) / num(s, "charsPerSecond", 22.0).max(0.1)
                * f64::from(AUTHORED_FPS)
        }),
        "matrix-decode" => authored(|s| num(s, "revealDuration", 60.0)),
        "slot-machine-roll" => authored(|s| {
            last_index(
                text(s, "from")
                    .encode_utf16()
                    .count()
                    .max(text(s, "to").encode_utf16().count()),
            ) * num(s, "columnStagger", 4.0)
        }),
        "staggered-fade-up" => authored(|s| {
            last_index(spaced(text(s, "text"))) * num(s, "staggerDelay", 4.0)
                + num(s, "wordDuration", 12.0)
        }),
        "kinetic-center-build" => authored(|s| {
            let word = num(s, "wordTime", 13.0);
            (word * 10.0 / 13.0).round() + last_index(spaced(text(s, "text"))) * word
        }),
        "mask-reveal-up" => authored(|s| {
            last_index(lines(text(s, "text"))) * (num(s, "lineStagger", 3.0) + EXIT_STAGGER)
        }),
        "line-by-line-slide" => authored(|s| {
            last_index(lines(text(s, "text"))) * (num(s, "lineStagger", 4.0) + EXIT_STAGGER)
        }),
        "word-push" => authored(|s| {
            word_push_end(
                text(s, "text"),
                num(s, "wordGap", 10.0),
                num(s, "accel", 0.8),
                num(s, "pushTime", 12.0),
            )
        }),
        "rgb-glitch-text" => authored(|s| num(s, "glitchAt", 20.0) + num(s, "glitchDuration", 8.0)),
        "ink-underline" => authored(|s| num(s, "durationSteps", 5.0) * num(s, "step", 3.0)),
        "scribble-circle" => authored(|s| num(s, "durationSteps", 10.0) * num(s, "step", 3.0)),
        "ink-arrow" => {
            authored(|s| num(s, "drawDur", 36.0) + num(s, "headDur", num(s, "step", 3.0) * 4.0))
        }
        "confetti" => authored(|s| num(s, "lifetime", 90.0)),
        "keystroke" => authored(keystroke_end),
        "terminal-simulator" => authored(terminal_end),
        "glass-code-block" => {
            authored(|s| last_index(lines(text(s, "code"))) * num(s, "staggerFrames", 4.0) + 8.0)
        }
        "code-morph" => authored(code_morph_end),
        "logo-enter" => authored(|s| last_index(list(s, "logos").len()) * num(s, "stagger", 7.0)),
        "check-list" => authored(check_list_end),
        "search-reveal" => authored(|s| {
            let step = num(s, "framesPerCharacter", 4.0).clamp(1.0, 12.0);
            (54.0 + numbers::float(characters(text(s, "text"))) * step + 10.0).max(90.0)
        }),
        "agent-run" => authored(agent_run_end),
        "signup-flow" => {
            authored(|s| num(s, "loadingSeconds", 42.0) + num(s, "noticeSeconds", 66.0) + 240.0)
        }
        "ai-prompt-flow" => authored(|s| {
            num(s, "typingSeconds", 50.0)
                + num(s, "loadingSeconds", 86.0)
                + num(s, "noticeSeconds", 60.0)
                + 38.0
        }),
        "command-menu" => authored(|s| {
            if s.get("autoplay") == Some(&Value::Bool(false)) {
                0.0
            } else {
                24.0 + numbers::float(characters(text(s, "query"))) * num(s, "letterSeconds", 2.5)
                    + 8.0
                    + num(s, "pauseSeconds", 18.0)
                    + 6.0
            }
        }),
        "release-teaser" => seconds(|_| RELEASE_TEASER_SECONDS),
        "launch-anything" => seconds(|_| LAUNCH_ANYTHING_SECONDS),
        "animated-bar-chart" => {
            authored(|s| last_index(list(s, "bars").len()) * num(s, "staggerFrames", 6.0))
        }
        "metric-ticker" => frames(|s| {
            numbers::float(list(s, "metrics").len()).min(num(s, "maxCards", 4.0).max(1.0)) * 8.0
        }),
        "funnel-chart" => frames(|s| {
            last_index(list(s, "stages").len()) * num(s, "staggerInFrames", 12.0)
                + num(s, "durationInFrames", 22.0)
        }),
        "donut-chart" => frames(|s| {
            last_index(list(s, "segments").len()) * num(s, "staggerInFrames", 9.0)
                + num(s, "durationInFrames", 26.0)
        }),
        "comparison-bars" => frames(|s| {
            last_index(list(s, "rows").len()) * num(s, "staggerInFrames", 12.0)
                + num(s, "pairOffsetInFrames", 6.0)
                + num(s, "durationInFrames", 30.0)
        }),
        "bar-chart-race" => frames(|s| {
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
        "gauge-dial" => frames(|s| num(s, "durationInFrames", 44.0)),
        "handwriting-text" => frames(|s| {
            last_index(characters(text(s, "text"))) * num(s, "staggerInFrames", 3.0)
                + num(s, "durationInFrames", 10.0)
        }),
        "neon-flicker-text" => {
            frames(|s| last_index(characters(text(s, "text"))) * num(s, "staggerInFrames", 0.0))
        }
        "text-mask-video" => frames(|s| num(s, "durationInFrames", 0.0)),
        "pricing-card" => seconds(|s| {
            (num(s, "priceAtSeconds", 0.42) + num(s, "rollSeconds", 0.9)).max(
                num(s, "featuresAtSeconds", 1.05)
                    + last_index(list(s, "features").len()) * num(s, "featureStaggerSeconds", 0.22)
                    + 0.58,
            )
        }),
        "comparison-table" => seconds(|s| {
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
        "changelog-entry" => seconds(|s| {
            (num(s, "startAtSeconds", 0.66)
                + last_index(list(s, "changes").len()) * num(s, "staggerSeconds", 0.28)
                + 0.46)
                .max(0.84)
        }),
        "logo-wall" => seconds(logo_wall_end),
        "end-card" => seconds(end_card_end),
        "code-reveal" => seconds(code_reveal_end),
        "code-diff-wipe" => seconds(|s| 0.7 + num(s, "wipeSeconds", 1.7) + 1.4),
        "kanban-move" => seconds(|s| {
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
        "file-tree-reveal" => seconds(|s| {
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
        "commit-graph" => seconds(|s| {
            num(s, "startAtSeconds", 0.34)
                + (numbers::float(list(s, "commits").len()) - 1.0).max(1.0)
                    * num(s, "stepSeconds", 0.36)
        }),
        "notification-stack" => seconds(notification_end),
        "punch-lines" => authored(punch_lines_end),
        "status-cycle" => authored(|s| {
            num(s, "startAt", 0.0)
                + num(s, "introFrames", 24.0)
                + last_index(comma_list(text(s, "statuses")).len()) * num(s, "statusHold", 30.0)
        }),
        "text-highlight" => authored(|s| {
            num(s, "startAt", 6.0)
                + num(s, "drawDuration", 14.0)
                + num(s, "holdDuration", 12.0)
                + num(s, "rushDuration", 24.0)
        }),
        "logo-drift" => seconds(|s| num(s, "exitAt", 3.13) + num(s, "exitDuration", 0.0)),
        "captured-video" => authored(|s| {
            max_of(
                list(s, "camera")
                    .iter()
                    .map(|m| field_num(m, "at", 0.0) + field_num(m, "duration", 25.0)),
            )
        }),
        _ => return None,
    })
}

fn timing_of(meta: &ElementMeta) -> Option<Timing> {
    if let Some(timing) = element_timing(&meta.key) {
        return Some(timing);
    }
    meta.authored_clock.then_some(KEEPS_ITS_LENGTH)
}

pub fn fits_its_animation(meta: &ElementMeta) -> bool {
    timing_of(meta).is_some()
}

fn end_frames(timing: Timing, settings: &Settings, fps: u32) -> f64 {
    let end = (timing.end)(settings);
    let fps = f64::from(fps);
    match timing.unit {
        Unit::Seconds => end * fps,
        Unit::Authored => end * fps / f64::from(AUTHORED_FPS),
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

pub fn animation_length(meta: &ElementMeta, params: &Settings, fps: u32) -> u32 {
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
    let authored = f64::from(AUTHORED_FPS);
    let designed_hold = f64::from(meta.default_duration_in_frames)
        - end_frames(timing, &defaults, AUTHORED_FPS) / num(&defaults, "speed", 1.0);
    let hold = if designed_hold < 0.0 {
        CUT_OFF_SECONDS * authored
    } else {
        designed_hold
    };
    let scaled_hold = if timing.unit == Unit::Frames {
        hold
    } else {
        hold * f64::from(fps) / authored
    };
    let speed = num(&settings, "speed", 1.0);
    if speed <= 0.0 {
        return 1;
    }
    if timing.keeps_length {
        return whole((scaled_hold / speed).ceil()).max(1);
    }
    whole((end_frames(timing, &settings, fps) / speed + scaled_hold).ceil()).max(1)
}
