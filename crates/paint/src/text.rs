use opsis_layout::output::{Placed, PlacedKind, TextRun};
use opsis_scene::color::Color;
use opsis_scene::style::{DecorationLine, DecorationStyle, PaintOrder, Visibility};
use opsis_skia::{FontSpec, Paint, Path, PointF, StrokeCap, StrokeJoin};
use opsis_types::numbers::narrow;

use crate::Painter;
use crate::color::resolve;
use crate::geometry::px;

pub fn font_of(run: &TextRun) -> FontSpec {
    FontSpec {
        size: run.size,
        embolden: run.embolden,
        skew_x: narrow(-f64::from(run.skew).to_radians().tan()),
    }
}

fn positions(run: &TextRun, dx: f64, dy: f64) -> (Vec<u16>, Vec<PointF>) {
    run.glyphs
        .iter()
        .map(|g| (g.id, PointF::new(narrow(g.x + dx), narrow(g.y + dy))))
        .unzip()
}

fn draw_runs(
    painter: &mut Painter<'_, '_, '_>,
    runs: &[TextRun],
    offset: (f64, f64),
    paint: &Paint,
) {
    for run in runs {
        let (glyphs, points) = positions(run, offset.0, offset.1);
        let font = font_of(run);
        let Painter {
            surface, resources, ..
        } = painter;
        if let Some(typeface) = resources.typeface(run) {
            surface.draw_glyphs(typeface, font, (&glyphs, &points), PointF::ZERO, paint);
        }
    }
}

pub fn outlines(painter: &mut Painter<'_, '_, '_>, runs: &[TextRun], into: &mut Path) {
    for run in runs {
        let (glyphs, points) = positions(run, 0.0, 0.0);
        let font = font_of(run);
        if let Some(typeface) = painter.resources.typeface(run) {
            let outline = typeface.outlines(font, &glyphs, &points, PointF::ZERO);
            into.add_path(&outline, opsis_skia::Matrix33::IDENTITY);
        }
    }
}

pub fn draw_text(painter: &mut Painter<'_, '_, '_>, placed: &Placed) {
    let PlacedKind::Text { runs, .. } = &placed.kind else {
        return;
    };
    let style = &placed.text;
    if style.visibility == Visibility::Hidden || runs.is_empty() {
        return;
    }
    let fill = resolve(&style.color, &Color::BLACK).rgba();
    for shadow in style.text_shadow.iter().rev() {
        let color = resolve(&shadow.color, &style.color).rgba();
        if color.alpha <= 0.0 {
            continue;
        }
        let dx = px(&shadow.x, style, &painter.context, 0.0);
        let dy = px(&shadow.y, style, &painter.context, 0.0);
        let blur = px(&shadow.blur, style, &painter.context, 0.0).max(0.0);
        let mut paint = Paint::filled(color);
        paint.set_mask_blur(narrow(blur / 2.0));
        draw_runs(painter, runs, (dx, dy), &paint);
    }
    let stroke = style.text_stroke.as_ref().and_then(|stroke| {
        let width = px(&stroke.width, style, &painter.context, 0.0);
        (width > 0.0).then(|| {
            let color = resolve(stroke.color.as_ref().unwrap_or(&style.color), &style.color).rgba();
            let mut paint = Paint::filled(color);
            paint.set_stroke(narrow(width), StrokeCap::Butt, StrokeJoin::Miter, 4.0);
            paint
        })
    });
    decorate(painter, style, runs, false);
    let fill_paint = Paint::filled(fill);
    let stroke_first = style.paint_order == PaintOrder::StrokeFirst;
    if stroke_first && let Some(stroke) = &stroke {
        draw_runs(painter, runs, (0.0, 0.0), stroke);
    }
    if fill.alpha > 0.0 {
        draw_runs(painter, runs, (0.0, 0.0), &fill_paint);
    }
    if !stroke_first && let Some(stroke) = &stroke {
        draw_runs(painter, runs, (0.0, 0.0), stroke);
    }
    decorate(painter, style, runs, true);
}

