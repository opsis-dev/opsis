use std::collections::HashMap;

use opsis_compile::lanes::{LaneBox, LaneEdge, lane_centres, step_positions};
use proptest::prelude::*;

#[derive(Debug, Clone)]
struct Layout {
    sources: Vec<LaneBox>,
    target: LaneBox,
}

fn layout() -> impl Strategy<Value = Layout> {
    (
        proptest::collection::vec((-400.0f64..400.0, 40.0f64..200.0), 1..10),
        600.0f64..900.0,
        -200.0f64..200.0,
    )
        .prop_map(|(sources, target_x, target_y)| Layout {
            sources: sources
                .into_iter()
                .map(|(y, height)| LaneBox {
                    x: 0.0,
                    y,
                    width: 256.0,
                    height,
                })
                .collect(),
            target: LaneBox {
                x: target_x,
                y: target_y,
                width: 256.0,
                height: 120.0,
            },
        })
}

fn ids(n: usize) -> Vec<(String, String)> {
    (0..n).map(|i| (format!("s{i}"), format!("e{i}"))).collect()
}

fn boxes<'a>(layout: &Layout, names: &'a [(String, String)]) -> HashMap<&'a str, LaneBox> {
    let mut map: HashMap<&str, LaneBox> = names
        .iter()
        .zip(&layout.sources)
        .map(|((source, _), b)| (source.as_str(), *b))
        .collect();
    map.insert("target", layout.target);
    map
}

fn edges(names: &[(String, String)]) -> Vec<LaneEdge<'_>> {
    names
        .iter()
        .map(|(source, edge)| LaneEdge {
            id: edge,
            source,
            target: "target",
        })
        .collect()
}

fn overlap(a: &LaneBox, b: &LaneBox, target: &LaneBox) -> bool {
    let span = |s: &LaneBox| {
        (
            s.y.min(target.y),
            (s.y + s.height).max(target.y + target.height),
        )
    };
    let (a_top, a_bottom) = span(a);
    let (b_top, b_bottom) = span(b);
    a_top <= b_bottom && b_top <= a_bottom
}

proptest! {
    #[test]
    fn every_step_falls_inside_its_gap(layout in layout()) {
        let names = ids(layout.sources.len());
        let boxes = boxes(&layout, &names);
        for step in step_positions(&boxes, &edges(&names)).values() {
            prop_assert!(*step > 0.0 && *step < 1.0, "{step}");
        }
    }

    #[test]
    fn edges_whose_spans_overlap_never_share_a_lane(layout in layout()) {
        let names = ids(layout.sources.len());
        let boxes = boxes(&layout, &names);
        let centres = lane_centres(&boxes, &edges(&names));
        for (i, (_, a)) in names.iter().enumerate() {
            for (j, (_, b)) in names.iter().enumerate().skip(i + 1) {
                if overlap(&layout.sources[i], &layout.sources[j], &layout.target) {
                    prop_assert!((centres[a].x - centres[b].x).abs() > 1e-9);
                }
            }
        }
    }

    #[test]
    fn a_target_behind_its_source_gets_no_lane(layout in layout()) {
        let names = ids(layout.sources.len());
        let mut boxes = boxes(&layout, &names);
        boxes.insert("target", LaneBox { x: -600.0, ..layout.target });
        prop_assert!(step_positions(&boxes, &edges(&names)).is_empty());
    }
}
