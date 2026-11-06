use std::f64::consts::PI;

use opsis_types::mix::{EqBand, EqType, SAMPLE_RATE};

pub const MIN_HZ: f64 = 20.0;
pub const MAX_HZ: f64 = 20_000.0;
pub const RANGE_DB: f64 = 18.0;
const HIGHPASS_POWER: i32 = 4;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Biquad {
    pub b: [f64; 3],
    pub a: [f64; 3],
}

fn shelf(amp: f64, cos_w0: f64, two_sqrt_amp_alpha: f64, high: bool) -> Biquad {
    let cos = if high { -cos_w0 } else { cos_w0 };
    let sign = if high { -1.0 } else { 1.0 };
    let lowered = (amp - 1.0).mul_add(-cos, amp + 1.0);
    let raised = (amp - 1.0).mul_add(cos, amp + 1.0);
    Biquad {
        b: [
            amp * (lowered + two_sqrt_amp_alpha),
            sign * 2.0 * amp * ((amp + 1.0).mul_add(-cos, amp - 1.0)),
            amp * (lowered - two_sqrt_amp_alpha),
        ],
        a: [
            raised + two_sqrt_amp_alpha,
            -sign * 2.0 * ((amp + 1.0).mul_add(cos, amp - 1.0)),
            raised - two_sqrt_amp_alpha,
        ],
    }
}

pub fn coefficients(band: &EqBand, sample_rate: f64) -> Biquad {
    let amp = 10f64.powf(band.gain / 40.0);
    let w0 = 2.0 * PI * band.freq / sample_rate;
    let cos_w0 = w0.cos();
    let alpha = w0.sin() / (2.0 * band.q.max(f64::EPSILON));
    match band.kind {
        EqType::Peak => Biquad {
            b: [
                alpha.mul_add(amp, 1.0),
                -2.0 * cos_w0,
                (-alpha).mul_add(amp, 1.0),
            ],
            a: [1.0 + alpha / amp, -2.0 * cos_w0, 1.0 - alpha / amp],
        },
        EqType::Lowshelf => shelf(amp, cos_w0, 2.0 * amp.sqrt() * alpha, false),
        EqType::Highshelf => shelf(amp, cos_w0, 2.0 * amp.sqrt() * alpha, true),
    }
}

fn magnitude(x: [f64; 3], w: f64) -> f64 {
    let re = x[2].mul_add((2.0 * w).cos(), x[1].mul_add(w.cos(), x[0]));
    let im = (-x[1]).mul_add(w.sin(), -(x[2] * (2.0 * w).sin()));
    re.hypot(im)
}

pub fn response_db(biquad: &Biquad, freq: f64, sample_rate: f64) -> f64 {
    let w = 2.0 * PI * freq / sample_rate;
    20.0 * (magnitude(biquad.b, w) / magnitude(biquad.a, w)).log10()
}

pub fn highpass_db(cutoff: f64, freq: f64) -> f64 {
    -10.0 * (cutoff / freq).powi(HIGHPASS_POWER).ln_1p() / std::f64::consts::LN_10
}

pub fn curve(bands: &[EqBand], on: bool, highpass: Option<f64>, steps: u32) -> Vec<f64> {
    let rate = f64::from(SAMPLE_RATE);
    let biquads: Vec<Biquad> = if on {
        bands.iter().map(|band| coefficients(band, rate)).collect()
    } else {
        Vec::new()
    };
    let steps = steps.max(1);
    (0..=steps)
        .map(|i| {
            let freq = MIN_HZ * (MAX_HZ / MIN_HZ).powf(f64::from(i) / f64::from(steps));
            let mut db: f64 = biquads.iter().map(|q| response_db(q, freq, rate)).sum();
            if let Some(cutoff) = highpass {
                db += highpass_db(cutoff, freq);
            }
            db.clamp(-RANGE_DB, RANGE_DB)
        })
        .collect()
}
