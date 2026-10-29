use std::io::Write;

use opsis_probe::probe;
use proptest::prelude::*;

const MAGIC: &[&[u8]] = &[
    b"\x00\x00\x00\x18ftypisom",
    b"\x00\x00\x00\x08moov",
    &[0x1A, 0x45, 0xDF, 0xA3],
    b"RIFF\x00\x00\x00\x00WAVE",
    b"RIFF\x00\x00\x00\x00AVI ",
    b"RIFF\x00\x00\x00\x00WEBP",
    b"FORM\x00\x00\x00\x00AIFF",
    b"caff\x00\x01\x00\x00",
    b"OggS",
    &[
        0x30, 0x26, 0xB2, 0x75, 0x8E, 0x66, 0xCF, 0x11, 0xA6, 0xD9, 0x00, 0xAA, 0x00, 0x62, 0xCE,
        0x6C,
    ],
    &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A],
    &[0xFF, 0xD8, 0xFF],
    b"GIF89a",
    b"<svg ",
    b"ID3\x03\x00\x00\x00\x00\x00\x10",
    b"fLaC",
    &[0xFF, 0xF1],
    &[0xFF, 0xFB],
];

fn written(bytes: &[u8]) -> tempfile::NamedTempFile {
    let mut file = tempfile::NamedTempFile::new().expect("a temp file");
    file.write_all(bytes).expect("written");
    file
}

fn wav(rate: u32, channels: u16, bits: u16, frames: u32) -> Vec<u8> {
    let block = channels * (bits / 8);
    let data = frames * u32::from(block);
    let mut out = Vec::new();
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&channels.to_le_bytes());
    out.extend_from_slice(&rate.to_le_bytes());
    out.extend_from_slice(&(rate * u32::from(block)).to_le_bytes());
    out.extend_from_slice(&block.to_le_bytes());
    out.extend_from_slice(&bits.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data.to_le_bytes());
    out.resize(out.len() + usize::try_from(data).unwrap(), 0);
    out
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(400))]

    #[test]
    fn answers_damaged_files_without_panicking(
        magic in prop::sample::select(MAGIC.to_vec()),
        rest in prop::collection::vec(any::<u8>(), 0..2048),
    ) {
        let mut bytes = magic.to_vec();
        bytes.extend(rest);
        let file = written(&bytes);
        let _ = probe(file.path());
    }

    #[test]
    fn reads_back_a_wave_file_as_written(
        rate in prop::sample::select(vec![8000u32, 22_050, 44_100, 48_000, 96_000]),
        channels in 1u16..=6,
        bits in prop::sample::select(vec![8u16, 16, 24, 32]),
        frames in 1u32..20_000,
    ) {
        let file = written(&wav(rate, channels, bits, frames));
        let found = probe(file.path()).expect("a probe");
        let audio = found.audio.expect("sound");
        prop_assert_eq!(found.format, "wav");
        prop_assert_eq!(audio.sample_rate, Some(rate));
        prop_assert_eq!(audio.channels, Some(u32::from(channels)));
        let codec = if bits == 8 { "pcm_u8".to_owned() } else { format!("pcm_s{bits}le") };
        prop_assert_eq!(audio.codec, codec);
        let ms = (f64::from(frames) * 1000.0 / f64::from(rate)).round();
        prop_assert_eq!(found.duration_ms.map(f64::from), (ms > 0.0).then_some(ms));
    }
}

#[test]
fn says_what_it_cannot_read() {
    let file = written(b"just some text that is no media file");
    assert!(matches!(
        probe(file.path()),
        Err(opsis_probe::ProbeError::Unknown)
    ));
}
