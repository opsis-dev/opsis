use kurbo::{BezPath, CubicBez, ParamCurveArclen, Point, Shape};
use opsis_types::numbers::{exact_eq, float, truncate};

use crate::elements::transition::transition_morph_shape::MorphShape;

const ARC_ACCURACY: f64 = 1e-12;
const MIN_BOX: f64 = 1e-6;
const TARGET_GROWTH: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Segment {
    Move(Point),
    Cubic(Point, Point, Point),
}

impl Segment {
    pub const fn end(self) -> Point {
        match self {
            Self::Move(p) | Self::Cubic(_, _, p) => p,
        }
    }

    pub fn numbers(self) -> Vec<f64> {
        match self {
            Self::Move(p) => vec![p.x, p.y],
            Self::Cubic(a, b, c) => vec![a.x, a.y, b.x, b.y, c.x, c.y],
        }
    }

    fn mapped(self, map: impl Fn(Point) -> Point) -> Self {
        match self {
            Self::Move(p) => Self::Move(map(p)),
            Self::Cubic(a, b, c) => Self::Cubic(map(a), map(b), map(c)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MorphPair {
    pub from: Vec<Segment>,
    pub to: Vec<Segment>,
    pub closed: bool,
}

pub const fn outline(shape: MorphShape) -> &'static str {
    match shape {
        MorphShape::Circle => {
            "M 50 2 C 76.5 2 98 23.5 98 50 C 98 76.5 76.5 98 50 98 C 23.5 98 2 76.5 2 50 C 2 23.5 23.5 2 50 2 Z"
        }
        MorphShape::Square => {
            "M 2 2 C 34 2 66 2 98 2 C 98 34 98 66 98 98 C 66 98 34 98 2 98 C 2 66 2 34 2 2 Z"
        }
        MorphShape::Squircle => {
            "M 50 2 C 84 2 98 16 98 50 C 98 84 84 98 50 98 C 16 98 2 84 2 50 C 2 16 16 2 50 2 Z"
        }
        MorphShape::Triangle => {
            "M 50 4 C 66 32 82 60 96 92 C 65 92 35 92 4 92 C 18 60 34 32 50 4 Z"
        }
        MorphShape::Diamond => {
            "M 50 2 C 66 18 82 34 98 50 C 82 66 66 82 50 98 C 34 82 18 66 2 50 C 18 34 34 18 50 2 Z"
        }
        MorphShape::Blob => {
            "M 50 6 C 76 10 94 26 92 52 C 90 78 70 96 48 94 C 24 92 6 74 8 48 C 10 24 26 4 50 6 Z"
        }
    }
}

#[expect(
    clippy::many_single_char_names,
    reason = "the six numbers are a cubic's two control points and end, named as the film's path code names them"
)]
fn parsed(d: &str) -> (Vec<Segment>, bool) {
    let mut numbers = Vec::new();
    let mut segments = Vec::new();
    let mut command = ' ';
    let mut closed = false;
    for token in d.split_whitespace() {
        if let Ok(n) = token.parse::<f64>() {
            numbers.push(n);
        } else {
            command = token.chars().next().unwrap_or(' ');
            closed = command.eq_ignore_ascii_case(&'z');
        }
        match (command, numbers.as_slice()) {
            ('M', &[x, y]) => segments.push(Segment::Move(Point::new(x, y))),
            ('C', &[a, b, c, e, f, g]) => segments.push(Segment::Cubic(
                Point::new(a, b),
                Point::new(c, e),
                Point::new(f, g),
            )),
            _ => continue,
        }
        numbers.clear();
    }
    (segments, closed)
}

fn bez_path(segments: &[Segment]) -> BezPath {
    let mut path = BezPath::new();
    for segment in segments {
        match *segment {
            Segment::Move(p) => path.move_to(p),
            Segment::Cubic(a, b, c) => path.curve_to(a, b, c),
        }
    }
    path
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film fits each shape with separate multiplies and adds, and the fitted corners have to land on exactly 0 as its do"
)]
fn fitted(segments: &[Segment]) -> Vec<Segment> {
    let bounds = bez_path(segments).bounding_box();
    let (width, height) = (bounds.width().max(MIN_BOX), bounds.height().max(MIN_BOX));
    let scale = (1.0 / width).min(1.0 / height);
    let left = (1.0 - width * scale) / 2.0;
    let top = (1.0 - height * scale) / 2.0;
    let (tx, ty) = (left - bounds.x0 * scale, top - bounds.y0 * scale);
    segments
        .iter()
        .map(|s| s.mapped(|p| Point::new(p.x * scale + tx, p.y * scale + ty)))
        .collect()
}

