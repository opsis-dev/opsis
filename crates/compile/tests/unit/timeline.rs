use indexmap::IndexMap;
use opsis_compile::time::frames_to_ms;
use opsis_compile::timeline::chapters::{
    FilmChapter, chapter_list, chapter_metadata, film_chapters,
};
use opsis_compile::timeline::flatten::node_spans;
use opsis_types::actions::ActionKind;
use proptest::prelude::*;
use serde_json::{Value, json};

use crate::helpers::{
    Graph, TITLE, action, export_film, film, film_spec, flat_film, make_log, near, record_result,
};

#[test]
fn maps_bars_through_speed_and_trims() {
    let wf = film(|g| {
        g.node("t", TITLE, json!({ "clipDuration": 120 }));
        g.node(
            "trim",
            "timeline.trim",
            json!({ "startMs": 1000, "durationMs": 0 }),
        );
        g.node("speed", "timeline.speed", json!({ "rate": 2 }));
        g.link("t", "trim").link("trim", "speed");
        "speed".into()
    });
    let bar = flat_film(&wf).bars.remove(0);
    assert!(
        near(bar.from, 0.0, 1e-3)
            && near(bar.to, 45.0, 0.5)
            && near(bar.rate, 2.0, 1e-3)
            && near(bar.content_start, 30.0, 0.5),
        "{bar:?}"
    );
    let chain: Vec<String> = bar.chain.iter().filter_map(|l| l.node_id.clone()).collect();
    assert_eq!(chain, ["t", "trim", "speed"]);
}

#[test]
fn stacks_overlapping_layers_on_lanes_in_drawing_order() {
    let wf = film(|g| {
        g.node("a", TITLE, json!({ "clipDuration": 90 }));
        g.node("b", TITLE, json!({ "clipDuration": 30 }));
        g.node("layers", "timeline.stack", json!({ "offsetsMs": "0,500" }));
        g.link_ports("a", "layers", "clip", "in1")
            .link_ports("b", "layers", "clip", "in2");
        "layers".into()
    });
    let f = flat_film(&wf);
    assert_eq!(f.lanes, 2);
    let b = f.bars.iter().find(|x| x.node_id == "b").unwrap();
    assert_eq!(b.lane, 1);
    assert!(near(b.from, 15.0, 0.5));
    let moved = b
        .chain
        .iter()
        .filter_map(|l| l.handles.as_ref()?.move_to.as_ref())
        .find(|m| m.node_id == "layers")
        .unwrap();
    assert_eq!((moved.param.as_str(), moved.index), ("offsetsMs", Some(1)));
}

#[test]
fn repeats_bars_for_every_loop_up_to_the_most_a_repeat_allows() {
    let looped = |times: u32, length: u32| {
        film(|g| {
            g.node("t", TITLE, json!({ "clipDuration": length }));
            g.node("loop", "timeline.loop", json!({ "times": times }));
            g.link("t", "loop");
            "loop".into()
        })
    };
    let spans: Vec<(f64, f64)> = flat_film(&looped(3, 30))
        .bars
        .iter()
        .map(|b| (b.from.round(), b.to.round()))
        .collect();
    assert_eq!(spans, [(0.0, 30.0), (30.0, 60.0), (60.0, 90.0)]);
    let bars = flat_film(&looped(100, 15)).bars;
    assert_eq!((bars.len(), bars.last().unwrap().to.round()), (100, 1500.0));
}

#[test]
fn spans_every_loop_and_speed_change_of_a_node() {
    let wf = film(|g| {
        g.node("t", TITLE, json!({ "clipDuration": 60 }));
        g.node("speed", "timeline.speed", json!({ "rate": 2 }));
        g.node("loop", "timeline.loop", json!({ "times": 3 }));
        g.link("t", "speed").link("speed", "loop");
        "loop".into()
    });
    let spans: IndexMap<String, (f64, f64)> = node_spans(&film_spec(&wf))
        .into_iter()
        .map(|(k, r)| (k, (r.from, r.to)))
        .collect();
    assert_eq!(
        spans,
        IndexMap::from([
            ("t".into(), (0.0, 90.0)),
            ("speed".into(), (0.0, 90.0)),
            ("loop".into(), (0.0, 90.0))
        ])
    );
}

#[test]
fn marks_transitions_between_sequenced_clips() {
    let wf = film(|g| {
        g.node("a", TITLE, json!({ "clipDuration": 60 }));
        g.node("b", TITLE, json!({ "clipDuration": 60 }));
        g.sequence(
            "seq",
            Some(json!({ "kind": "fade", "durationInFrames": 15 })),
        );
        g.link_ports("a", "seq", "clip", "in1")
            .link_ports("b", "seq", "clip", "in2");
        "seq".into()
    });
    let f = flat_film(&wf);
    assert_eq!(f.transitions.len(), 1);
    let t = &f.transitions[0];
    let slot = t.slot.as_ref().unwrap();
    assert_eq!(
        (
            t.from,
            t.to,
            t.kind.as_str(),
            slot.node_id.as_str(),
            slot.port.as_str()
        ),
        (45.0, 60.0, "fade", "seq", "in2")
    );
}

fn chapter(id: &str, t: f64, label: &str) -> Value {
    json!({ "id": id, "kind": "chapter", "t": t, "point": { "x": 0, "y": 0 }, "label": label })
}