const MIN_THICKNESS: f64 = 1.0;
const WAVE_STEPS: f64 = 2.0;

fn decorate(
    painter: &mut Painter<'_, '_, '_>,
    style: &opsis_layout::computed::TextStyle,
    runs: &[TextRun],
    over_text: bool,
) {
    for decoration in &style.decorations {
        let color = resolve(&decoration.color, &style.color).rgba();
        if color.alpha <= 0.0 {
            continue;
        }
        for line in &decoration.lines {
            if (*line == DecorationLine::LineThrough) != over_text {
                continue;
            }
            for run in runs {
                let (offset, size) = match line {
                    DecorationLine::LineThrough => run.strikethrough,
                    _ => run.underline,
                };
                let thickness = decoration
                    .thickness
                    .unwrap_or_else(|| f64::from(size))
                    .round()
                    .max(MIN_THICKNESS);
                let top = match line {
                    DecorationLine::Overline => run.baseline - f64::from(run.ascent),
                    DecorationLine::LineThrough => {
                        run.baseline - f64::from(offset) - thickness / 2.0
                    }
                    _ => run.baseline - f64::from(offset).round(),
                };
                draw_line(
                    painter,
                    decoration.style,
                    (run.start, run.advance),
                    (top, thickness),
                    color,
                );
            }
        }
    }
}

fn draw_line(
    painter: &mut Painter<'_, '_, '_>,
    kind: DecorationStyle,
    (start, length): (f64, f64),
    (top, thickness): (f64, f64),
    color: opsis_skia::Rgba,
) {
    let mut paint = Paint::filled(color);
    match kind {
        DecorationStyle::Wavy => {
            let amplitude = thickness * WAVE_STEPS;
            let step = amplitude * 2.0;
            let middle = top + thickness / 2.0;
            let mut path = Path::new();
            path.move_to(PointF::new(narrow(start), narrow(middle)));
            let waves = opsis_types::numbers::whole((length / step).ceil());
            for k in 0..waves {
                let x = f64::from(k).mul_add(step, start);
                let next = x + step;
                let peak = if k % 2 == 0 {
                    middle - amplitude
                } else {
                    middle + amplitude
                };
                path.quad_to(
                    PointF::new(narrow(x + step / 2.0), narrow(peak)),
                    PointF::new(narrow(next), narrow(middle)),
                );
            }
            paint.set_stroke(narrow(thickness), StrokeCap::Butt, StrokeJoin::Round, 4.0);
            painter.surface.save();
            painter.surface.clip_rect(
                opsis_skia::RectF::new(
                    narrow(start),
                    narrow(middle - amplitude - thickness),
                    narrow(length),
                    narrow(2.0 * (amplitude + thickness)),
                ),
                true,
            );
            painter.surface.draw_path(&path, &paint);
            painter.surface.restore();
        }
        DecorationStyle::Dotted | DecorationStyle::Dashed => {
            let mut path = Path::new();
            let middle = top + thickness / 2.0;
            path.move_to(PointF::new(narrow(start), narrow(middle)));
            path.line_to(PointF::new(narrow(start + length), narrow(middle)));
            let dash = if kind == DecorationStyle::Dotted {
                thickness
            } else {
                thickness * 3.0
            };
            paint.set_stroke(narrow(thickness), StrokeCap::Butt, StrokeJoin::Miter, 4.0);
            paint.set_dash(&[narrow(dash), narrow(dash)], 0.0);
            painter.surface.draw_path(&path, &paint);
        }
        DecorationStyle::Double => {
            for y in [top, thickness.mul_add(2.0, top)] {
                painter.surface.draw_rect(
                    opsis_skia::RectF::new(
                        narrow(start),
                        narrow(y),
                        narrow(length),
                        narrow(thickness),
                    ),
                    &paint,
                );
            }
        }
        DecorationStyle::Solid => {
            painter.surface.draw_rect(
                opsis_skia::RectF::new(
                    narrow(start),
                    narrow(top),
                    narrow(length),
                    narrow(thickness),
                ),
                &paint,
            );
        }
    }
}
