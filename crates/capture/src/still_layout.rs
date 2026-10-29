pub use opsis_compile::nodes::capture::StillLook;
use opsis_compile::time::{clamp, even_px};
use serde::{Deserialize, Serialize};

const TALL_SHOT: f64 = 1.5;
const TALLEST_CUTOUT: f64 = 4.0;
const MAX_EXPORT_SCALE: f64 = 2.0;
const MAX_SCALE: f64 = 8.0;
const SCALE_STEPS: f64 = 2.0;
const HD_PIXELS: f64 = 1920.0 * 1080.0;
const MAX_HD_SURFACE_SCALE: f64 = 4.0;
const SURFACE_SCALE_STEPS: f64 = 10.0;
const SURFACE_BUDGET_PX: f64 = HD_PIXELS * MAX_HD_SURFACE_SCALE * MAX_HD_SURFACE_SCALE;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Area {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

struct ShotArea {
    width: f64,
    height: f64,
    body: Option<Area>,
}

fn aspect(width: f64, height: f64) -> f64 {
    width / height
}

pub fn contain_box(outer: Size, aspect_ratio: f64) -> Size {
    if outer.width / outer.height > aspect_ratio {
        Size {
            width: outer.height * aspect_ratio,
            height: outer.height,
        }
    } else {
        Size {
            width: outer.width,
            height: outer.width / aspect_ratio,
        }
    }
}

fn window_box(aspect_ratio: f64, canvas: Size, fit: f64) -> Size {
    let b = contain_box(
        Size {
            width: canvas.width * fit,
            height: canvas.height * fit,
        },
        aspect_ratio,
    );
    Size {
        width: b.width.round().max(1.0),
        height: b.height.round().max(1.0),
    }
}

fn window_width(shot: &ShotArea, canvas: Size, fit: f64) -> f64 {
    let tall = aspect(shot.width, shot.height) * TALL_SHOT < aspect(canvas.width, canvas.height);
    let ratio = if tall {
        aspect(canvas.width, canvas.height)
    } else {
        aspect(shot.width, shot.height)
    };
    window_box(ratio, canvas, fit).width
}

fn cutout_width(shot: &ShotArea, canvas: Size, fit: f64, max_upscale: f64) -> f64 {
    let body = shot.body.unwrap_or(Area {
        x: 0.0,
        y: 0.0,
        width: shot.width,
        height: shot.height,
    });
    let want = window_box(aspect(body.width, body.height), canvas, fit);
    let s = (want.width / body.width)
        .min(want.height / body.height)
        .min(max_upscale);
    shot.width * s
}

fn bleed_width(shot: &ShotArea, canvas: Size) -> f64 {
    let zoom = (canvas.width / shot.width).max(canvas.height / shot.height);
    shot.width * zoom
}

fn drawn_width(look: StillLook, shot: &ShotArea, canvas: Size, fit: f64, max_upscale: f64) -> f64 {
    match look {
        StillLook::Bleed => bleed_width(shot, canvas),
        StillLook::Cutout => cutout_width(shot, canvas, fit, max_upscale),
        StillLook::Window => window_width(shot, canvas, fit),
    }
}

fn max_scale_for(region: Size) -> f64 {
    let by_area = (SURFACE_BUDGET_PX / (region.width * region.height)).sqrt();
    clamp(
        (by_area * SURFACE_SCALE_STEPS).floor() / SURFACE_SCALE_STEPS,
        1.0,
        MAX_SCALE,
    )
}

pub fn tallest_shot(look: StillLook, width: f64, canvas: Size) -> f64 {
    let tallest = if look == StillLook::Cutout {
        width * TALLEST_CUTOUT
    } else {
        (width * TALL_SHOT) / aspect(canvas.width, canvas.height)
    };
    tallest.ceil()
}

pub fn padded_region(b: Area, scroll_x: f64, scroll_y: f64, padding: f64) -> (Area, Area) {
    let left = b.x + scroll_x;
    let top = b.y + scroll_y;
    let x = (left - padding).max(0.0);
    let y = (top - padding).max(0.0);
    let region = Area {
        x,
        y,
        width: left + b.width + padding - x,
        height: top + b.height + padding - y,
    };
    let body = Area {
        x: left - x,
        y: top - y,
        width: b.width,
        height: b.height,
    };
    (region, body)
}

pub fn capture_canvas(project: Size) -> Size {
    Size {
        width: project.width * MAX_EXPORT_SCALE,
        height: project.height * MAX_EXPORT_SCALE,
    }
}

pub fn capture_scale(
    look: StillLook,
    region: Size,
    canvas: Size,
    body: Option<Area>,
    fit: f64,
    least: f64,
) -> f64 {
    let area = ShotArea {
        width: region.width,
        height: region.height,
        body,
    };
    let drawn = drawn_width(look, &area, canvas, fit, f64::INFINITY) / region.width;
    let wanted = (drawn * SCALE_STEPS).ceil() / SCALE_STEPS;
    max_scale_for(region).min(least.max(wanted))
}

pub fn scaled_viewport(logical: Size, scale: f64) -> Size {
    Size {
        width: even_px(logical.width * scale),
        height: even_px(logical.height * scale),
    }
}
