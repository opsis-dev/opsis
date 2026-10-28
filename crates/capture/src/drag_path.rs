use opsis_compile::time::{EPSILON, clamp};
use opsis_types::actions::{Point, Rect, Viewport};
use opsis_types::numbers;

const GLIDE_PX_PER_MS: f64 = 0.55;
const GLIDE_MIN_MS: f64 = 450.0;
const GLIDE_MAX_MS: f64 = 1350.0;
const OVERSHOOT_THRESHOLD_PX: f64 = 500.0;
const OVERSHOOT_PX: f64 = 26.0;
const BULGE: f64 = 0.1;
const DRAG_MIN_MS: f64 = 600.0;
const DRAG_MAX_MS: f64 = 2500.0;
const MAX_PATH_POINTS: usize = 64;

pub type Curve = Box<dyn Fn(f64) -> Point + Send + Sync>;

pub struct Glide {
    pub duration_ms: f64,
    pub aim: Point,
    pub overshooting: bool,
    pub at: Curve,
}

pub fn glide_plan(
    from: Point,
    to: Point,
    design_scale: f64,
    viewport: Option<Viewport>,
    travel_ms: Option<f64>,
) -> Glide {
    let dist = from.distance(to);
    let duration_ms = match travel_ms {
        Some(ms) if ms > 0.0 => ms,
        _ => clamp(
            dist / (GLIDE_PX_PER_MS * design_scale),
            GLIDE_MIN_MS,
            GLIDE_MAX_MS,
        ),
    };
    let overshooting = dist > OVERSHOOT_THRESHOLD_PX * design_scale;
    let past = |a: f64, b: f64, size: Option<f64>| {
        clamp(
            b + ((b - a) / dist) * OVERSHOOT_PX * design_scale,
            1.0,
            size.unwrap_or(b) - 1.0,
        )
    };
    let aim = if overshooting {
        Point {
            x: past(from.x, to.x, viewport.map(|v| f64::from(v.width))),
            y: past(from.y, to.y, viewport.map(|v| f64::from(v.height))),
        }
    } else {
        to
    };
    Glide {
        duration_ms,
        aim,
        overshooting,
        at: bulged_curve(from, to, aim),
    }
}

pub fn bulged_curve(from: Point, to: Point, aim: Point) -> Curve {
    let dist = from.distance(to);
    if dist < EPSILON {
        return Box::new(move |_| to);
    }
    let nx = -(to.y - from.y) / dist;
    let ny = (to.x - from.x) / dist;
    let side = if (to.x - from.x) * (to.y - from.y) >= 0.0 {
        1.0
    } else {
        -1.0
    };
    let bulge = side * dist * BULGE;
    let cpx = (from.x + aim.x) / 2.0 + nx * bulge;
    let cpy = (from.y + aim.y) / 2.0 + ny * bulge;
    Box::new(move |p| {
        let u = 1.0 - p;
        Point {
            x: u * u * from.x + 2.0 * u * p * cpx + p * p * aim.x,
            y: u * u * from.y + 2.0 * u * p * cpy + p * p * aim.y,
        }
    })
}

fn segment_lengths(points: &[Point]) -> Vec<f64> {
    points.windows(2).map(|w| w[0].distance(w[1])).collect()
}

pub fn polyline_at(points: Vec<Point>) -> Curve {
    let lengths = segment_lengths(&points);
    let total: f64 = lengths.iter().sum();
    Box::new(move |p| {
        let (first, last) = (points[0], points[points.len() - 1]);
        if total < EPSILON || p <= 0.0 {
            return first;
        }
        let mut left = p.min(1.0) * total;
        for (i, &seg) in lengths.iter().enumerate() {
            if left <= seg && seg > 0.0 {
                return points[i].lerp(points[i + 1], left / seg);
            }
            left -= seg;
        }
        last
    })
}

pub fn drag_duration(points: &[Point], design_scale: f64, drag_ms: f64) -> f64 {
    if drag_ms > 0.0 {
        return drag_ms;
    }
    clamp(
        segment_lengths(points).iter().sum::<f64>() / (GLIDE_PX_PER_MS * design_scale),
        DRAG_MIN_MS,
        DRAG_MAX_MS,
    )
}

pub fn nudge_toward(from: Point, to: Point, px: f64) -> Point {
    let dist = from.distance(to);
    if dist < EPSILON {
        return Point {
            x: from.x + px,
            y: from.y,
        };
    }
    let step = px.min(dist);
    Point {
        x: from.x + ((to.x - from.x) / dist) * step,
        y: from.y + ((to.y - from.y) / dist) * step,
    }
}

pub fn offset_path(start: Point, offsets: &[Point]) -> Vec<Point> {
    let points: Vec<Point> = offsets
        .iter()
        .map(|o| Point {
            x: start.x + o.x,
            y: start.y + o.y,
        })
        .collect();
    match points.first() {
        Some(first) if first.x == start.x && first.y == start.y => points,
        _ => std::iter::once(start).chain(points).collect(),
    }
}

pub fn thin_path(points: &[Point]) -> Vec<Point> {
    let rounded: Vec<Point> = points
        .iter()
        .map(|p| Point {
            x: p.x.round(),
            y: p.y.round(),
        })
        .collect();
    if rounded.len() <= MAX_PATH_POINTS {
        return rounded;
    }
    (0..MAX_PATH_POINTS)
        .map(|i| {
            rounded[numbers::truncate::<usize>(
                (numbers::float(i * (rounded.len() - 1)) / numbers::float(MAX_PATH_POINTS - 1))
                    .round(),
            )]
        })
        .collect()
}

pub fn bounds_of(points: &[Point]) -> Rect {
    let min_x = points.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
    let min_y = points.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
    let max_x = points.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max);
    let max_y = points.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max);
    Rect {
        x: min_x,
        y: min_y,
        w: max_x - min_x,
        h: max_y - min_y,
    }
}
