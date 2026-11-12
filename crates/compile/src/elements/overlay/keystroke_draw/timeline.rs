use opsis_scene::clock::js_round;
use opsis_scene::easing::Easing;
use opsis_types::numbers::float;

use crate::elements::overlay::keystroke::Keystroke;
use crate::elements::overlay::keystroke_draw::keys::{Key, parse_keys};

const PRESS: f64 = 3.0;
const PRESS_GAP: f64 = 3.0;
const RELEASE_GAP: f64 = 3.0;
const LEAD: f64 = 7.0;
const BEAT: f64 = 2.0;
const RELEASE: f64 = 10.0;
const RELEASE_RISE: f64 = 3.0;
const RELEASE_TAPER: f64 = 6.0;
const RELEASE_DAMPING: f64 = 0.46;
const ENTER: f64 = 10.0;
const ENTER_RISE: f64 = 4.0;
const ENTER_TAPER: f64 = 7.0;
const ENTER_DAMPING: f64 = 0.65;
const BRIEF: f64 = 18.0;
pub const EXIT: f64 = 8.0;
const MAX_FRAMES: f64 = 600.0;
const SEPARATOR: f64 = 0.95;
const KEY_GAP: f64 = 0.14;

pub fn round3(value: f64) -> f64 {
    js_round(value * 1000.0) / 1000.0 + 0.0
}

pub fn smoothstep(value: f64) -> f64 {
    let t = value.clamp(0.0, 1.0);
    t * t * 2.0_f64.mul_add(-t, 3.0)
}

pub fn eased(frames: f64, duration: f64, curve: &dyn Fn(f64) -> f64) -> f64 {
    if frames.is_nan() || frames <= 0.0 {
        return 0.0;
    }
    if frames >= duration {
        return 1.0;
    }
    curve(frames / duration)
}

pub fn glide(t: f64) -> f64 {
    Easing::Bezier(0.33, 1.0, 0.68, 1.0).at(t)
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film page's JavaScript rounds every step, and the pictures have to match it"
)]
fn settle(rise: f64, damping: f64, taper: f64, end: f64, frames: f64) -> f64 {
    let frequency = (std::f64::consts::PI - damping.acos()) / rise;
    let ratio = damping / (1.0 - damping.powi(2)).sqrt();
    if frames.is_nan() || frames <= 0.0 {
        return 0.0;
    }
    if frames >= end {
        return 1.0;
    }
    let phase = frequency * frames;
    let decay = (-ratio * phase).exp();
    let ring = decay * (phase.cos() + ratio * phase.sin());
    1.0 - ring * (1.0 - smoothstep((frames - taper) / (end - taper)))
}

pub fn rise(frames: f64) -> f64 {
    settle(ENTER_RISE, ENTER_DAMPING, ENTER_TAPER, ENTER, frames)
}