fn along(a: Point, b: Point, t: f64) -> Point {
    Point::new((b.x - a.x).mul_add(t, a.x), (b.y - a.y).mul_add(t, a.y))
}

fn curved(segments: &[Segment], closed: bool) -> Vec<Segment> {
    let mut out = segments.to_vec();
    if closed && let (Some(first), Some(last)) = (segments.first(), segments.last()) {
        let (from, to) = (last.end(), first.end());
        out.push(Segment::Cubic(
            along(from, to, 1.0 / 3.0),
            along(from, to, 2.0 / 3.0),
            to,
        ));
    }
    out
}

fn lengths(path: &[Segment]) -> Vec<f64> {
    let mut at = path.first().map_or(Point::ORIGIN, |s| s.end());
    path.iter()
        .map(|segment| {
            let length = match *segment {
                Segment::Move(_) => 0.0,
                Segment::Cubic(a, b, c) => CubicBez::new(at, a, b, c).arclen(ARC_ACCURACY),
            };
            at = segment.end();
            length
        })
        .collect()
}

pub fn js_round(value: f64) -> f64 {
    (value + 0.5).floor()
}

fn path_splits(path: &[Segment], target: usize) -> Vec<usize> {
    let lengths = lengths(path);
    let total: f64 = lengths.iter().sum();
    if exact_eq(total, 0.0) {
        return vec![1; path.len()];
    }
    let ideal = total / float(target);
    let mut splits: Vec<usize> = lengths
        .iter()
        .enumerate()
        .map(|(i, &length)| {
            if i > 0 && length > ideal {
                truncate(js_round(length / ideal))
            } else {
                1
            }
        })
        .collect();
    let allocated: usize = splits.iter().sum();
    let mut candidates: Vec<(usize, f64)> = lengths
        .iter()
        .enumerate()
        .skip(1)
        .filter(|(_, l)| **l > 0.0)
        .map(|(i, l)| (i, *l))
        .collect();
    let shrinking = allocated > target;
    let mut remaining = allocated.abs_diff(target);
    if remaining == 0 || candidates.is_empty() {
        return splits;
    }
    if shrinking {
        candidates.sort_by(|a, b| a.1.total_cmp(&b.1));
    } else {
        candidates.sort_by(|a, b| b.1.total_cmp(&a.1));
    }
    let mut i = 0;
    while i < candidates.len() {
        let index = candidates[i].0;
        if !shrinking {
            splits[index] += 1;
            remaining -= 1;
        } else if splits[index] > 1 {
            splits[index] -= 1;
            remaining -= 1;
        }
        if remaining == 0 {
            break;
        }
        if i == candidates.len() - 1 {
            i = 0;
        }
        i += 1;
    }
    splits
}

#[expect(
    clippy::many_single_char_names,
    reason = "the six numbers are a cubic's two control points and end, named as the film's path code names them"
)]
fn split_cubic(points: [Point; 4], t: f64) -> ([Point; 4], [Point; 4]) {
    let [p0, p1, p2, p3] = points;
    let (a, b, c) = (along(p0, p1, t), along(p1, p2, t), along(p2, p3, t));
    let (d, e) = (along(a, b, t), along(b, c, t));
    let middle = along(d, e, t);
    ([p0, a, d, middle], [middle, e, c, p3])
}

fn split_to_count(points: [Point; 4], count: usize) -> Vec<[Point; 4]> {
    if count <= 1 {
        return vec![points];
    }
    let mut rest = points;
    (0..count)
        .map(|i| {
            let (first, second) = split_cubic(rest, 1.0 / float(count - i));
            rest = second;
            first
        })
        .collect()
}

