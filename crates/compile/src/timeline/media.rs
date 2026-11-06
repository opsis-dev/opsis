use opsis_types::analysis::WaveformLevel;

const SILENT_AMPLITUDE: f64 = 1e-4;
const DISPLAY_RANGE_DB: f64 = 48.0;
pub const WAVE_FILL: f64 = 0.95;

pub fn shaped(value: f64) -> f64 {
    let amplitude = value.abs().min(1.0);
    if amplitude < SILENT_AMPLITUDE {
        return 0.0;
    }
    value.signum() * (20.0 * amplitude.log10() / DISPLAY_RANGE_DB + 1.0).max(0.0)
}

pub fn pick_level(levels: &[WaveformLevel], px_per_second: f64) -> Option<&WaveformLevel> {
    let mut sorted: Vec<&WaveformLevel> = levels.iter().collect();
    sorted.sort_by(|a, b| a.rate.total_cmp(&b.rate));
    sorted
        .iter()
        .find(|level| level.rate >= px_per_second)
        .or_else(|| sorted.last())
        .copied()
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WaveSpan {
    pub from: f64,
    pub to: f64,
    pub trim_before: f64,
    pub looped: bool,
    pub volume: f64,
    pub duration_s: f64,
}

pub fn wave_column(
    level: &WaveformLevel,
    span: WaveSpan,
    abs: f64,
    fps: f64,
    gain: f64,
) -> Option<(f64, f64)> {
    if abs < span.from || abs >= span.to {
        return None;
    }
    let mut seconds = (abs - span.from + span.trim_before) / fps;
    if span.looped && span.duration_s > 0.0 {
        seconds %= span.duration_s;
    }
    let index = (seconds * level.rate).floor();
    if index < 0.0 {
        return None;
    }
    let at = opsis_types::numbers::truncate::<usize>(index);
    let (high, low) = (level.max.get(at)?, level.min.get(at)?);
    let volume = span.volume * gain;
    Some((shaped(high * volume), shaped(low * volume)))
}

pub fn tile_start(left: f64, offset: f64, tile_width: f64) -> f64 {
    ((left + offset) / tile_width)
        .floor()
        .mul_add(tile_width, -left)
}

pub fn tile_index(source_ms: f64, interval_ms: f64, count: u32) -> u32 {
    if count == 0 || interval_ms <= 0.0 {
        return 0;
    }
    let index = (source_ms / interval_ms).floor().max(0.0);
    opsis_types::numbers::truncate::<u32>(index).min(count - 1)
}
