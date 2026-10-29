use std::collections::HashMap;

use ab_glyph::FontVec;
use image::{DynamicImage, Rgba, Rgba32FImage};
use opsis_compile::nodes::image::{Mark, MarkKind};

use opsis_types::numbers;
use opsis_types::numbers::whole;

use crate::ops::{background_rgba, is_deep, over};
use crate::text::{draw_text, measure};

pub type MarkFonts = HashMap<(String, u32), FontVec>;

const HEAD_LENGTH: f64 = 4.5;
const HEAD_WIDTH: f64 = 3.2;
const SMALLEST_HEAD: f64 = 12.0;
const STEP_TEXT: f64 = 0.6;
const TEXT_PAD: f64 = 0.3;
const TEXT_RADIUS: f64 = 0.25;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

impl Bounds {
    fn around(points: &[(f64, f64)], margin: f64) -> Self {
        let xs = points.iter().map(|p| p.0);
        let ys = points.iter().map(|p| p.1);
        Self {
            left: xs.clone().fold(f64::INFINITY, f64::min) - margin,
            top: ys.clone().fold(f64::INFINITY, f64::min) - margin,
            right: xs.fold(f64::NEG_INFINITY, f64::max) + margin,
            bottom: ys.fold(f64::NEG_INFINITY, f64::max) + margin,
        }
    }
}

fn unit_colour(css: &str) -> [f32; 4] {
    background_rgba(css).map(|v| f32::from(v) / 255.0)
}

fn coverage(distance: f64) -> f32 {
    numbers::narrow((0.5 - distance).clamp(0.0, 1.0))
}

fn segment_distance(point: (f64, f64), from: (f64, f64), to: (f64, f64)) -> f64 {
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let length = dx * dx + dy * dy;
    let t = if length > 0.0 {
        (((point.0 - from.0) * dx + (point.1 - from.1) * dy) / length).clamp(0.0, 1.0)
    } else {
        0.0
    };
    (point.0 - from.0 - t * dx).hypot(point.1 - from.1 - t * dy)
}

fn triangle_distance(point: (f64, f64), corners: [(f64, f64); 3]) -> f64 {
    let edges = [
        (corners[0], corners[1]),
        (corners[1], corners[2]),
        (corners[2], corners[0]),
    ];
    let to_edge = edges
        .iter()
        .map(|(a, b)| segment_distance(point, *a, *b))
        .fold(f64::INFINITY, f64::min);
    let side = |a: (f64, f64), b: (f64, f64)| {
        (b.0 - a.0) * (point.1 - a.1) - (b.1 - a.1) * (point.0 - a.0)
    };
    let signs = edges.map(|(a, b)| side(a, b));
    let inside = signs.iter().all(|s| *s >= 0.0) || signs.iter().all(|s| *s <= 0.0);
    if inside { -to_edge } else { to_edge }
}

pub struct Arrow {
    pub tail: (f64, f64),
    pub base: (f64, f64),
    pub head: [(f64, f64); 3],
}

pub fn arrow_shape(tail: (f64, f64), tip: (f64, f64), width: f64, head: f64) -> Arrow {
    let (dx, dy) = (tip.0 - tail.0, tip.1 - tail.1);
    let length = dx.hypot(dy).max(1e-6);
    let (ux, uy) = (dx / length, dy / length);
    let head_length = ((width * HEAD_LENGTH).max(SMALLEST_HEAD) * head).min(length);
    let half = (width * HEAD_WIDTH).max(SMALLEST_HEAD * 0.7) * head / 2.0;
    let base = (tip.0 - ux * head_length, tip.1 - uy * head_length);
    Arrow {
        tail,
        base,
        head: [
            tip,
            (base.0 - uy * half, base.1 + ux * half),
            (base.0 + uy * half, base.1 - ux * half),
        ],
    }
}

fn ellipse_distance(point: (f64, f64), centre: (f64, f64), radii: (f64, f64)) -> f64 {
    let (a, b) = (radii.0.max(1.0), radii.1.max(1.0));
    let (dx, dy) = (point.0 - centre.0, point.1 - centre.1);
    let r = ((dx / a).powi(2) + (dy / b).powi(2)).sqrt();
    if r < 1e-9 {
        return -a.min(b);
    }
    let gradient = ((dx / (a * a)).powi(2) + (dy / (b * b)).powi(2)).sqrt() / r;
    (r - 1.0) / gradient.max(1e-9)
}

fn box_distance(point: (f64, f64), bounds: Bounds, radius: f64) -> f64 {
    let (cx, cy) = (
        (bounds.left + bounds.right) / 2.0,
        (bounds.top + bounds.bottom) / 2.0,
    );
    let (hw, hh) = (
        (bounds.right - bounds.left) / 2.0,
        (bounds.bottom - bounds.top) / 2.0,
    );
    let r = radius.min(hw).min(hh).max(0.0);
    let qx = (point.0 - cx).abs() - hw + r;
    let qy = (point.1 - cy).abs() - hh + r;
    qx.max(0.0).hypot(qy.max(0.0)) + qx.max(qy).min(0.0) - r
}

