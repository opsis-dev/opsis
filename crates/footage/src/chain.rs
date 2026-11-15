use std::path::PathBuf;
use std::sync::Arc;

use opsis_compile::nodes::image::RedactImage;
use opsis_media::grade::GradeChain;
use opsis_types::numbers;

pub const SEEK_MARGIN_US: i64 = 1_000_000;
pub const MATCH_TOLERANCE_US: i64 = 500;
pub const JUMP_US: i64 = 2_000_000;
pub const SHUTTER_FRAMES: f64 = 0.5;
pub const FALLBACK_FRAME_US: i64 = 33_333;
const US_PER_S: f64 = 1_000_000.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelLayout {
    Rgb,
    Bgra,
}

impl PixelLayout {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Rgb => "rgb24",
            Self::Bgra => "bgra",
        }
    }

    pub const fn bytes(self) -> u64 {
        match self {
            Self::Rgb => 3,
            Self::Bgra => 4,
        }
    }
}

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
    pub trim_us: i64,
    pub loop_us: Option<i64>,
    pub frame_times: Option<Arc<[i64]>>,
    pub size: Option<(u32, u32)>,
    pub speed: f64,
    pub places: u32,
}

impl FootageSource {
    pub const fn plain(file: PathBuf, grade: GradeChain) -> Self {
        Self {
            file,
            grade,
            redactions: Vec::new(),
            trim_us: 0,
            loop_us: None,
            frame_times: None,
            size: None,
            speed: 1.0,
            places: 1,
        }
    }

    pub fn frame_us(&self) -> i64 {
        let Some(times) = self.frame_times.as_deref() else {
            return FALLBACK_FRAME_US;
        };
        let mut steps: Vec<i64> = times
            .windows(2)
            .map(|w| w[1] - w[0])
            .filter(|&d| d > 0)
            .collect();
        steps.sort_unstable();
        steps
            .get(steps.len() / 2)
            .copied()
            .unwrap_or(FALLBACK_FRAME_US)
    }
}

pub fn file_time(source: &FootageSource, time_us: i64) -> i64 {
    let played = time_us.max(0);
    let within = match source.loop_us {
        Some(period) if period > 0 => played % period,
        _ => played,
    };
    source.trim_us.max(0) + within
}

pub fn shown_index(times: &[i64], file_us: i64) -> usize {
    let target = file_us + MATCH_TOLERANCE_US;
    times.partition_point(|&t| t <= target).saturating_sub(1)
}

pub fn snapped_time(source: &FootageSource, time_us: i64) -> i64 {
    let Some(times) = source.frame_times.as_deref().filter(|t| !t.is_empty()) else {
        return time_us;
    };
    let played = time_us.max(0);
    let file = file_time(source, played);
    let shown = times[shown_index(times, file)].max(source.trim_us.max(0));
    let snapped = played - (file - shown);
    if snapped >= 0 && file_time(source, snapped) == shown {
        snapped
    } else {
        played
    }
}

pub fn window_span_us(ahead_frames: usize, film_fps: f64, speed: f64, frame_us: i64) -> i64 {
    let film_seconds = (numbers::float(ahead_frames) + SHUTTER_FRAMES) / film_fps.max(1.0);
    let source_us = (film_seconds * speed.abs() * US_PER_S).ceil();
    numbers::truncate::<i64>(source_us.min(1e15)) + 2 * frame_us.max(1)
}

pub fn window_frames(span_us: i64, frame_us: i64) -> u64 {
    u64::try_from(span_us.max(0) / frame_us.max(1)).unwrap_or(0) + 2
}

pub fn seconds_text(us: i64) -> String {
    let us = us.max(0);
    format!("{}.{:06}", us / 1_000_000, us % 1_000_000)
}

