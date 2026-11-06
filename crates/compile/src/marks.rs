use serde_json::Value;

const MIN_DRAG: f64 = 0.01;
const KINDS: [&str; 5] = ["arrow", "rectangle", "oval", "step", "text"];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MarkPoints {
    pub x: f64,
    pub y: f64,
    pub x2: f64,
    pub y2: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

fn unit(value: f64) -> f64 {
    (value.clamp(0.0, 1.0) * 1000.0).round() / 1000.0
}

pub fn kind_of(mark: &Value) -> &'static str {
    mark.get("kind")
        .and_then(Value::as_str)
        .and_then(|kind| KINDS.into_iter().find(|k| *k == kind))
        .unwrap_or("arrow")
}

pub fn two_ended(kind: &str) -> bool {
    matches!(kind, "arrow" | "rectangle" | "oval")
}

pub fn points_of(mark: &Value) -> MarkPoints {
    let at = |key: &str| mark.get(key).and_then(Value::as_f64).unwrap_or(0.0);
    MarkPoints {
        x: at("x"),
        y: at("y"),
        x2: at("x2"),
        y2: at("y2"),
    }
}

pub fn moved(points: MarkPoints, dx: f64, dy: f64) -> MarkPoints {
    let left = points.x.min(points.x2);
    let right = points.x.max(points.x2);
    let top = points.y.min(points.y2);
    let bottom = points.y.max(points.y2);
    let sx = dx.clamp(-left, (1.0 - right).max(-left));
    let sy = dy.clamp(-top, (1.0 - bottom).max(-top));
    MarkPoints {
        x: unit(points.x + sx),
        y: unit(points.y + sy),
        x2: unit(points.x2 + sx),
        y2: unit(points.y2 + sy),
    }
}

pub fn with_end(points: MarkPoints, second: bool, at: Point) -> MarkPoints {
    if second {
        MarkPoints {
            x2: unit(at.x),
            y2: unit(at.y),
            ..points
        }
    } else {
        MarkPoints {
            x: unit(at.x),
            y: unit(at.y),
            ..points
        }
    }
}

pub fn drawn(kind: &str, start: Point, end: Point) -> Option<MarkPoints> {
    if !two_ended(kind) {
        return Some(MarkPoints {
            x: unit(end.x),
            y: unit(end.y),
            x2: unit(end.x),
            y2: unit(end.y),
        });
    }
    if (end.x - start.x).hypot(end.y - start.y) < MIN_DRAG {
        return None;
    }
    Some(MarkPoints {
        x: unit(start.x),
        y: unit(start.y),
        x2: unit(end.x),
        y2: unit(end.y),
    })
}

pub fn step_numbers(marks: &[Value], first: i64) -> Vec<Option<i64>> {
    let mut next = first;
    marks
        .iter()
        .map(|mark| {
            (kind_of(mark) == "step").then(|| {
                let number = next;
                next += 1;
                number
            })
        })
        .collect()
}