fn paint(
    canvas: &mut Rgba32FImage,
    bounds: Bounds,
    colour: [f32; 4],
    cover: impl Fn((f64, f64)) -> f32,
) {
    let (w, h) = (canvas.width(), canvas.height());
    let x0 = whole(bounds.left.floor()).min(w);
    let x1 = whole(bounds.right.ceil()).min(w);
    let y0 = whole(bounds.top.floor()).min(h);
    let y1 = whole(bounds.bottom.ceil()).min(h);
    for y in y0..y1 {
        for x in x0..x1 {
            let amount = cover((f64::from(x) + 0.5, f64::from(y) + 0.5));
            if amount > 0.0 {
                let pixel = canvas.get_pixel_mut(x, y);
                let mut top = colour;
                top[3] *= amount;
                *pixel = Rgba(over(top, pixel.0));
            }
        }
    }
}

fn corners(mark: &Mark, size: (f64, f64)) -> ((f64, f64), (f64, f64)) {
    (
        (mark.x * size.0, mark.y * size.1),
        (mark.x2 * size.0, mark.y2 * size.1),
    )
}

fn draw_mark(canvas: &mut Rgba32FImage, mark: &Mark, step: u32, fonts: &MarkFonts) {
    let size = (f64::from(canvas.width()), f64::from(canvas.height()));
    let colour = unit_colour(&mark.color);
    let width = f64::from(mark.line_width);
    let (start, end) = corners(mark, size);
    match mark.kind {
        MarkKind::Arrow => {
            let arrow = arrow_shape(start, end, width, mark.head_size);
            let bounds = Bounds::around(&[start, end, arrow.head[1], arrow.head[2]], width + 2.0);
            paint(canvas, bounds, colour, |p| {
                let line = segment_distance(p, arrow.tail, arrow.base) - width / 2.0;
                coverage(line.min(triangle_distance(p, arrow.head)))
            });
        }
        MarkKind::Rectangle | MarkKind::Oval => {
            let area = Bounds::around(&[start, end], 0.0);
            let bounds = Bounds::around(&[start, end], width + 2.0);
            let oval = mark.kind == MarkKind::Oval;
            let centre = (
                (area.left + area.right) / 2.0,
                (area.top + area.bottom) / 2.0,
            );
            let radii = (
                (area.right - area.left) / 2.0,
                (area.bottom - area.top) / 2.0,
            );
            let radius = f64::from(mark.radius);
            let edge = |p| {
                if oval {
                    ellipse_distance(p, centre, radii)
                } else {
                    box_distance(p, area, radius)
                }
            };
            if mark.fill > 0.0 {
                let mut inside = colour;
                inside[3] *= numbers::narrow(mark.fill.clamp(0.0, 1.0));
                paint(canvas, bounds, inside, |p| coverage(edge(p)));
            }
            paint(canvas, bounds, colour, |p| {
                coverage(edge(p).abs() - width / 2.0)
            });
        }
        MarkKind::Step => {
            let radius = f64::from(mark.size) / 2.0;
            let bounds = Bounds::around(&[start], radius + 2.0);
            paint(canvas, bounds, colour, |p| {
                coverage((p.0 - start.0).hypot(p.1 - start.1) - radius)
            });
            if let Some(font) = fonts.get(&(mark.font.clone(), mark.weight)) {
                let text = step.to_string();
                let text_size = f64::from(mark.size) * STEP_TEXT;
                let measured = measure(font, text_size, &text);
                let origin = (
                    start.0 - measured.width / 2.0,
                    start.1 - measured.height / 2.0,
                );
                draw_text(
                    canvas,
                    font,
                    text_size,
                    origin,
                    &text,
                    unit_colour(&mark.text_color),
                );
            }
        }
        MarkKind::Text => {
            if let Some(font) = fonts.get(&(mark.font.clone(), mark.weight)) {
                if !mark.text_background.trim().is_empty() {
                    let text_size = f64::from(mark.size);
                    let measured = measure(font, text_size, &mark.text);
                    let pad = text_size * TEXT_PAD;
                    let area = Bounds {
                        left: start.0 - pad,
                        top: start.1 - pad,
                        right: start.0 + measured.width + pad,
                        bottom: start.1 + measured.height + pad,
                    };
                    let radius = text_size * TEXT_RADIUS;
                    paint(
                        canvas,
                        Bounds::around(&[(area.left, area.top), (area.right, area.bottom)], 2.0),
                        unit_colour(&mark.text_background),
                        |p| coverage(box_distance(p, area, radius)),
                    );
                }
                draw_text(
                    canvas,
                    font,
                    f64::from(mark.size),
                    start,
                    &mark.text,
                    colour,
                );
            }
        }
    }
}

pub fn annotate(
    image: &DynamicImage,
    marks: &[Mark],
    first_step: u32,
    fonts: &MarkFonts,
) -> DynamicImage {
    let mut canvas = image.to_rgba32f();
    let mut step = first_step;
    for mark in marks {
        draw_mark(&mut canvas, mark, step, fonts);
        if mark.kind == MarkKind::Step {
            step += 1;
        }
    }
    let out = DynamicImage::ImageRgba32F(canvas);
    if is_deep(image) {
        DynamicImage::ImageRgba16(out.to_rgba16())
    } else {
        DynamicImage::ImageRgba8(out.to_rgba8())
    }
}
