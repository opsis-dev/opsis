use opsis_scene::color::Color;
use opsis_scene::hash::full_hash;
use opsis_scene::node::{Node, div};
use opsis_scene::random::js_number_text;
use opsis_scene::style::Style;
use opsis_scene::timeline::{At, DrawError, Drawn, Library, Presented, SceneTracks, draw_film};
use opsis_types::recording::RecordingProps;
use opsis_types::timeline::{
    GroupProps, ItemBase, SequencerProps, TimelineItem, TimelineSpec, TimelineTrack,
};
use proptest::prelude::*;
use serde_json::{Map, Value};

struct Labels;

fn label(component: &str, at: &At) -> Node {
    div(Style::default())
        .child(format!("{component}@{}", js_number_text(at.clock.frame)))
        .into()
}

impl Library for Labels {
    fn element(&self, component: &str, _: &Map<String, Value>, at: &At) -> Drawn {
        Ok(Some(label(component, at)))
    }

    fn wrapper(&self, component: &str, _: &Map<String, Value>, child: Node, at: &At) -> Drawn {
        Ok(Some(
            div(Style::default())
                .child(label(component, at))
                .child(child)
                .into(),
        ))
    }

    fn presentation(&self, presented: &Presented<'_>, child: Vec<Node>, _: &At) -> Drawn {
        Ok(Some(
            div(Style::default().opacity(presented.progress))
                .children(child)
                .into(),
        ))
    }

    fn scenes(&self, _: &SequencerProps, _: &dyn SceneTracks, _: &At) -> Drawn {
        Ok(None)
    }

    fn recording(&self, _: &RecordingProps, at: &At) -> Result<Node, DrawError> {
        Ok(label("recording", at))
    }

    fn color(&self, _: &str) -> Color {
        Color::BLACK
    }
}

fn element(id: &str, from: u32, length: u32) -> TimelineItem {
    TimelineItem::Element {
        base: ItemBase::new(id, from, length),
        component: id.to_owned(),
        props: Map::new(),
    }
}

fn spec(items: Vec<TimelineItem>, length: u32) -> TimelineSpec {
    TimelineSpec {
        width: 1920,
        height: 1080,
        fps: 30,
        duration_in_frames: length,
        background: None,
        tracks: vec![TimelineTrack {
            id: "t".to_owned(),
            items,
            layer: None,
        }],
        audio: Vec::new(),
        assets: None,
        reframe: None,
        fonts: None,
        mix: None,
    }
}

fn texts(node: &Node, out: &mut Vec<String>) {
    match node {
        Node::Text(text) => out.push(text.clone()),
        Node::Element(element) => element.children.iter().for_each(|c| texts(c, out)),
    }
}

fn shown(spec: &TimelineSpec, frame: f64) -> Vec<String> {
    let mut out = Vec::new();
    texts(&draw_film(&Labels, spec, frame).expect("a frame"), &mut out);
    out
}

fn grouped(
    remap: Option<Vec<(f64, f64)>>,
    loop_every: Option<u32>,
    start_from: Option<u32>,
) -> TimelineItem {
    TimelineItem::Group {
        base: ItemBase::new("g", 0, 300),
        props: Box::new(GroupProps {
            tracks: vec![TimelineTrack {
                id: "inner".to_owned(),
                items: vec![element("inside", 0, 1_000)],
                layer: None,
            }],
            remap,
            loop_every,
            start_from,
            ..GroupProps::default()
        }),
    }
}

proptest! {
    #[test]
    fn a_frame_drawn_twice_or_after_another_is_the_same(frame in 0u32..300, before in 0u32..300) {
        let film = spec(vec![element("a", 0, 100), element("b", 80, 120), element("c", 250, 50)], 300);
        let fresh = draw_film(&Labels, &film, f64::from(frame)).expect("a frame");
        let _ = draw_film(&Labels, &film, f64::from(before)).expect("a frame");
        let again = draw_film(&Labels, &film, f64::from(frame)).expect("a frame");
        prop_assert_eq!(full_hash(&fresh), full_hash(&again));
    }

    #[test]
    fn items_show_only_inside_their_range_on_their_own_clock(frame in 0u32..300, from in 0u32..200, length in 1u32..100) {
        let film = spec(vec![element("x", from, length)], 300);
        let expected = if frame >= from && frame < from + length {
            vec![format!("x@{}", frame - from)]
        } else {
            Vec::new()
        };
        prop_assert_eq!(shown(&film, f64::from(frame)), expected);
    }

    #[test]
    fn a_looping_group_repeats(frame in 0u32..300, every in 1u32..60) {
        let film = spec(vec![grouped(None, Some(every), None)], 300);
        let inner = shown(&film, f64::from(frame));
        prop_assert_eq!(inner.len(), 1);
        let seen: f64 = inner[0].trim_start_matches("inside@").parse().expect("a frame");
        prop_assert!(seen >= 0.0 && seen < f64::from(every));
    }

    #[test]
    fn a_start_offset_shifts_the_group(frame in 0u32..300, start in 1u32..200) {
        let film = spec(vec![grouped(None, None, Some(start))], 300);
        prop_assert_eq!(shown(&film, f64::from(frame)), vec![format!("inside@{}", frame + start)]);
    }

    #[test]
    fn a_remap_holds_whole_frames(frame in 0u32..300) {
        let film = spec(vec![grouped(Some(vec![(0.0, 0.0), (300.0, 150.0)]), None, None)], 300);
        prop_assert_eq!(shown(&film, f64::from(frame)), vec![format!("inside@{}", frame / 2)]);
    }
}
