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
    (length, length - shown * length)
}