fn split_path(path: &[Segment], target: usize) -> Vec<Segment> {
    if path.len() < 2 || target <= 1 {
        return path.to_vec();
    }
    let splits = path_splits(path, target);
    let mut at = path[0].end();
    let mut out = vec![path[0]];
    for (segment, count) in path.iter().zip(splits).skip(1) {
        if let Segment::Cubic(a, b, c) = *segment
            && count >= 1
        {
            out.extend(
                split_to_count([at, a, b, c], count)
                    .into_iter()
                    .map(|[_, p1, p2, p3]| Segment::Cubic(p1, p2, p3)),
            );
        }
        at = segment.end();
    }
    out
}

fn rotations(path: &[Segment]) -> Vec<Vec<Segment>> {
    let count = path.len();
    (0..count)
        .map(|start| {
            (0..count)
                .map(|i| {
                    let mut index = start + i;
                    if i == 0 || matches!(path.get(index), Some(Segment::Move(_))) {
                        return Segment::Move(path[index].end());
                    }
                    if index >= count {
                        index -= count - 1;
                    }
                    path[index]
                })
                .collect()
        })
        .collect()
}

fn rotated(path: &[Segment], reference: &[Segment]) -> Vec<Segment> {
    let mut best: Option<(f64, Vec<Segment>)> = None;
    for rotation in rotations(path) {
        let distance: f64 = rotation
            .iter()
            .zip(reference)
            .map(|(a, b)| (a.end() - b.end()).hypot2())
            .sum();
        if best.as_ref().is_none_or(|(least, _)| distance < *least) {
            best = Some((distance, rotation));
        }
    }
    best.map_or_else(|| path.to_vec(), |(_, rotation)| rotation)
}

fn total_length(path: &[Segment]) -> f64 {
    lengths(path).iter().sum()
}

pub fn prepare_morph(from: MorphShape, to: MorphShape) -> MorphPair {
    let (start, start_closed) = parsed(outline(from));
    let (end, end_closed) = parsed(outline(to));
    let first = curved(&fitted(&start), start_closed);
    let second = curved(&fitted(&end), end_closed);
    let least = first.len().max(second.len());
    let average = f64::midpoint(total_length(&first), total_length(&second));
    let per_segment = average / float(least);
    let ideal = least.max(truncate(js_round(average / per_segment.max(1.0))));
    let target = ideal.min(least * TARGET_GROWTH);
    let from = split_path(&first, target);
    let to = rotated(&split_path(&second, target), &from);
    MorphPair {
        from,
        to,
        closed: start_closed && end_closed,
    }
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film blends each number as a(1 - t) + bt, and the shapes have to match it"
)]
fn blended(a: f64, b: f64, t: f64) -> f64 {
    a * (1.0 - t) + b * t
}

#[expect(
    clippy::many_single_char_names,
    reason = "the six numbers are a cubic's two control points and end, named as the film's path code names them"
)]
pub fn morphed(progress: f64, pair: &MorphPair) -> Vec<Segment> {
    let t = progress.clamp(0.0, 1.0);
    let mix = |a: Point, b: Point| Point::new(blended(a.x, b.x, t), blended(a.y, b.y, t));
    pair.from
        .iter()
        .enumerate()
        .map(
            |(i, from)| match (*from, pair.to.get(i).copied().unwrap_or(*from)) {
                (Segment::Move(a), other) => Segment::Move(mix(a, other.end())),
                (Segment::Cubic(a, b, c), Segment::Cubic(x, y, z)) => {
                    Segment::Cubic(mix(a, x), mix(b, y), mix(c, z))
                }
                (cubic @ Segment::Cubic(..), Segment::Move(_)) => cubic,
            },
        )
        .collect()
}

pub fn path_data(segments: &[Segment], closed: bool, map: impl Fn(Point) -> Point) -> String {
    let mut out: Vec<String> = segments
        .iter()
        .map(|segment| {
            let mapped = segment.mapped(&map);
            let command = if matches!(segment, Segment::Move(_)) {
                "M"
            } else {
                "C"
            };
            let numbers: Vec<String> = mapped.numbers().iter().map(f64::to_string).collect();
            format!("{command}{}", numbers.join(" "))
        })
        .collect();
    if closed {
        out.push("Z".to_owned());
    }
    out.join(" ")
}
