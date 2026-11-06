use opsis_types::mix::{MIN_DB, SILENCE_DB};

pub const SWEEP_DEG: f64 = 270.0;
pub const START_DEG: f64 = -SWEEP_DEG / 2.0;
const SILENT_POWER: f64 = 1e-12;
const MINUS: &str = "\u{2212}";
const WHOLE_MS_FROM: f64 = 100.0;
const KILOHERTZ_FROM: f64 = 1000.0;
const WHOLE_KILOHERTZ_FROM: f64 = 10_000.0;
const CENTRE_PAN: f64 = 0.01;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KnobRange {
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub log: bool,
}

fn round_to_step(value: f64, step: f64) -> f64 {
    if step <= 0.0 {
        return value;
    }
    let decimals = (-step.log10().floor()).max(0.0);
    let scale = 10f64.powf(decimals);
    ((value / step).round() * step * scale).round() / scale
}

pub fn value_to_fraction(range: KnobRange, value: f64) -> f64 {
    if range.log {
        (value / range.min).log(range.max / range.min).clamp(0.0, 1.0)
    } else if range.max > range.min {
        ((value - range.min) / (range.max - range.min)).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

pub fn fraction_to_value(range: KnobRange, fraction: f64) -> f64 {
    let c = fraction.clamp(0.0, 1.0);
    let value = if range.log {
        range.min * (range.max / range.min).powf(c)
    } else {
        c.mul_add(range.max - range.min, range.min)
    };
    round_to_step(value, range.step).clamp(range.min, range.max)
}

pub fn snap_value(range: KnobRange, value: f64) -> f64 {
    round_to_step(value.clamp(range.min, range.max), range.step).clamp(range.min, range.max)
}

pub const fn angle_of(fraction: f64) -> f64 {
    fraction.mul_add(SWEEP_DEG, START_DEG)
}

pub fn sample_at(values: &[f64], frame: f64) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let last = opsis_types::numbers::float(values.len() - 1);
    let at = frame.round().clamp(0.0, last);
    values
        .get(opsis_types::numbers::truncate::<usize>(at))
        .copied()
}

fn power(db: f64) -> f64 {
    10f64.powf(db / 10.0)
}

pub fn share(master_rms: &[f64], channel_rms: &[f64], frame: f64) -> f64 {
    let (Some(master), Some(channel)) =
        (sample_at(master_rms, frame), sample_at(channel_rms, frame))
    else {
        return 1.0;
    };
    let pm = power(master);
    if pm <= SILENT_POWER {
        1.0
    } else {
        (power(channel) / pm).min(1.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Level {
    pub peak: f64,
    pub rms: f64,
    pub hold: f64,
}

pub fn level_at(peak: &[f64], rms: &[f64], frame: f64, hold_frames: usize) -> Option<Level> {
    let at = opsis_types::numbers::truncate::<usize>(
        frame
            .round()
            .clamp(0.0, opsis_types::numbers::float(peak.len().checked_sub(1)?)),
    );
    let hold = peak[at.saturating_sub(hold_frames)..=at]
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    Some(Level {
        peak: *peak.get(at)?,
        rms: rms.get(at).copied().unwrap_or(f64::NEG_INFINITY),
        hold,
    })
}

pub fn db_to_gain(db: f64) -> f64 {
    if db <= MIN_DB {
        0.0
    } else {
        10f64.powf(db / 20.0)
    }
}

fn signed_text(value: f64, digits: usize, plus: bool) -> String {
    let text = format!("{:.digits$}", value.abs());
    if text.parse::<f64>().is_ok_and(|v| v == 0.0) {
        return text;
    }
    let sign = if value < 0.0 {
        MINUS
    } else if plus && value > 0.0 {
        "+"
    } else {
        ""
    };
    format!("{sign}{text}")
}

pub fn format_reading(value: f64, floor: f64, plus: bool) -> String {
    if value.is_nan() || value <= floor {
        format!("{MINUS}\u{221e}")
    } else {
        signed_text(value, 1, plus)
    }
}

pub fn format_db(value: f64) -> String {
    format_reading(value, MIN_DB, true)
}

pub fn format_level_db(value: f64) -> String {
    format_reading(value, SILENCE_DB, true)
}

pub fn format_signed_db(value: f64) -> String {
    signed_text(value, 1, true)
}

pub fn format_hz(value: f64) -> String {
    if value >= KILOHERTZ_FROM {
        let digits = usize::from(value < WHOLE_KILOHERTZ_FROM);
        format!("{:.digits$}k", value / KILOHERTZ_FROM)
    } else {
        format!("{}", value.round())
    }
}

pub fn format_ms(value: f64) -> String {
    if value >= WHOLE_MS_FROM {
        format!("{}", value.round())
    } else {
        format!("{value:.1}")
    }
}

pub fn format_pan(value: f64) -> String {
    if value.abs() < CENTRE_PAN {
        "C".to_owned()
    } else if value < 0.0 {
        format!("L{}", (-value * 100.0).round())
    } else {
        format!("R{}", (value * 100.0).round())
    }
}
