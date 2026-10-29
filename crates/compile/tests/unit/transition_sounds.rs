use opsis_compile::compile::compile_workflow;
use opsis_compile::node::CardInputs;
use opsis_compile::nodes::audio::{transition_cues, transition_sound_file};
use opsis_types::values::{ClipValue, CompileValue, Hits};
use opsis_types::workflow::{Workflow, WorkflowSettings};
use serde_json::{Value, json};

use crate::helpers::{TITLE, card_texts, compile_node_with, film, node_result};

fn sequenced(after: Option<(&'static str, Value)>) -> Workflow {
    film(|g| {
        g.node("a", TITLE, json!({ "clipDuration": 60 }));
        g.node("b", TITLE, json!({ "clipDuration": 60 }));
        g.node(
            "seq",
            "timeline.sequence",
            json!({ "transition": "fade", "transitionDuration": 15 }),
        );
        g.link_ports("a", "seq", "clip", "in1")
            .link_ports("b", "seq", "clip", "in2");
        match after {
            Some((kind, params)) => {
                g.node(
                    "trim",
                    "timeline.trim",
                    json!({ "startMs": 500, "durationMs": 0 }),
                );
                g.node("after", kind, params);
                g.link("seq", "trim").link("trim", "after");
                "after".into()
            }
            None => "seq".into(),
        }
    })
}

fn clip_of(wf: &Workflow, id: &str) -> ClipValue {
    match compile_workflow(wf, None).values[id]["clip"].clone() {
        CompileValue::Clip(clip) => clip,
        other => panic!("{other:?}"),
    }
}

#[test]
fn finds_each_transitions_middle_through_a_trim_and_a_speed_change() {
    let settings = WorkflowSettings::default();
    let plain = transition_cues(&clip_of(&sequenced(None), "seq"), &settings, false);
    assert_eq!(plain.len(), 1);
    assert_eq!((plain[0].middle, plain[0].frames), (52.5, 15.0));
    let faster = sequenced(Some(("timeline.speed", json!({ "rate": 2 }))));
    let cues = transition_cues(&clip_of(&faster, "after"), &settings, false);
    assert_eq!(cues.len(), 1);
    assert_eq!((cues[0].middle, cues[0].frames), (18.75, 7.5));
}

#[test]
fn centres_each_sound_on_its_transition_and_counts_them() {
    let clip = clip_of(&sequenced(None), "seq");
    let inputs = json!({ "clip": CompileValue::Clip(clip) });
    let file = transition_sound_file(500);
    let made = node_result(json!({
        "outputs": {},
        "artifacts": [{ "kind": "audio", "path": format!(".opsis/cache/h/{file}") }]
    }));
    let audio = |params: Value, result| match compile_node_with(
        "audio.transitionSounds",
        params,
        inputs.clone(),
        result,
    )
    .unwrap()
    .shift_remove("audio")
    {
        Some(CompileValue::Audio(audio)) => audio,
        other => panic!("{other:?}"),
    };
    let generated = audio(json!({ "vary": false }), Some(&made));
    assert_eq!(
        generated.hits,
        Some(Hits {
            clicks: 0,
            keys: 0,
            transitions: 1
        })
    );
    let item = &generated.items[0];
    assert_eq!(item.src, format!("project:.opsis/cache/h/{file}"));
    assert_eq!(
        (item.from, item.length_frames, item.hit),
        (45, Some(15), Some(true))
    );
    assert!(
        audio(json!({}), None).items.is_empty(),
        "nothing to play before the run"
    );
    let from_file = audio(json!({ "soundFrom": "file" }), None);
    assert_eq!(from_file.items[0].src, "app:sfx/whoosh.wav");
    let placed = |align: &str| {
        let item = &audio(json!({ "soundFrom": "file", "align": align }), None).items[0];
        (item.from, f64::from(item.length_frames.unwrap()))
    };
    assert_eq!(placed("start").0, 45, "from the transition's start");
    let (end_from, length) = placed("end");
    assert_eq!(
        f64::from(end_from),
        (60.0 - length).round(),
        "ending with it"
    );
    assert_eq!(
        card_texts("audio.transitionSounds", json!({}), CardInputs::default()),
        ["Connect a clip with transitions"]
    );
}

#[test]
fn adds_plain_cuts_when_asked_with_the_variations_own_length() {
    use opsis_compile::nodes::audio::TransitionSounds;
    let wf = film(|g| {
        for id in ["a", "b", "c"] {
            g.node(id, TITLE, json!({ "clipDuration": 60 }));
        }
        g.node(
            "seq",
            "timeline.sequence",
            json!({
                "transition": "fade", "transitionDuration": 15,
                "cuts": [{ "before": "in3", "transition": "none" }]
            }),
        );
        g.link_ports("a", "seq", "clip", "in1")
            .link_ports("b", "seq", "clip", "in2")
            .link_ports("c", "seq", "clip", "in3");
        "seq".into()
    });
    let clip = clip_of(&wf, "seq");
    let settings = WorkflowSettings::default();
    let only = transition_cues(&clip, &settings, false);
    assert_eq!(
        only.iter()
            .map(|c| (c.middle, c.frames))
            .collect::<Vec<_>>(),
        [(52.5, 15.0)]
    );
    let with_cuts = transition_cues(&clip, &settings, true);
    assert_eq!(
        with_cuts
            .iter()
            .map(|c| (c.middle, c.frames))
            .collect::<Vec<_>>(),
        [(52.5, 15.0), (105.0, 0.0)]
    );
    let p = TransitionSounds {
        at_cuts: true,
        ..TransitionSounds::default()
    };
    assert_eq!(
        (
            p.length_ms(&with_cuts[0], 30),
            p.length_ms(&with_cuts[1], 30)
        ),
        (500, 800)
    );
}

#[test]
fn fades_sounds_still_playing_where_the_clip_ends_only_when_asked() {
    let under = |end_fade_ms: u32| {
        let wf = film(|g| {
            g.node("a", TITLE, json!({ "clipDuration": 60 }));
            g.node(
                "hit",
                "audio.sfx",
                json!({ "src": "app:sfx/whoosh.wav", "atMs": 1900 }),
            );
            g.node(
                "early",
                "audio.sfx",
                json!({ "src": "app:sfx/whoosh.wav", "atMs": 0 }),
            );
            g.node("mix", "audio.mix", json!({ "endFadeMs": end_fade_ms }));
            g.link("a", "mix")
                .link_ports("hit", "mix", "audio", "sounds")
                .link_ports("early", "mix", "audio", "sounds");
            "mix".into()
        });
        let clip = clip_of(&wf, "mix");
        let fade = |id: &str| {
            clip.audio
                .iter()
                .find(|a| a.node_id.as_deref() == Some(id))
                .map(|a| (a.fade_out_frames, a.duration_in_frames))
                .unwrap()
        };
        (fade("hit"), fade("early"))
    };
    let ((hard, _), _) = under(0);
    assert_eq!(hard, None);
    let ((faded, length), (early, _)) = under(1000);
    assert_eq!(
        faded, length,
        "a cut shorter than the fade fades all the way"
    );
    assert_eq!(
        early, None,
        "a sound that ends inside the clip is left alone"
    );
}
