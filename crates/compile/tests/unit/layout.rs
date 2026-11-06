use std::collections::HashMap;

use opsis_compile::layout::{CardShape, auto_layout};
use opsis_types::workflow::Workflow;
use proptest::prelude::*;
use serde_json::json;

const WIDTH: f64 = 256.0;

#[derive(Debug, Clone)]
struct Dag {
    heights: Vec<f64>,
    edges: Vec<(usize, usize)>,
}

fn dag() -> impl Strategy<Value = Dag> {
    (2usize..10)
        .prop_flat_map(|count| {
            (
                proptest::collection::vec(60.0f64..200.0, count),
                proptest::collection::vec((0..count, 0..count), 0..count * 2),
            )
        })
        .prop_map(|(heights, pairs)| {
            let mut edges: Vec<(usize, usize)> = pairs
                .into_iter()
                .filter(|(a, b)| a != b)
                .map(|(a, b)| (a.min(b), a.max(b)))
                .collect();
            edges.sort_unstable();
            edges.dedup();
            Dag { heights, edges }
        })
}

fn workflow(dag: &Dag) -> Workflow {
    let nodes: Vec<_> = (0..dag.heights.len())
        .map(|i| json!({"id": format!("n{i}"), "type": "test.node", "position": {"x": 0, "y": 0}, "params": {}}))
        .collect();
    let edges: Vec<_> = dag
        .edges
        .iter()
        .enumerate()
        .map(|(i, (a, b))| {
            json!({"id": format!("e{i}"), "source": format!("n{a}"), "sourceHandle": "out", "target": format!("n{b}"), "targetHandle": format!("in{i}")})
        })
        .collect();
    serde_json::from_value(
        json!({"version": 1, "id": "layout", "name": "Layout", "nodes": nodes, "edges": edges}),
    )
    .unwrap()
}

fn shapes(dag: &Dag) -> HashMap<String, CardShape> {
    dag.heights
        .iter()
        .enumerate()
        .map(|(i, height)| {
            (
                format!("n{i}"),
                CardShape {
                    width: WIDTH,
                    height: *height,
                },
            )
        })
        .collect()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    #[test]
    fn every_card_is_placed_left_to_right_without_overlap(dag in dag()) {
        let wf = workflow(&dag);
        let shapes = shapes(&dag);
        let placed = auto_layout(&wf, &shapes);
        prop_assert_eq!(placed.len(), dag.heights.len());
        for (a, b) in &dag.edges {
            let source = placed[&format!("n{a}")].x;
            let target = placed[&format!("n{b}")].x;
            prop_assert!(target >= source + WIDTH);
        }
        let boxes: Vec<(f64, f64, f64)> = (0..dag.heights.len())
            .map(|i| { let at = placed[&format!("n{i}")]; (at.x, at.y, dag.heights[i]) })
            .collect();
        for (i, a) in boxes.iter().enumerate() {
            for b in boxes.iter().skip(i + 1) {
                let apart = a.0 + WIDTH <= b.0 || b.0 + WIDTH <= a.0 || a.1 + a.2 <= b.1 || b.1 + b.2 <= a.1;
                prop_assert!(apart, "{:?} {:?}", a, b);
            }
        }
    }
}
