use std::sync::LazyLock;

use opsis_compile::text::single_quoted;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::codecs::{edit_audio, edit_video, ffmpeg_seconds, with_even_frame};

pub const REVERSE_BUDGET_BYTES: f64 = 512.0 * 1024.0 * 1024.0;
const UNKNOWN_BYTES_PER_PIXEL: f64 = 4.0;

static BYTES_PER_PIXEL: LazyLock<Vec<(Regex, f64)>> = LazyLock::new(|| {
    [
        (r"^gray", 1.0),
        (r"^(?:yuvj?420p|nv12|nv21|p01[026])", 1.5),
        (r"^(?:yuvj?422p|nv16|yuyv422|uyvy422|p21[026])", 2.0),
        (r"^(?:rgb|bgr)48", 6.0),
        (r"^(?:rgb|bgr)a64", 8.0),
        (r"^(?:yuvj?444p|gbrp|rgb24|bgr24|p41[026])", 3.0),
        (r"^(?:rgba|bgra|argb|abgr|rgb0|bgr0|0rgb|0bgr)$", 4.0),
    ]
    .into_iter()
    .map(|(pattern, bytes)| (Regex::new(pattern).expect("pattern"), bytes))
    .collect()
});

static HIGH_DEPTH: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:p|gray)(?:9|10|12|14|16)(?:le|be)?$|^p[024]1[026]").expect("pattern")
});

pub fn frame_bytes(width: f64, height: f64, pix_fmt: &str) -> f64 {
    let per_pixel = BYTES_PER_PIXEL
        .iter()
        .find(|(pattern, _)| pattern.is_match(pix_fmt))
        .map_or(UNKNOWN_BYTES_PER_PIXEL, |(_, b)| *b);
    let depth = if HIGH_DEPTH.is_match(pix_fmt) {
        2.0
    } else {
        1.0
    };
    (width * height * per_pixel * depth).ceil()
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReverseCut {
    pub picture_ms: f64,
    pub sound_ms: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct ReversePiece {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<ReverseCut>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<ReverseCut>,
}

pub const WHOLE_FILE: ReversePiece = ReversePiece {
    from: None,
    to: None,
};

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PieceBudget {
    pub duration_ms: f64,
    pub fps: f64,
    pub frame_bytes: f64,
    pub budget_bytes: f64,
}

fn cut_before(frame: f64, fps: f64) -> ReverseCut {
    ReverseCut {
        picture_ms: ((frame - 0.5) * 1000.0 / fps).round(),
        sound_ms: (frame * 1000.0 / fps).round(),
    }
}

pub fn reverse_pieces(budget: PieceBudget) -> Vec<ReversePiece> {
    let per_piece = f64::max(1.0, (budget.budget_bytes / budget.frame_bytes).floor());
    let frames = (budget.duration_ms * budget.fps / 1000.0).ceil();
    let mut cuts: Vec<Option<ReverseCut>> = vec![None];
    let mut first = per_piece;
    while first < frames {
        cuts.push(Some(cut_before(first, budget.fps)));
        first += per_piece;
    }
    let mut pieces: Vec<ReversePiece> = cuts
        .iter()
        .enumerate()
        .map(|(n, from)| ReversePiece {
            from: *from,
            to: cuts.get(n + 1).copied().flatten(),
        })
        .collect();
    pieces.reverse();
    pieces
}

pub fn piece_length_ms(piece: &ReversePiece, duration_ms: f64) -> f64 {
    let end = piece.to.map_or(duration_ms, |t| t.sound_ms);
    let start = piece.from.map_or(0.0, |f| f.sound_ms);
    f64::max(0.0, end - start)
}

fn trim_filter(name: &str, start_ms: f64, end_ms: Option<f64>) -> Vec<String> {
    let mut bounds = Vec::new();
    if start_ms != 0.0 {
        bounds.push(format!("start={}", ffmpeg_seconds(start_ms)));
    }
    if let Some(end) = end_ms {
        bounds.push(format!("end={}", ffmpeg_seconds(end)));
    }
    if bounds.is_empty() {
        Vec::new()
    } else {
        vec![format!("{name}={}", bounds.join(":"))]
    }
}

fn picture_filter(piece: &ReversePiece) -> String {
    let seek = piece.from.map_or(0.0, |f| f.picture_ms);
    let mut steps = trim_filter("trim", 0.0, piece.to.map(|t| t.picture_ms - seek));
    steps.push("reverse".to_owned());
    if piece.from.is_some() {
        steps.push("setpts=PTS-STARTPTS".to_owned());
    }
    steps.join(",")
}

fn sound_filter(piece: &ReversePiece) -> String {
    let seek = piece.from.map_or(0.0, |f| f.picture_ms);
    let start = piece.from.map_or(0.0, |f| f.sound_ms - seek);
    let mut steps = trim_filter("atrim", start, piece.to.map(|t| t.sound_ms - seek));
    steps.push("areverse".to_owned());
    if piece.from.is_some() {
        steps.push("asetpts=PTS-STARTPTS".to_owned());
    }
    steps.join(",")
}

pub fn reverse_args(
    input: &str,
    output: &str,
    piece: &ReversePiece,
    sound: bool,
    sound_codec: &[String],
) -> Vec<String> {
    let mut args = Vec::new();
    if let Some(from) = piece.from {
        args.extend(["-ss".to_owned(), ffmpeg_seconds(from.picture_ms)]);
    }
    args.extend([
        "-i".to_owned(),
        input.to_owned(),
        "-vf".to_owned(),
        with_even_frame(&picture_filter(piece)),
    ]);
    if sound {
        args.extend(["-af".to_owned(), sound_filter(piece)]);
        args.extend(sound_codec.iter().cloned());
    } else {
        args.push("-an".to_owned());
    }
    args.extend(edit_video());
    args.push(output.to_owned());
    args
}

pub fn reverse_sound_args(input: &str, output: &str, codecs: &[String]) -> Vec<String> {
    let mut args = vec![
        "-i".to_owned(),
        input.to_owned(),
        "-af".to_owned(),
        "areverse".to_owned(),
    ];
    args.extend(codecs.iter().cloned());
    args.push(output.to_owned());
    args
}

#[derive(Debug)]
pub struct ReversedPart {
    pub file: String,
    pub length_ms: f64,
}

pub fn reversed_parts_list(parts: &[ReversedPart]) -> String {
    parts
        .iter()
        .map(|p| {
            format!(
                "file {}\nduration {}",
                single_quoted(&p.file),
                ffmpeg_seconds(p.length_ms)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn join_reversed_args(list: &str, output: &str, sound: bool) -> Vec<String> {
    let mut args: Vec<String> = [
        "-f", "concat", "-safe", "0", "-i", list, "-map", "0", "-c:v", "copy",
    ]
    .iter()
    .map(|&a| a.to_owned())
    .collect();
    if sound {
        args.extend(edit_audio());
    }
    args.push(output.to_owned());
    args
}
