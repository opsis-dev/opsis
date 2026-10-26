use opsis_media::ffmpeg::{ProbeResult, can_stream_copy, parse_probe};
use serde_json::json;

use crate::common::{cases, typed, with};

const PROBE: &str = "media/probe.json";

#[test]
fn probes_and_stream_copies_are_as_recorded() {
    let f = opsis_golden::read(PROBE);
    let parsed: Vec<ProbeResult> = f["probes"]
        .as_array()
        .expect("probe cases")
        .iter()
        .map(|c| parse_probe(c["raw"].clone()).expect("a probe"))
        .collect();
    let probes = cases(&f["probes"], |case| {
        let probe = parse_probe(case["raw"].clone()).expect("a probe");
        with(case, &[("probe", json!(probe))])
    });
    let stream_copy = cases(&f["streamCopy"], |case| {
        let (a, b): (usize, usize) = (typed(&case["a"]), typed(&case["b"]));
        with(
            case,
            &[("copy", json!(can_stream_copy(&parsed[a], &parsed[b])))],
        )
    });
    opsis_golden::check(
        PROBE,
        &json!({ "probes": probes, "streamCopy": stream_copy }),
    );
}
