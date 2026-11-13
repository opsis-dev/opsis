use kurbo::{BezPath, ParamCurve, ParamCurveArclen, PathSeg};

const ACCURACY: f64 = 1e-6;
const TANGENT_STEP: f64 = 1e-4;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PathSample {
    pub x: f64,
    pub y: f64,
    pub angle: f64,
}

fn segments(d: &str) -> Vec<PathSeg> {
    BezPath::from_svg(d).map_or_else(|_| Vec::new(), |path| path.segments().collect())
}

pub fn path_length(d: &str) -> f64 {
    segments(d).iter().map(|s| s.arclen(ACCURACY)).sum()
}

fn point_at(segments: &[PathSeg], at: f64) -> (f64, f64) {
    let mut left = at.max(0.0);
    for segment in segments {
        let length = segment.arclen(ACCURACY);
        if left <= length {
            let t = segment.inv_arclen(left, ACCURACY);
            let p = segment.eval(t);
            return (p.x, p.y);
        }
        left -= length;
    }
    segments.last().map_or((0.0, 0.0), |s| {
        let p = s.end();
        (p.x, p.y)
    })
}

pub fn point_at_length(d: &str, at: f64) -> (f64, f64) {
    point_at(&segments(d), at)
}

pub fn sample_path(d: &str, progress: f64) -> PathSample {
    let parts = segments(d);
    let length: f64 = parts.iter().map(|s| s.arclen(ACCURACY)).sum();
    let at = progress.clamp(0.0, 1.0) * length;
    let (x, y) = point_at(&parts, at);
    let (dx, dy) = if length > 0.0 {
        let step = (length * TANGENT_STEP).max(1e-6);
        let from = at.min(length - step);
        let here = point_at(&parts, from);
        let ahead = point_at(&parts, from + step);
        (ahead.0 - here.0, ahead.1 - here.1)
    } else {
        (1.0, 0.0)
    };
    PathSample {
        x,
        y,
        angle: dy.atan2(dx).to_degrees(),
    }
}

pub fn path_draw(progress: f64, d: &str) -> (f64, f64) {
    let shown = progress.clamp(0.0, 1.0);
    let length = path_length(d);
    if shown <= 0.0 {
        let dash = length * 1.5;
        return (dash, dash);
    }
    (length, shown.mul_add(-length, length))
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film places each control point with these exact steps, and the paths have to match it"
)]
pub fn waypoints_to_path(points: &[(f64, f64)], smoothing: f64) -> String {
    let n = opsis_scene::random::js_number_text;
    let Some(first) = points.first() else {
        return String::new();
    };
    let mut out = vec![format!("M {} {}", n(first.0), n(first.1))];
    for index in 0..points.len().saturating_sub(1) {
        let previous = points[index.saturating_sub(1)];
        let (from, to) = (points[index], points[index + 1]);
        let next = points.get(index + 2).copied().unwrap_or(to);
        if smoothing <= 0.0 {
            out.push(format!("L {} {}", n(to.0), n(to.1)));
            continue;
        }
        let c1 = (
            from.0 + (to.0 - previous.0) / 2.0 * smoothing,
            from.1 + (to.1 - previous.1) / 2.0 * smoothing,
        );
        let c2 = (
            to.0 - (next.0 - from.0) / 2.0 * smoothing,
            to.1 - (next.1 - from.1) / 2.0 * smoothing,
        );
        out.push(format!(
            "C {} {} {} {} {} {}",
            n(c1.0),
            n(c1.1),
            n(c2.0),
            n(c2.1),
            n(to.0),
            n(to.1)
        ));
    }
    out.join(" ")
}

pub fn fit_view_box(paths: &[&str], padding: f64) -> String {
    let boxes: Vec<kurbo::Rect> = paths
        .iter()
        .filter(|d| !d.is_empty())
        .filter_map(|d| BezPath::from_svg(d).ok())
        .map(|path| kurbo::Shape::bounding_box(&path))
        .collect();
    if boxes.is_empty() {
        return "0 0 100 100".to_owned();
    }
    let x1 = boxes.iter().map(|b| b.x0).fold(f64::INFINITY, f64::min);
    let y1 = boxes.iter().map(|b| b.y0).fold(f64::INFINITY, f64::min);
    let x2 = boxes.iter().map(|b| b.x1).fold(f64::NEG_INFINITY, f64::max);
    let y2 = boxes.iter().map(|b| b.y1).fold(f64::NEG_INFINITY, f64::max);
    let n = opsis_scene::random::js_number_text;
    format!(
        "{} {} {} {}",
        n(x1 - padding),
        n(y1 - padding),
        n(padding.mul_add(2.0, x2 - x1).max(1.0)),
        n(padding.mul_add(2.0, y2 - y1).max(1.0))
    )
}
