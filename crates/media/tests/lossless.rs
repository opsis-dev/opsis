use std::path::Path;
use std::process::{Command, Stdio};

use opsis_compile::nodes::media::Codec;
use opsis_golden::ffmpeg;
use opsis_media::codecs::{FilmEncoding, film_encode_args, is_lossless, mux_args};
use proptest::prelude::*;

const LOSSLESS: [Codec; 2] = [Codec::H264Lossless, Codec::Ffv1];

fn run(args: &[&str], stdin: Option<&Path>) {
    let mut command = Command::new(ffmpeg());
    command.args(["-v", "error", "-y"]).args(args);
    if let Some(input) = stdin {
        command.stdin(Stdio::from(std::fs::File::open(input).unwrap()));
    }
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn text(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

const fn encoding(codec: Codec) -> FilmEncoding {
    FilmEncoding {
        codec,
        fps: 30.0,
        png_frames: is_lossless(codec),
        crf: None,
        x264_preset: None,
        pixel_format: None,
        prores: None,
        gif_loops: None,
    }
}

fn sound_codec(file: &Path) -> String {
    let output = Command::new(ffmpeg()).arg("-i").arg(file).output().unwrap();
    String::from_utf8_lossy(&output.stderr)
        .lines()
        .find_map(|line| line.split("Audio: ").nth(1))
        .and_then(|rest| rest.split([' ', ',']).next())
        .unwrap_or_default()
        .to_owned()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(4))]

    #[test]
    fn the_lossless_formats_give_back_every_pixel_and_carry_lossless_sound(
        half_width in 4u32..24,
        half_height in 4u32..24,
        frames in 6u32..10,
        seed in any::<u64>(),
    ) {
        let (width, height) = (half_width * 2, half_height * 2);
        let size = usize::try_from(width * height * 3 * frames).unwrap();
        let mut state = seed | 1;
        let pixels: Vec<u8> = (0..size)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                state.to_le_bytes()[0]
            })
            .collect();
        let dir = tempfile::tempdir().unwrap();
        let raw = dir.path().join("frames.rgb");
        std::fs::write(&raw, &pixels).unwrap();
        let pngs = dir.path().join("frames.png-stream");
        let dimensions = format!("{width}x{height}");
        run(
            &["-f", "rawvideo", "-pix_fmt", "rgb24", "-s", &dimensions, "-i", &text(&raw),
              "-c:v", "png", "-f", "image2pipe", &text(&pngs)],
            None,
        );
        let sound = dir.path().join("tone.wav");
        run(&["-f", "lavfi", "-i", "sine=frequency=440:duration=1", &text(&sound)], None);
        for codec in LOSSLESS {
            let film = dir.path().join(format!("film.{}", codec.extension()));
            let args = film_encode_args(&encoding(codec), &text(&film));
            run(&args.iter().map(String::as_str).collect::<Vec<_>>(), Some(&pngs));
            let decoded = dir.path().join("decoded.rgb");
            run(&["-i", &text(&film), "-f", "rawvideo", "-pix_fmt", "rgb24", &text(&decoded)], None);
            prop_assert!(std::fs::read(&decoded).unwrap() == pixels, "{codec:?} changed the picture");
            let muxed = dir.path().join(format!("muxed.{}", codec.extension()));
            let args = mux_args(codec, &text(&film), &text(&sound), &text(&muxed), None);
            run(&args.iter().map(String::as_str).collect::<Vec<_>>(), None);
            let wanted = if codec == Codec::Ffv1 { "flac" } else { "alac" };
            prop_assert_eq!(sound_codec(&muxed), wanted);
        }
    }
}
