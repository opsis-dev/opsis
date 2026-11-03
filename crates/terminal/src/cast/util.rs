use std::time::Duration;

use serde::{Deserialize, Deserializer};

const MICROS_PER_SECOND: u64 = 1_000_000;
const MICRO_DIGITS: usize = 6;

pub fn deserialize_time<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    let value: serde_json::Value = Deserialize::deserialize(deserializer)?;
    let number = value
        .as_f64()
        .map(|v| v.to_string())
        .ok_or_else(|| Error::custom("expected number"))?;
    let parts: Vec<&str> = number.split('.').collect();
    match parts.as_slice() {
        [left, right] => {
            let secs: u64 = left.parse().map_err(Error::custom)?;
            let right = right.trim();
            let micros: u64 = format!("{:0<6}", &right[..MICRO_DIGITS.min(right.len())])
                .parse()
                .map_err(Error::custom)?;
            Ok(Duration::from_micros(secs * MICROS_PER_SECOND + micros))
        }
        [number] => {
            let secs: u64 = number.parse().map_err(Error::custom)?;
            Ok(Duration::from_micros(secs * MICROS_PER_SECOND))
        }
        _ => Err(Error::custom(format!("invalid time format: {value}"))),
    }
}

#[derive(Debug, Default)]
pub struct Utf8Decoder(Vec<u8>);

impl Utf8Decoder {
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    pub fn feed(&mut self, input: &[u8]) -> String {
        let mut output = String::new();
        self.0.extend_from_slice(input);
        while !self.0.is_empty() {
            match std::str::from_utf8(&self.0) {
                Ok(valid) => {
                    output.push_str(valid);
                    self.0.clear();
                    break;
                }
                Err(e) => {
                    let n = e.valid_up_to();
                    let valid: Vec<u8> = self.0.drain(..n).collect();
                    output.push_str(&String::from_utf8_lossy(&valid));
                    match e.error_len() {
                        Some(len) => {
                            self.0.drain(..len);
                            output.push(char::REPLACEMENT_CHARACTER);
                        }
                        None => break,
                    }
                }
            }
        }
        output
    }

    pub fn finish(&mut self) -> String {
        let rest = String::from_utf8_lossy(&self.0).into_owned();
        self.0.clear();
        rest
    }
}

#[derive(Debug)]
pub struct Quantizer {
    q: i128,
    error: i128,
}

impl Quantizer {
    pub fn new(q: u64) -> Self {
        Self {
            q: i128::from(q),
            error: 0,
        }
    }

    pub fn next(&mut self, value: u128) -> u128 {
        let value = i128::try_from(value).unwrap_or(i128::MAX);
        let corrected = value + self.error;
        let steps = (corrected + self.q / 2) / self.q;
        let quantized = steps * self.q;
        self.error = corrected - quantized;
        u128::try_from(quantized).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::{Quantizer, Utf8Decoder};

    #[test]
    fn utf8_decoder() {
        let mut decoder = Utf8Decoder::new();
        assert_eq!(decoder.feed(b"czarna "), "czarna ");
        assert_eq!(decoder.feed(&[0xc5, 0xbc, 0xc3]), "ż");
        assert_eq!(decoder.feed(&[0xb3, 0xc5, 0x82]), "ół");
        assert_eq!(decoder.feed(&[0xc4]), "");
        assert_eq!(decoder.feed(&[0x87, 0x21]), "ć!");
        assert_eq!(decoder.feed(&[0x80]), "�");
        assert_eq!(decoder.feed(&[]), "");
        assert_eq!(decoder.feed(&[0x80, 0x81]), "��");
        assert_eq!(decoder.feed(&[]), "");
        assert_eq!(decoder.feed(&[0x23]), "#");
        assert_eq!(
            decoder.feed(&[0x83, 0x23, 0xf0, 0x90, 0x80, 0xc0, 0x21]),
            "�#��!"
        );
    }

    #[test]
    fn quantizer() {
        let mut quantizer = Quantizer::new(1_000);
        let input: [u128; 24] = [
            26692, 540_290, 64736, 105_951, 171_006, 191_943, 107_942, 128_108, 148_904, 108_973,
            211_002, 44701, 489_307, 405_987, 105_028, 194_590, 61043, 532_296, 319_015, 152_786,
            32578, 5445, 40542, 756,
        ];
        let expected: [u128; 24] = [
            27000, 540_000, 65000, 106_000, 171_000, 192_000, 108_000, 128_000, 149_000, 109_000,
            211_000, 44000, 490_000, 406_000, 105_000, 194_000, 61000, 532_000, 320_000, 152_000,
            33000, 5000, 41000, 1000,
        ];
        let mut rounded = Vec::new();
        let mut input_sum: i128 = 0;
        let mut quantized_sum: i128 = 0;
        for value in input {
            let q = quantizer.next(value);
            rounded.push(q);
            input_sum += i128::try_from(value).unwrap();
            quantized_sum += i128::try_from(q).unwrap();
            assert!((input_sum - quantized_sum).abs() <= 500);
        }
        assert_eq!(rounded, expected);
    }
}
