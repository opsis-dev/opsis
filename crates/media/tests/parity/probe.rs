use opsis_media::ffmpeg::{ProbeResult, can_stream_copy, parse_probe};
use pretty_assertions::assert_eq;

use crate::common::{fixture, list, typed};

#[test]
fn probes_and_stream_copies_match() {
    let f = fixture("media/probe.json");
    let parsed: Vec<ProbeResult> = list(&f["probes"])
        .iter()
        .map(|c| parse_probe(c["raw"].clone()).unwrap())
        .collect();
    for (probe, case) in parsed.iter().zip(list(&f["probes"])) {
        let expected = serde_json::to_value(typed::<ProbeResult>(&case["probe"])).unwrap();
        assert_eq!(serde_json::to_value(probe).unwrap(), expected, "{case}");
    }
    for case in list(&f["streamCopy"]) {
        let (a, b): (usize, usize) = (typed(&case["a"]), typed(&case["b"]));
        assert_eq!(
            can_stream_copy(&parsed[a], &parsed[b]),
            case["copy"] == true,
            "{case}"
        );
    }
}