pub fn decoder_args(source: &FootageSource, start_us: i64, layout: PixelLayout) -> Vec<String> {
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
        "{};[out]format={},showinfo=checksum=0[frames]",
        source.grade.chain,
        layout.name()
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
            layout.name(),
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

pub fn frame_bytes(info: FrameInfo, layout: PixelLayout) -> Option<usize> {
    let pixels = u64::from(info.width).checked_mul(u64::from(info.height))?;
    usize::try_from(pixels.checked_mul(layout.bytes())?).ok()
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

    fn source_with(trim_us: i64, loop_us: Option<i64>) -> FootageSource {
        FootageSource {
            trim_us,
            loop_us,
            ..FootageSource::plain(
                PathBuf::from("/films/clip.mp4"),
                GradeChain {
                    chain: "[0:v]null[out]".to_owned(),
                    extra_inputs: Vec::new(),
                },
            )
        }
    }

    fn picked(times: &[i64], source: &FootageSource, time_us: i64) -> usize {
        shown_index(times, file_time(source, time_us))
    }

    fn frame_times(start: i64, gaps: &[i64]) -> Vec<i64> {
        gaps.iter()
            .scan(start, |at, gap| {
                let time = *at;
                *at += gap;
                Some(time)
            })
            .collect()
    }

    proptest! {
        #[test]
        fn a_snapped_time_shows_the_same_frame_at_or_before_the_time(
            start in 0_i64..200_000,
            gaps in proptest::collection::vec(1_001_i64..120_000, 1..80),
            trim in 0_i64..2_000_000,
            period in proptest::option::of(1_i64..3_000_000),
            at in -100_000_i64..8_000_000,
        ) {
            let times = frame_times(start, &gaps);
            let source = FootageSource {
                frame_times: Some(times.clone().into()),
                ..source_with(trim, period)
            };
            let snapped = snapped_time(&source, at);
            let index = picked(&times, &source, at);
            prop_assert_eq!(picked(&times, &source, snapped), index);
            prop_assert_eq!(snapped_time(&source, snapped), snapped);
            prop_assert!(index == 0 || times[index] <= file_time(&source, at) + MATCH_TOLERANCE_US);
            prop_assert!(times.get(index + 1).is_none_or(|&next| next > file_time(&source, at) + MATCH_TOLERANCE_US));
        }

        #[test]
        fn times_in_one_frame_snap_to_one_address(
            gaps in proptest::collection::vec(1_001_i64..120_000, 2..60),
            frame in 0usize..58,
            within in 0.0_f64..1.0,
        ) {
            let times = frame_times(0, &gaps);
            let frame = frame.min(times.len() - 2);
            let source = FootageSource {
                frame_times: Some(times.clone().into()),
                ..source_with(0, None)
            };
            let from = (times[frame] - MATCH_TOLERANCE_US).max(0);
            let until = times[frame + 1] - MATCH_TOLERANCE_US - 1;
            let inside = from + opsis_types::numbers::truncate::<i64>(within * opsis_types::numbers::float_wide(u64::try_from(until - from).unwrap()));
            prop_assert_eq!(snapped_time(&source, inside), snapped_time(&source, from));
        }

        #[test]
        fn the_window_holds_every_frame_the_drawn_frames_ahead_can_ask_for(
            ahead in 0usize..80,
            fps in 1_u32..120,
            speed in 0.05_f64..8.0,
            frame_us in 1_000_i64..100_000,
            oldest in 0_u32..10_000,
        ) {
            let film_fps = f64::from(fps);
            let span = window_span_us(ahead, film_fps, speed, frame_us);
            let source_at = |frame: f64| frame / film_fps * speed * US_PER_S;
            let earliest = source_at(f64::from(oldest) - SHUTTER_FRAMES);
            let latest = source_at(f64::from(oldest) + opsis_types::numbers::float(ahead));
            let shown_before = earliest - opsis_types::numbers::float_wide(u64::try_from(frame_us).unwrap());
            prop_assert!(opsis_types::numbers::float_wide(u64::try_from(span).unwrap()) >= latest - shown_before);
            let frames = window_frames(span, frame_us);
            prop_assert!(i128::from(frames) * i128::from(frame_us) >= i128::from(span));
            prop_assert!(window_span_us(ahead + 1, film_fps, speed, frame_us) >= span);
        }
    }

    proptest! {
        #[test]
        fn plays_from_the_trim_and_wraps_within_the_loop(trim in 0_i64..10_000_000, period in 1_i64..10_000_000, at in 0_i64..100_000_000) {
            let once = source_with(trim, None);
            prop_assert_eq!(file_time(&once, at), trim + at);
            let looped = source_with(trim, Some(period));
            let time = file_time(&looped, at);
            prop_assert!(time >= trim && time < trim + period);
            prop_assert_eq!(file_time(&looped, at + period), time);
            prop_assert_eq!(file_time(&looped, -at), trim);
        }
    }

    #[test]
    fn the_decoder_reads_from_the_start_or_seeks_before_its_frame() {
        let source = FootageSource::plain(
            PathBuf::from("/films/take.mp4"),
            GradeChain {
                chain: "[0:v]eq=contrast=1.1[g];[1:v][g]haldclut[out]".to_owned(),
                extra_inputs: vec!["/films/look.png".to_owned()],
            },
        );
        let tail = "-filter_complex [0:v]eq=contrast=1.1[g];[1:v][g]haldclut[out];[out]format=bgra,showinfo=checksum=0[frames] -map [frames] -an -f rawvideo -pix_fmt bgra pipe:1";
        assert_eq!(
            decoder_args(&source, 0, PixelLayout::Bgra).join(" "),
            format!(
                "-nostdin -hide_banner -nostats -loglevel info -copyts -start_at_zero -i /films/take.mp4 -i /films/look.png {tail}"
            )
        );
        assert_eq!(
            decoder_args(&source, 2_500_000, PixelLayout::Bgra).join(" "),
            format!(
                "-nostdin -hide_banner -nostats -loglevel info -copyts -start_at_zero -ss 2.500000 -i /films/take.mp4 -i /films/look.png {tail}"
            )
        );
    }
}
