use opsis_media::reverse::{
    PieceBudget, ReversePiece, ReversedPart, frame_bytes, join_reversed_args, piece_length_ms,
    reverse_args, reverse_pieces, reverse_sound_args, reversed_parts_list,
};
use serde_json::{Value, json};

use crate::common::{cases, text, typed, with};

const REVERSE: &str = "media/reverse.json";

fn plan(case: &Value) -> Value {
    let pcm = ["-c:a", "pcm_s16le"].map(str::to_owned);
    let budget: PieceBudget = typed(&case["plan"]);
    let pieces = reverse_pieces(budget);
    let lengths: Vec<f64> = pieces
        .iter()
        .map(|p| piece_length_ms(p, budget.duration_ms))
        .collect();
    let args: Vec<Vec<String>> = pieces
        .iter()
        .flat_map(|p| {
            [
                reverse_args("in.mp4", "out.mkv", p, true, &pcm),
                reverse_args("in.mp4", "out.mkv", p, false, &[]),
            ]
        })
        .collect();
    with(
        case,
        &[
            ("pieces", json!(pieces)),
            ("lengths", json!(lengths)),
            ("args", json!(args)),
        ],
    )
}

#[test]
fn reverse_plans_and_arguments_are_as_recorded() {
    let f = opsis_golden::read(REVERSE);
    let frame_bytes = cases(&f["frameBytes"], |case| {
        let bytes = frame_bytes(1921.0, 1081.0, text(&case["pixFmt"]));
        with(case, &[("bytes", json!(bytes))])
    });
    let whole: Vec<Vec<String>> = [true, false]
        .iter()
        .map(|s| reverse_args("in.mp4", "out.mp4", &ReversePiece::default(), *s, &[]))
        .collect();
    let mp3 = ["-vn", "-c:a", "libmp3lame", "-q:a", "2"].map(str::to_owned);
    let parts = [
        ReversedPart {
            file: "/tmp/part-0.mkv".into(),
            length_ms: 1234.5,
        },
        ReversedPart {
            file: "/tmp/it's part-1.mkv".into(),
            length_ms: 999.0,
        },
    ];
    let joins: Vec<Vec<String>> = [true, false]
        .iter()
        .map(|s| join_reversed_args("parts.txt", "out.mp4", *s))
        .collect();
    let made = json!({
        "frameBytes": frame_bytes,
        "plans": cases(&f["plans"], plan),
        "whole": whole,
        "sound": reverse_sound_args("in.mp3", "out.mp3", &mp3),
        "list": reversed_parts_list(&parts),
        "join": joins,
    });
    opsis_golden::check(REVERSE, &made);
}
