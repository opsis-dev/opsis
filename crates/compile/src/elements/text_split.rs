use opsis_scene::easing::Easing;
use opsis_scene::random::random;
use opsis_types::numbers::{float, whole};

use crate::elements::anim::{clamped, clamped_eased};
use crate::elements::draw::DrawFailure;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitMode {
    Chars,
    Words,
    Lines,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitOrder {
    Start,
    End,
    Center,
    Edges,
    Random,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Unit {
    pub text: String,
    pub index: usize,
    pub rank: usize,
    pub enter: f64,
    pub exit: f64,
    pub displace: f64,
    pub opacity: f64,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Word {
    pub units: Vec<Unit>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub words: Vec<Word>,
}

#[derive(Debug, Clone)]
pub struct SplitTiming {
    pub mode: SplitMode,
    pub stagger: Option<f64>,
    pub duration: Option<f64>,
    pub delay: f64,
    pub order: SplitOrder,
    pub seed: f64,
    pub exit: bool,
    pub exit_frames: Option<f64>,
    pub exit_at: Option<f64>,
    pub exit_stagger: Option<f64>,
}

impl Default for SplitTiming {
    fn default() -> Self {
        Self {
            mode: SplitMode::Chars,
            stagger: None,
            duration: None,
            delay: 0.0,
            order: SplitOrder::Start,
            seed: 1.0,
            exit: false,
            exit_frames: None,
            exit_at: None,
            exit_stagger: None,
        }
    }
}

const ENTER_OPACITY_LEAD: f64 = 0.55;
const EXIT_OPACITY_LEAD: f64 = 0.7;
const EXIT_DURATION_RATIO: f64 = 0.7;
const MIN_EXIT_FRAMES: f64 = 6.0;

pub const fn easing_enter() -> Easing {
    Easing::Bezier(0.16, 1.0, 0.3, 1.0)
}

pub fn easing_exit() -> Easing {
    Easing::ease_in(Easing::Cubic)
}

pub fn default_exit_frames(duration: f64) -> f64 {
    MIN_EXIT_FRAMES.max((duration * EXIT_DURATION_RATIO).round())
}

const fn default_stagger(mode: SplitMode) -> f64 {
    match mode {
        SplitMode::Chars => 2.0,
        SplitMode::Words => 4.0,
        SplitMode::Lines => 7.0,
    }
}

const fn default_duration(mode: SplitMode) -> f64 {
    match mode {
        SplitMode::Chars => 20.0,
        SplitMode::Words => 24.0,
        SplitMode::Lines => 28.0,
    }
}

fn pieces(text: &str, mode: SplitMode) -> Vec<Vec<Vec<String>>> {
    text.split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
        .map(|line| {
            let words: Vec<&str> = if mode == SplitMode::Lines {
                Some(line.trim())
                    .filter(|l| !l.is_empty())
                    .into_iter()
                    .collect()
            } else {
                line.split_whitespace().collect()
            };
            words
                .into_iter()
                .map(|word| {
                    if mode == SplitMode::Chars {
                        word.chars().map(String::from).collect()
                    } else {
                        vec![word.to_owned()]
                    }
                })
                .collect()
        })
        .collect()
}

fn rounded(value: f64) -> usize {
    usize::try_from(whole(value)).unwrap_or(0)
}

fn ranks(total: usize, order: SplitOrder, seed: f64) -> Vec<usize> {
    let middle = float(total.saturating_sub(1)) / 2.0;
    let indices = 0..total;
    match order {
        SplitOrder::End => indices.map(|i| total - 1 - i).collect(),
        SplitOrder::Center => indices
            .map(|i| rounded((float(i) - middle).abs()))
            .collect(),
        SplitOrder::Edges => indices
            .map(|i| rounded(middle - (float(i) - middle).abs()))
            .collect(),
        SplitOrder::Random => {
            let seed = opsis_scene::random::js_number_text(seed);
            let mut shuffled: Vec<usize> = indices.collect();
            shuffled.sort_by(|a, b| {
                random(format!("{seed}:{a}")).total_cmp(&random(format!("{seed}:{b}")))
            });
            let mut out = vec![0; total];
            for (rank, original) in shuffled.into_iter().enumerate() {
                out[original] = rank;
            }
            out
        }
        SplitOrder::Start => indices.collect(),
    }
}

pub fn split_text(
    text: &str,
    timing: &SplitTiming,
    frame: f64,
    window: f64,
) -> Result<Vec<Line>, DrawFailure> {
    let stagger = timing
        .stagger
        .unwrap_or_else(|| default_stagger(timing.mode));
    let duration = 1.0_f64.max(
        timing
            .duration
            .unwrap_or_else(|| default_duration(timing.mode))
            .round(),
    );
    let delay = 0.0_f64.max(timing.delay.round());
    let exit_frames = timing
        .exit_frames
        .map_or_else(|| default_exit_frames(duration), |f| 1.0_f64.max(f.round()));
    let exit_stagger = timing.exit_stagger.unwrap_or(stagger);
    let split = pieces(text, timing.mode);
    let total = split.iter().flatten().map(Vec::len).sum();
    let ranked = ranks(total, timing.order, timing.seed);
    let max_rank = float(ranked.iter().copied().max().unwrap_or(0));
    let last_enter = max_rank.mul_add(stagger, delay) + duration;
    let wants_exit = timing.exit || timing.exit_frames.is_some() || timing.exit_at.is_some();
    let requested = timing.exit_at.or_else(|| {
        window
            .is_finite()
            .then(|| max_rank.mul_add(-exit_stagger, window - exit_frames))
    });
    let exit_start = requested.filter(|_| wants_exit).map(|r| r.max(last_enter));
    let mut index = 0;
    let mut lines = Vec::new();
    for line in split {
        let mut words = Vec::new();
        for word in line {
            let mut units = Vec::new();
            for piece in word {
                let rank = ranked.get(index).copied().unwrap_or(index);
                let start = float(rank).mul_add(stagger, delay);
                let enter_time = clamped(frame, [start, start + duration], [0.0, 1.0])?;
                let enter = clamped_eased(enter_time, [0.0, 1.0], [0.0, 1.0], easing_enter())?;
                let enter_opacity = clamped(enter_time, [0.0, ENTER_OPACITY_LEAD], [0.0, 1.0])?;
                let unit = match exit_start {
                    None => Unit {
                        text: piece,
                        index,
                        rank,
                        enter,
                        exit: 0.0,
                        displace: 1.0 - enter,
                        opacity: enter_opacity,
                        value: enter,
                    },
                    Some(exit_start) => {
                        let unit_exit = float(rank).mul_add(exit_stagger, exit_start);
                        let exit_time =
                            clamped(frame, [unit_exit, unit_exit + exit_frames], [0.0, 1.0])?;
                        let exit = clamped_eased(exit_time, [0.0, 1.0], [0.0, 1.0], easing_exit())?;
                        let exit_opacity =
                            clamped(exit_time, [0.0, EXIT_OPACITY_LEAD], [1.0, 0.0])?;
                        Unit {
                            text: piece,
                            index,
                            rank,
                            enter,
                            exit,
                            displace: if exit > 0.0 { exit } else { 1.0 - enter },
                            opacity: enter_opacity * exit_opacity,
                            value: enter * (1.0 - exit),
                        }
                    }
                };
                units.push(unit);
                index += 1;
            }
            words.push(Word { units });
        }
        lines.push(Line { words });
    }
    Ok(lines)
}