fn press(frames: f64) -> f64 {
    eased(frames, PRESS, &smoothstep)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KeyTiming {
    pub press: f64,
    pub release: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChordTiming {
    pub enter: f64,
    pub fire: f64,
    pub keys: Vec<KeyTiming>,
}

pub fn depth(time: f64, timing: KeyTiming) -> f64 {
    if time.is_nan() || time <= timing.press {
        return 0.0;
    }
    if time < timing.release {
        return press(time - timing.press);
    }
    let bottom = press(timing.release - timing.press);
    bottom
        * (1.0
            - settle(
                RELEASE_RISE,
                RELEASE_DAMPING,
                RELEASE_TAPER,
                RELEASE,
                time - timing.release,
            ))
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cap {
    pub chord: usize,
    pub index: usize,
    pub x: f64,
    pub width: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Separator {
    pub chord: usize,
    pub x: f64,
    pub width: f64,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Layout {
    pub caps: Vec<Cap>,
    pub separators: Vec<Separator>,
    pub widths: Vec<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Group {
    pub step: usize,
    pub chords: Vec<Vec<Key>>,
    pub timing: Vec<ChordTiming>,
    pub at: f64,
    pub enter: f64,
    pub last_release: f64,
    pub layout: Layout,
    pub pushes: Vec<f64>,
    pub exit: f64,
    pub end: f64,
}

fn layout_of(chords: &[Vec<Key>]) -> Layout {
    let mut layout = Layout::default();
    let mut cursor = 0.0;
    for (chord, keys) in chords.iter().enumerate() {
        if chord > 0 {
            layout.separators.push(Separator {
                chord,
                x: round3(cursor),
                width: SEPARATOR,
            });
            cursor += SEPARATOR;
        }
        for (index, key) in keys.iter().enumerate() {
            if index > 0 {
                cursor += KEY_GAP;
            }
            layout.caps.push(Cap {
                chord,
                index,
                x: round3(cursor),
                width: key.units,
            });
            cursor += key.units;
        }
        layout.widths.push(round3(cursor));
    }
    layout
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film page's JavaScript rounds every step, and the pictures have to match it"
)]
fn schedule(chords: &[Vec<Key>], hold: f64) -> Vec<ChordTiming> {
    let mut timing = Vec::new();
    let mut enter = 0.0;
    for chord in chords {
        let last = float(chord.len().saturating_sub(1));
        let fire = enter + LEAD + PRESS_GAP * last + PRESS;
        let keys = (0..chord.len())
            .map(|index| {
                let behind = last - float(index);
                KeyTiming {
                    press: fire - PRESS - PRESS_GAP * behind,
                    release: fire + hold + RELEASE_GAP * behind,
                }
            })
            .collect();
        timing.push(ChordTiming { enter, fire, keys });
        enter = fire + hold + RELEASE_GAP * last + BEAT;
    }
    timing
}

const fn within(value: f64, min: f64, max: f64) -> f64 {
    value.clamp(min, max)
}

pub fn timeline(props: &Keystroke, fps: f64) -> Vec<Group> {
    let usual_hold = within(props.hold, 0.0, MAX_FRAMES);
    let scale = 30.0 / fps.max(1.0);
    let linger = within(props.linger, 0.0, MAX_FRAMES) * scale;
    let mut drafts = Vec::new();
    for (step, entry) in props.steps.iter().enumerate() {
        if !entry.at.is_finite() {
            continue;
        }
        let chords = parse_keys(&entry.keys, props.platform);
        if chords.is_empty() {
            continue;
        }
        let at = entry.at * scale;
        let hold = within(entry.hold.unwrap_or(usual_hold), 0.0, MAX_FRAMES) * scale;
        let planned = schedule(&chords, hold);
        let offset = at - planned.last().map_or(0.0, |c| c.fire);
        let timing: Vec<ChordTiming> = planned
            .iter()
            .map(|chord| ChordTiming {
                enter: round3(chord.enter + offset),
                fire: round3(chord.fire + offset),
                keys: chord
                    .keys
                    .iter()
                    .map(|key| KeyTiming {
                        press: round3(key.press + offset),
                        release: round3(key.release + offset),
                    })
                    .collect(),
            })
            .collect();
        let last_release = timing
            .iter()
            .flat_map(|c| c.keys.iter().map(|k| k.release))
            .fold(f64::NEG_INFINITY, f64::max);
        let enter = timing.first().map_or(0.0, |c| c.enter);
        let layout = layout_of(&chords);
        drafts.push(Group {
            step,
            chords,
            timing,
            at: round3(at),
            enter,
            last_release,
            layout,
            pushes: Vec::new(),
            exit: 0.0,
            end: 0.0,
        });
    }
    drafts.sort_by(|a, b| {
        a.enter
            .total_cmp(&b.enter)
            .then(a.at.total_cmp(&b.at))
            .then(a.step.cmp(&b.step))
    });
    let mut groups = drafts.clone();
    for (index, group) in groups.iter_mut().enumerate() {
        let mut exit = group.enter.max(group.last_release + linger);
        for newer in &drafts[index + 1..] {
            if newer.enter >= exit + EXIT {
                break;
            }
            let at = newer.enter.max(group.enter);
            let brief = at.max(group.last_release) + BRIEF;
            group.pushes.push(at);
            exit = exit.min(if group.pushes.len() == 1 { brief } else { at });
            if group.pushes.len() == 2 {
                break;
            }
        }
        group.exit = round3(exit.max(group.enter));
        group.end = round3(group.exit + EXIT);
    }
    groups
}