fn chaptered_film(
    edit: Option<(&str, Value)>,
) -> (opsis_types::timeline::TimelineSpec, Vec<FilmChapter>) {
    let log = make_log(
        [
            chapter("a", 2000.0, "Sign in"),
            chapter("b", 20_000.0, "Build"),
            chapter("c", 25_000.0, "Too short"),
            chapter("d", 45_000.0, "Ship"),
        ]
        .into_iter()
        .map(action)
        .collect(),
        60_000.0,
    );
    let mut g = Graph::default();
    let mut last = g.recorded_take(&[]);
    if let Some((kind, params)) = edit {
        g.node("edit", kind, params);
        g.link(&last, "edit");
        last = "edit".into();
    }
    g.node("out", "output.render", json!({}));
    g.link(&last, "out");
    let results = IndexMap::from([("record".to_owned(), record_result(&log))]);
    let spec = export_film(&g.build(), Some(&results), "out");
    let chapters = film_chapters(&spec, "Intro");
    (spec, chapters)
}

fn titles(chapters: &[FilmChapter]) -> Vec<(String, f64)> {
    chapters
        .iter()
        .map(|c| (c.title.clone(), (c.start_ms / 100.0).round() / 10.0))
        .collect()
}

fn named(list: &[(&str, f64)]) -> Vec<(String, f64)> {
    list.iter().map(|(t, s)| ((*t).to_owned(), *s)).collect()
}

#[test]
fn chapters_start_at_zero_and_merge_ones_shorter_than_ten_seconds() {
    let (spec, chapters) = chaptered_film(None);
    assert_eq!(
        titles(&chapters),
        named(&[("Sign in", 0.0), ("Build", 20.0), ("Ship", 45.0)])
    );
    assert_eq!(
        chapters.last().unwrap().end_ms,
        frames_to_ms(spec.duration_in_frames, spec.fps)
    );
    assert_eq!(chapters[0].end_ms, chapters[1].start_ms);
}

#[test]
fn chapters_follow_delays_speed_changes_and_trims() {
    assert_eq!(
        titles(&chaptered_film(Some(("timeline.delay", json!({ "ms": 15_000 })))).1),
        named(&[
            ("Intro", 0.0),
            ("Sign in", 17.0),
            ("Build", 35.0),
            ("Ship", 60.0)
        ])
    );
    assert_eq!(
        titles(&chaptered_film(Some(("timeline.speed", json!({ "rate": 2 })))).1),
        named(&[("Sign in", 0.0), ("Build", 10.0)])
    );
    assert_eq!(
        titles(
            &chaptered_film(Some((
                "timeline.trim",
                json!({ "startMs": 10_000, "durationMs": 0 })
            )))
            .1
        ),
        named(&[("Intro", 0.0), ("Build", 10.0), ("Ship", 35.0)])
    );
}

#[test]
fn a_film_without_chapters_has_none() {
    let wf = film(|g| g.node("t", TITLE, json!({ "clipDuration": 90 })));
    assert_eq!(film_chapters(&export_film(&wf, None, "out"), "Intro"), []);
}

#[test]
fn writes_chapter_lists_and_escaped_metadata() {
    let chapters = [
        FilmChapter {
            title: "Sign in".into(),
            start_ms: 0.0,
            end_ms: 20_000.0,
        },
        FilmChapter {
            title: "Build = ship; #1".into(),
            start_ms: 20_000.0,
            end_ms: 3_725_000.0,
        },
    ];
    assert_eq!(
        chapter_list(&chapters),
        "0:00 Sign in\n0:20 Build = ship; #1\n"
    );
    assert_eq!(
        chapter_metadata(&chapters),
        [
            ";FFMETADATA1",
            "[CHAPTER]",
            "TIMEBASE=1/1000",
            "START=0",
            "END=20000",
            "title=Sign in",
            "[CHAPTER]",
            "TIMEBASE=1/1000",
            "START=20000",
            "END=3725000",
            r"title=Build \= ship\; \#1",
            ""
        ]
        .join("\n")
    );
}

#[test]
fn gives_each_bar_its_local_frame_at_every_timeline_frame() {
    let wf = film(|g| {
        g.node("t", TITLE, json!({ "clipDuration": 120 }));
        g.node(
            "trim",
            "timeline.trim",
            json!({ "startMs": 1000, "durationMs": 0 }),
        );
        g.node("speed", "timeline.speed", json!({ "rate": 2 }));
        g.link("t", "trim").link("trim", "speed");
        "speed".into()
    });
    let bar = flat_film(&wf).bars.remove(0);
    let frames = bar.local_frames();
    assert_eq!(frames.first, 0.0);
    for (i, local) in frames.values.iter().take(40).enumerate() {
        let expected = 2.0f64.mul_add(f64::from(u32::try_from(i).unwrap()), 30.0);
        assert!(
            near(*local, expected, 0.01),
            "frame {i}: {local} isn't {expected}"
        );
    }
}

proptest! {
    #[test]
    fn a_chapter_node_marks_where_its_clip_starts_in_the_film(
        first in 30u32..600,
        second in 30u32..600,
        title in "[A-Za-z ]{0,12}",
    ) {
        let wf = film(|g| {
            g.node("a", TITLE, json!({ "clipDuration": first }));
            g.node("b", TITLE, json!({ "clipDuration": second }));
            g.node("ch", "timeline.chapter", json!({ "title": title }));
            g.link("b", "ch");
            g.sequence("seq", None);
            g.link_ports("a", "seq", "clip", "in1")
                .link_ports("ch", "seq", "clip", "in2");
            "seq".into()
        });
        let chapters: Vec<(String, f64)> = flat_film(&wf)
            .marks
            .into_iter()
            .filter(|m| m.kind == ActionKind::Chapter)
            .map(|m| (m.label, m.frame))
            .collect();
        let named = match title.trim() {
            "" => "Chapter",
            trimmed => trimmed,
        };
        prop_assert_eq!(chapters, vec![(named.to_owned(), f64::from(first))]);
    }
}
