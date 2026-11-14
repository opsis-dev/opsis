use std::path::Path;
use std::process::{Command, Stdio};

use opsis_compile::nodes::media::Codec;
use opsis_golden::ffmpeg;
use opsis_media::codecs::{FilmEncoding, film_encode_args};
use proptest::prelude::*;

const CODECS: [Codec; 4] = [Codec::H264, Codec::Vp9, Codec::H264Lossless, Codec::Ffv1];
const LOSSY_SLACK: i16 = 6;

fn run(args: &[String], stdin: Option<&Path>) {
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

const fn encoding(codec: Codec) -> FilmEncoding {
    FilmEncoding {
        codec,
        fps: 30.0,
        crf: Some(1.0),
        x264_preset: None,
        pixel_format: None,
        prores: None,
        gif_loops: None,
    }
}

fn text(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(4))]

    #[test]
    fn raw_frames_keep_their_colour_through_every_codec(red in any::<u8>(), green in any::<u8>(), blue in any::<u8>()) {
        let (width, height, frames) = (64_u32, 36_u32, 3_u32);
        let pixels: Vec<u8> = (0..width * height * frames)
            .flat_map(|_| [red, green, blue, 255])
            .collect();
        let dir = tempfile::tempdir().unwrap();
        let raw = dir.path().join("frames.rgba");
        std::fs::write(&raw, &pixels).unwrap();
        for codec in CODECS {
            let film = dir.path().join(format!("film.{}", codec.extension()));
            run(&film_encode_args(&encoding(codec), (width, height), &text(&film)), Some(&raw));
            let decoded = dir.path().join("decoded.rgb");
            run(
                &["-i".to_owned(), text(&film), "-frames:v".to_owned(), "1".to_owned(), "-f".to_owned(), "rawvideo".to_owned(), "-pix_fmt".to_owned(), "rgb24".to_owned(), text(&decoded)],
                None,
            );
            let back = std::fs::read(&decoded).unwrap();
            let centre = usize::try_from((height / 2 * width + width / 2) * 3).unwrap();
            let got = [back[centre], back[centre + 1], back[centre + 2]];
            let slack = if matches!(codec, Codec::H264Lossless | Codec::Ffv1) { 0 } else { LOSSY_SLACK };
            for (want, have) in [red, green, blue].iter().zip(got) {
                prop_assert!((i16::from(*want) - i16::from(have)).abs() <= slack, "{codec:?}: {:?} became {got:?}", [red, green, blue]);
            }
        }
    }
}
