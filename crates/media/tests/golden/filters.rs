use crate::common::{cases, json, number, text, typed, with};
use opsis_compile::nodes::media::{Dither, ScaleMode};
use opsis_media::filters::{
    JoinPart, JoinSize, atempo_chain, ffmpeg_color, fit_filter, gif_filter, join_filter,
    loudnorm_filter, speed_args, speed_sound_args,
};

const FILTERS: &str = "media/filters.json";

#[test]
fn filters_are_as_recorded() {
    let f = opsis_golden::read(FILTERS);
    let colours = cases(&f["colours"], |case| {
        with(case, &[("out", json(ffmpeg_color(text(&case["color"]))))])
    });
    let fit = cases(&f["fit"], |case| {
        let p = &case["p"];
        let mode: ScaleMode = typed(&p["mode"]);
        let out = fit_filter(
            mode,
            number(&p["width"]),
            number(&p["height"]),
            text(&p["background"]),
        );
        with(case, &[("out", json(out))])
    });
    let gif = cases(&f["gif"], |case| {
        let p = &case["p"];
        let dither: Dither = typed(&p["dither"]);
        let out = gif_filter(number(&p["fps"]), number(&p["width"]), dither);
        with(case, &[("out", json(out))])
    });
    let join = cases(&f["join"], |case| {
        let parts: Vec<JoinPart> = typed(&case["parts"]);
        let to: JoinSize = typed(&case["to"]);
        with(case, &[("out", json(join_filter(&parts, to)))])
    });
    let speed = cases(&f["speed"], |case| {
        let o = &case["opts"];
        let out = speed_args(
            "in.mp4",
            "out.mp4",
            number(&o["rate"]),
            number(&o["fps"]),
            o["sound"] == true,
        );
        with(case, &[("args", json(out))])
    });
    let speed_sound = cases(&f["speedSound"], |case| {
        let codecs = ["-vn", "-c:a", "aac"].map(str::to_owned);
        let out = speed_sound_args("in.mp3", "out.mp3", number(&case["rate"]), &codecs);
        with(case, &[("args", json(out))])
    });
    let atempo = cases(&f["atempo"], |case| {
        with(
            case,
            &[("chain", json(atempo_chain(number(&case["rate"]))))],
        )
    });
    let loudnorm = cases(&f["loudnorm"], |case| {
        let out = loudnorm_filter(
            number(&case["lufs"]),
            number(&case["truePeak"]),
            number(&case["range"]),
        );
        with(case, &[("filter", json(out))])
    });
    let made = serde_json::json!({
        "colours": colours,
        "fit": fit,
        "gif": gif,
        "join": join,
        "speed": speed,
        "speedSound": speed_sound,
        "atempo": atempo,
        "loudnorm": loudnorm,
    });
    opsis_golden::check(FILTERS, &made);
}
