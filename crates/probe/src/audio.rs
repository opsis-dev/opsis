use opsis_types::numbers;

use crate::read::{Bytes, Short, Source};
use crate::{AudioStream, Found, ProbeError};

pub fn extended(bytes: &[u8]) -> Result<f64, Short> {
    let mut r = Bytes::new(bytes);
    let head = r.be16()?;
    let mantissa = r.be64()?;
    let exponent = i32::from(head & 0x7FFF) - 16383 - 63;
    let value = numbers::float_wide(mantissa) * 2f64.powi(exponent);
    Ok(if head & 0x8000 != 0 { -value } else { value })
}

pub fn aiff(source: &mut Source) -> Result<Found, ProbeError> {
    let head = source.bytes(8, 4)?;
    let compressed = head.as_slice() == b"AIFC";
    let mut at = 12u64;
    while at + 8 <= source.len() {
        let chunk = source.bytes(at, 8)?;
        let mut r = Bytes::new(&chunk);
        let id = r.take(4)?;
        let size = u64::from(r.be32()?);
        if id == b"COMM" {
            let body = source.bytes(at + 8, usize::try_from(size.min(64)).unwrap_or(0))?;
            let mut c = Bytes::new(&body);
            let channels = u32::from(c.be16()?);
            let frames = c.be32()?;
            let bits = c.be16()?;
            let rate = extended(c.take(10)?)?;
            let kind = if compressed {
                c.take(4).unwrap_or(b"NONE")
            } else {
                b"NONE".as_slice()
            };
            let codec = match kind {
                b"NONE" | b"twos" if bits == 8 => "pcm_s8".to_owned(),
                b"NONE" | b"twos" => format!("pcm_s{bits}be"),
                b"sowt" => format!("pcm_s{bits}le"),
                b"fl32" | b"FL32" => "pcm_f32be".to_owned(),
                b"fl64" | b"FL64" => "pcm_f64be".to_owned(),
                b"ulaw" | b"ULAW" => "pcm_mulaw".to_owned(),
                b"alaw" | b"ALAW" => "pcm_alaw".to_owned(),
                b"ima4" => "adpcm_ima_qt".to_owned(),
                other => String::from_utf8_lossy(other).trim().to_lowercase(),
            };
            return Ok(Found {
                format: "aiff",
                seconds: (rate > 0.0).then(|| f64::from(frames) / rate),
                video: None,
                audio: Some(AudioStream {
                    codec,
                    sample_rate: Some(numbers::truncate::<u32>(rate)).filter(|&r| r > 0),
                    channels: Some(channels).filter(|&c| c > 0),
                }),
            });
        }
        at += 8 + size + size % 2;
    }
    Err(ProbeError::Damaged)
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::extended;

    fn written(value: u32) -> [u8; 10] {
        let shift = value.leading_zeros();
        let exponent = 16383 + 31 - shift;
        let mantissa = u64::from(value) << (32 + shift);
        let mut out = [0u8; 10];
        out[..2].copy_from_slice(&u16::try_from(exponent).unwrap().to_be_bytes());
        out[2..].copy_from_slice(&mantissa.to_be_bytes());
        out
    }

    proptest! {
        #[test]
        fn reads_every_whole_sample_rate(rate in 1u32..1_000_000) {
            prop_assert_eq!(extended(&written(rate)).ok(), Some(f64::from(rate)));
        }
    }
}
