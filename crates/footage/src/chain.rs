use std::path::PathBuf;

use opsis_compile::nodes::image::RedactImage;
use opsis_media::grade::GradeChain;

pub const SEEK_MARGIN_US: i64 = 1_000_000;
pub const MATCH_TOLERANCE_US: i64 = 500;
pub const JUMP_US: i64 = 2_000_000;

#[derive(Debug, Clone, PartialEq)]
pub struct TimedRedaction {
    pub from_us: i64,
    pub until_us: Option<i64>,
    pub redact: RedactImage,
}

impl TimedRedaction {
    pub fn covers(&self, pts_us: i64) -> bool {
        pts_us >= self.from_us && self.until_us.is_none_or(|until| pts_us < until)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FootageSource {
    pub file: PathBuf,
    pub grade: GradeChain,
    pub redactions: Vec<TimedRedaction>,
}

pub fn seconds_text(us: i64) -> String {
    let us = us.max(0);
    format!("{}.{:06}", us / 1_000_000, us % 1_000_000)
}

pub fn decoder_args(source: &FootageSource, start_us: i64) -> Vec<String> {
    let mut args: Vec<String> = [
        "-nostdin",
        "-hide_banner",
        "-nostats",
        "-loglevel",
        "info",
        "-copyts",
        "-start_at_zero",
    ]
    .map(str::to_owned)
    .to_vec();
    if start_us > 0 {
        args.extend(["-ss".to_owned(), seconds_text(start_us)]);
    }
    args.extend(["-i".to_owned(), source.file.to_string_lossy().into_owned()]);
    for extra in &source.grade.extra_inputs {
        args.extend(["-i".to_owned(), extra.clone()]);
    }
    let graph = format!(
        "{};[out]format=rgb24,showinfo=checksum=0[frames]",
        source.grade.chain
    );
    args.extend(
        [
            "-filter_complex",
            graph.as_str(),
            "-map",
            "[frames]",
            "-an",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgb24",
            "pipe:1",
        ]
        .map(str::to_owned),
    );
    args
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeBase {
    pub num: i64,
    pub den: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameInfo {
    pub pts_us: i64,
    pub width: u32,
    pub height: u32,
}

fn field<'a>(line: &'a str, name: &str) -> Option<&'a str> {
    let at = line.find(name)? + name.len();
    line[at..].split_whitespace().next()
}

fn is_showinfo(line: &str) -> bool {
    line.contains("Parsed_showinfo")
}

pub fn parse_time_base(line: &str) -> Option<TimeBase> {
    if !is_showinfo(line) {
        return None;
    }
    let text = field(line, "config in time_base:")?.trim_end_matches(',');
    let (num, den) = text.split_once('/')?;
    let base = TimeBase {
        num: num.parse().ok()?,
        den: den.parse().ok()?,
    };
    (base.num > 0 && base.den > 0).then_some(base)
}

pub fn pts_to_us(pts: i64, base: TimeBase) -> Option<i64> {
    let scaled = i128::from(pts) * i128::from(base.num) * 1_000_000;
    let den = i128::from(base.den);
    let rounded = if scaled >= 0 {
        (scaled * 2 + den) / (den * 2)
    } else {
        (scaled * 2 - den) / (den * 2)
    };
    i64::try_from(rounded).ok()
}

pub fn parse_frame(line: &str, base: TimeBase) -> Option<FrameInfo> {
    if !is_showinfo(line) {
        return None;
    }
    let pts: i64 = field(line, " pts:")?.parse().ok()?;
    let (width, height) = field(line, " s:")?.split_once('x')?;
    Some(FrameInfo {
        pts_us: pts_to_us(pts, base)?,
        width: width.parse().ok()?,
        height: height.parse().ok()?,
    })
}

pub fn frame_bytes(info: FrameInfo) -> Option<usize> {
    let pixels = usize::try_from(u64::from(info.width) * u64::from(info.height)).ok()?;
    pixels.checked_mul(3)
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    const FRAME: &str = "[Parsed_showinfo_2 @ 0x7cff24011540] n:   1 pts:  18944 pts_time:1.233333 duration:    512 duration_time:0.0333333 fmt:rgb24 cl:left sar:1/1 s:320x240 i:P iskey:0 type:B ";
    const CONFIG: &str =
        "[Parsed_showinfo_2 @ 0x7cff24011540] config in time_base: 1/15360, frame_rate: 30/1";

    #[test]
    fn reads_the_time_base_and_each_frame_from_showinfo() {
        let base = parse_time_base(CONFIG).unwrap();
        assert_eq!(base, TimeBase { num: 1, den: 15360 });
        assert_eq!(
            parse_frame(FRAME, base),
            Some(FrameInfo {
                pts_us: 1_233_333,
                width: 320,
                height: 240
            })
        );
        assert_eq!(
            parse_frame(
                "[Parsed_showinfo_2 @ 0x1] color_range:pc color_space:gbr",
                base
            ),
            None
        );
        assert_eq!(parse_frame("[mp4 @ 0x1] pts: 3 s:2x2", base), None);
    }

    proptest! {
        #[test]
        fn a_frame_on_a_rate_grid_lands_on_its_microsecond(frame in 0_i64..10_000_000, rate in 1_i64..240) {
            let base = TimeBase { num: 1, den: rate * 512 };
            let us = pts_to_us(frame * 512, base).unwrap();
            let exact = i128::from(frame) * 1_000_000 * 2 / i128::from(rate);
            prop_assert!((i128::from(us) * 2 - exact).abs() <= 1);
        }

        #[test]
        fn seconds_text_reads_back_as_the_same_microseconds(us in 0_i64..100_000_000_000) {
            let text = seconds_text(us);
            let (whole, part) = text.split_once('.').unwrap();
            prop_assert_eq!(whole.parse::<i64>().unwrap() * 1_000_000 + part.parse::<i64>().unwrap(), us);
            prop_assert_eq!(part.len(), 6);
        }

        #[test]
        fn a_redaction_covers_exactly_its_time_range(from in 0_i64..1_000_000, length in proptest::option::of(1_i64..1_000_000), at in 0_i64..3_000_000) {
            let redaction = TimedRedaction { from_us: from, until_us: length.map(|l| from + l), redact: RedactImage::default() };
            let inside = at >= from && length.is_none_or(|l| at < from + l);
            prop_assert_eq!(redaction.covers(at), inside);
        }
    }

    #[test]
    fn the_decoder_reads_from_the_start_or_seeks_before_its_frame() {
        let source = FootageSource {
            file: PathBuf::from("/films/take.mp4"),
            grade: GradeChain {
                chain: "[0:v]eq=contrast=1.1[g];[1:v][g]haldclut[out]".to_owned(),
                extra_inputs: vec!["/films/look.png".to_owned()],
            },
            redactions: Vec::new(),
        };
        let tail = "-filter_complex [0:v]eq=contrast=1.1[g];[1:v][g]haldclut[out];[out]format=rgb24,showinfo=checksum=0[frames] -map [frames] -an -f rawvideo -pix_fmt rgb24 pipe:1";
        assert_eq!(
            decoder_args(&source, 0).join(" "),
            format!(
                "-nostdin -hide_banner -nostats -loglevel info -copyts -start_at_zero -i /films/take.mp4 -i /films/look.png {tail}"
            )
        );
        assert_eq!(
            decoder_args(&source, 2_500_000).join(" "),
            format!(
                "-nostdin -hide_banner -nostats -loglevel info -copyts -start_at_zero -ss 2.500000 -i /films/take.mp4 -i /films/look.png {tail}"
            )
        );
    }
}
