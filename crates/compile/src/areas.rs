use opsis_types::actions::Bounds;

pub const MIN_SIDE: f64 = 0.01;

fn round3(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
}

pub fn box_from(x0: f64, y0: f64, x1: f64, y1: f64) -> Bounds {
    let (x0, y0, x1, y1) = (
        x0.clamp(0.0, 1.0),
        y0.clamp(0.0, 1.0),
        x1.clamp(0.0, 1.0),
        y1.clamp(0.0, 1.0),
    );
    Bounds {
        x: round3(x0.min(x1)),
        y: round3(y0.min(y1)),
        width: round3((x1 - x0).abs()),
        height: round3((y1 - y0).abs()),
    }
}

pub fn moved_box(origin: Bounds, dx: f64, dy: f64) -> Bounds {
    Bounds {
        x: round3((origin.x + dx).clamp(0.0, (1.0 - origin.width).max(0.0))),
        y: round3((origin.y + dy).clamp(0.0, (1.0 - origin.height).max(0.0))),
        ..origin
    }
}

pub fn big_enough(area: Bounds) -> bool {
    area.width > MIN_SIDE && area.height > MIN_SIDE
}
