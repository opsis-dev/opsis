use std::collections::HashMap;
use std::hash::BuildHasher;

use indexmap::IndexMap;

pub const LANE_GAP: f64 = 14.0;
pub const STEP_OFFSET: f64 = 20.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LaneBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct LaneEdge<'a> {
    pub id: &'a str,
    pub source: &'a str,
    pub target: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LaneCentre {
    pub x: f64,
    pub left: f64,
    pub right: f64,
}

#[derive(Debug)]
struct Run<'a> {
    id: &'a str,
    left: f64,
    right: f64,
    top: f64,
    bottom: f64,
}

fn run_of<'a, S: BuildHasher>(
    edge: &LaneEdge<'a>,
    boxes: &HashMap<&str, LaneBox, S>,
) -> Option<(i64, Run<'a>)> {
    let from = boxes.get(edge.source)?;
    let to = boxes.get(edge.target)?;
    let left = from.x + from.width + STEP_OFFSET;
    let right = to.x - STEP_OFFSET;
    (right > left).then(|| {
        (
            opsis_types::numbers::truncate(to.x.round()),
            Run {
                id: edge.id,
                left,
                right,
                top: from.y.min(to.y),
                bottom: (from.y + from.height).max(to.y + to.height),
            },
        )
    })
}

fn lanes_of(runs: &mut [Run<'_>]) -> Vec<usize> {
    runs.sort_by(|a, b| {
        a.top
            .total_cmp(&b.top)
            .then(a.bottom.total_cmp(&b.bottom))
            .then(a.id.cmp(b.id))
    });
    let mut ends: Vec<f64> = Vec::new();
    runs.iter()
        .map(|run| {
            let lane = ends
                .iter()
                .position(|end| *end < run.top)
                .unwrap_or(ends.len());
            if lane == ends.len() {
                ends.push(run.bottom);
            } else {
                ends[lane] = run.bottom;
            }
            lane
        })
        .collect()
}

fn count(n: usize) -> f64 {
    f64::from(u32::try_from(n).unwrap_or(u32::MAX))
}

pub fn lane_centres<S: BuildHasher>(
    boxes: &HashMap<&str, LaneBox, S>,
    edges: &[LaneEdge<'_>],
) -> HashMap<String, LaneCentre> {
    let mut columns: IndexMap<i64, Vec<Run<'_>>> = IndexMap::new();
    for edge in edges {
        if let Some((column, run)) = run_of(edge, boxes) {
            columns.entry(column).or_default().push(run);
        }
    }
    let mut centres = HashMap::new();
    for runs in columns.values_mut() {
        let lanes = lanes_of(runs);
        let lane_count = count(lanes.iter().max().map_or(0, |m| m + 1));
        let left = runs.iter().map(|r| r.left).fold(f64::MIN, f64::max);
        let right = runs.iter().map(|r| r.right).fold(f64::MAX, f64::min);
        if right <= left {
            continue;
        }
        let spacing = LANE_GAP.min((right - left) / (lane_count + 1.0));
        let middle = f64::midpoint(left, right);
        for (run, lane) in runs.iter().zip(lanes) {
            let offset = count(lane) - (lane_count - 1.0) / 2.0;
            centres.insert(
                run.id.to_owned(),
                LaneCentre {
                    x: offset.mul_add(spacing, middle),
                    left: run.left,
                    right: run.right,
                },
            );
        }
    }
    centres
}

pub fn step_positions<S: BuildHasher>(
    boxes: &HashMap<&str, LaneBox, S>,
    edges: &[LaneEdge<'_>],
) -> HashMap<String, f64> {
    lane_centres(boxes, edges)
        .into_iter()
        .map(|(id, centre)| (id, (centre.x - centre.left) / (centre.right - centre.left)))
        .collect()
}
