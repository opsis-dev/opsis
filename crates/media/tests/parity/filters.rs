use crate::common::{args, fixture, list, number, text, typed};
use opsis_compile::nodes::media::{Dither, ScaleMode};
use opsis_media::filters::{
    Join, JoinPart, JoinSize, atempo_chain, ffmpeg_color, fit_filter, gif_filter, join_filter,
    loudnorm_filter, speed_args, speed_sound_args,
};
use pretty_assertions::assert_eq;

#[test]
fn filters_match() {
    let f = fixture("media/filters.json");
    for case in list(&f["colours"]) {
        assert_eq!(
            ffmpeg_color(text(&case["color"])),
            text(&case["out"]),
            "{case}"
        );
    }
    for case in list(&f["fit"]) {
        let p = &case["p"];
        let mode: ScaleMode = typed(&p["mode"]);
        let out = fit_filter(
            mode,
            number(&p["width"]),
            number(&p["height"]),
            text(&p["background"]),
        );
        assert_eq!(out, text(&case["out"]), "{case}");
    }
    for case in list(&f["gif"]) {
        let p = &case["p"];
        let dither: Dither = typed(&p["dither"]);
        assert_eq!(
            gif_filter(number(&p["fps"]), number(&p["width"]), dither),
            text(&case["out"]),
            "{case}"
        );
    }
    for case in list(&f["join"]) {
        let parts: Vec<JoinPart> = typed(&case["parts"]);
        let to: JoinSize = typed(&case["to"]);
        assert_eq!(
            join_filter(&parts, to),
            typed::<Join>(&case["out"]),
            "{case}"
        );
    }
    for case in list(&f["speed"]) {
        let o = &case["opts"];
        let out = speed_args(
            "in.mp4",
            "out.mp4",
            number(&o["rate"]),
            number(&o["fps"]),
            o["sound"] == true,
        );
        assert_eq!(out, args(&case["args"]), "{case}");
    }
    for case in list(&f["speedSound"]) {
        let codecs = ["-vn", "-c:a", "aac"].map(str::to_owned);
        assert_eq!(
            speed_sound_args("in.mp3", "out.mp3", number(&case["rate"]), &codecs),
            args(&case["args"]),
            "{case}"
        );
    }
    for case in list(&f["atempo"]) {
        assert_eq!(
            atempo_chain(number(&case["rate"])),
            text(&case["chain"]),
            "{case}"
        );
    }
    for case in list(&f["loudnorm"]) {
        let out = loudnorm_filter(
            number(&case["lufs"]),
            number(&case["truePeak"]),
            number(&case["range"]),
        );
        assert_eq!(out, text(&case["filter"]), "{case}");
    }
}
