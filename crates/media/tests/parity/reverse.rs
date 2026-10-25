use opsis_media::reverse::{
    PieceBudget, ReversePiece, ReversedPart, frame_bytes, join_reversed_args, piece_length_ms,
    reverse_args, reverse_pieces, reverse_sound_args, reversed_parts_list,
};
use pretty_assertions::assert_eq;

use crate::common::{args, fixture, list, number, text, typed};

#[test]
fn reverse_plans_and_arguments_match() {
    let f = fixture("media/reverse.json");
    for case in list(&f["frameBytes"]) {
        assert_eq!(
            frame_bytes(1921.0, 1081.0, text(&case["pixFmt"])),
            number(&case["bytes"]),
            "{case}"
        );
    }
    let pcm = ["-c:a", "pcm_s16le"].map(str::to_owned);
    for case in list(&f["plans"]) {
        let plan: PieceBudget = typed(&case["plan"]);
        let pieces = reverse_pieces(plan);
        assert_eq!(
            pieces,
            typed::<Vec<ReversePiece>>(&case["pieces"]),
            "{case}"
        );
        let lengths: Vec<f64> = pieces
            .iter()
            .map(|p| piece_length_ms(p, plan.duration_ms))
            .collect();
        assert_eq!(lengths, typed::<Vec<f64>>(&case["lengths"]));
        let made: Vec<Vec<String>> = pieces
            .iter()
            .flat_map(|p| {
                [
                    reverse_args("in.mp4", "out.mkv", p, true, &pcm),
                    reverse_args("in.mp4", "out.mkv", p, false, &[]),
                ]
            })
            .collect();
        assert_eq!(made, typed::<Vec<Vec<String>>>(&case["args"]));
    }
    let whole: Vec<Vec<String>> = [true, false]
        .iter()
        .map(|s| reverse_args("in.mp4", "out.mp4", &ReversePiece::default(), *s, &[]))
        .collect();
    assert_eq!(whole, typed::<Vec<Vec<String>>>(&f["whole"]));
    let mp3 = ["-vn", "-c:a", "libmp3lame", "-q:a", "2"].map(str::to_owned);
    assert_eq!(
        reverse_sound_args("in.mp3", "out.mp3", &mp3),
        args(&f["sound"])
    );
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
    assert_eq!(reversed_parts_list(&parts), text(&f["list"]));
    let joins: Vec<Vec<String>> = [true, false]
        .iter()
        .map(|s| join_reversed_args("parts.txt", "out.mp4", *s))
        .collect();
    assert_eq!(joins, typed::<Vec<Vec<String>>>(&f["join"]));
}
