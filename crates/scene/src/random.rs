use opsis_types::numbers::{exact_eq, float_wide};

const FNV_OFFSET: u32 = 0x811c_9dc5;
const FNV_PRIME: u32 = 0x0100_0193;
const MIX: u32 = 0x6d2b_79f5;
const TWO_TO_32: f64 = 4_294_967_296.0;
const EXPONENT_ABOVE: f64 = 1e21;
const EXPONENT_BELOW: f64 = 1e-6;

#[derive(Debug, Clone, PartialEq)]
pub enum Seed {
    Number(f64),
    Text(String),
}

impl From<f64> for Seed {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl From<u32> for Seed {
    fn from(value: u32) -> Self {
        Self::Number(f64::from(value))
    }
}

impl From<i32> for Seed {
    fn from(value: i32) -> Self {
        Self::Number(f64::from(value))
    }
}

impl From<&str> for Seed {
    fn from(value: &str) -> Self {
        Self::Text(value.to_owned())
    }
}

impl From<String> for Seed {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

fn with_exponent_sign(text: &str) -> String {
    match text.split_once('e') {
        Some((mantissa, exponent)) if !exponent.starts_with('-') => {
            format!("{mantissa}e+{exponent}")
        }
        _ => text.to_owned(),
    }
}

pub fn js_number_text(value: f64) -> String {
    if value.is_nan() {
        return "NaN".to_owned();
    }
    if value.is_infinite() {
        return if value > 0.0 { "Infinity" } else { "-Infinity" }.to_owned();
    }
    if exact_eq(value, 0.0) {
        return "0".to_owned();
    }
    let size = value.abs();
    if size >= EXPONENT_ABOVE || size < EXPONENT_BELOW {
        with_exponent_sign(&format!("{value:e}"))
    } else {
        format!("{value}")
    }
}

fn seed_text(seed: &Seed) -> String {
    match seed {
        Seed::Number(value) => js_number_text(*value),
        Seed::Text(text) => text.clone(),
    }
}

fn hashed(text: &str) -> u32 {
    text.encode_utf16().fold(FNV_OFFSET, |hash, unit| {
        (hash ^ u32::from(unit)).wrapping_mul(FNV_PRIME)
    })
}

pub fn random(seed: impl Into<Seed>) -> f64 {
    let mut state = hashed(&seed_text(&seed.into())).wrapping_add(MIX);
    state = (state ^ (state >> 15)).wrapping_mul(state | 1);
    state ^= state.wrapping_add((state ^ (state >> 7)).wrapping_mul(state | 0x3d));
    float_wide(u64::from(state ^ (state >> 14))) / TWO_TO_32
}
