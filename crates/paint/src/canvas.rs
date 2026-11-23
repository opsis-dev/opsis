use opsis_layout::output::{Placed, Rect};
use opsis_layout::transform::Matrix;
use opsis_scene::color::Color;
use opsis_scene::drawing::{Cap, Command, Composite, Drawing, Fill, Join};
use opsis_scene::style::Style;
use opsis_skia::{
    BlendMode, Gradient, GradientSpace, Image, Matrix33, Paint, Path, PointF, RectF, Rgba,
    Sampling, Shader, StrokeCap, StrokeJoin,
};
use opsis_types::numbers::narrow;

use crate::Painter;
use crate::boxes::Areas;
use crate::color::resolve;
use crate::geometry::rect_f;
use crate::text::font_of;

const QUARTER_TURN: f64 = std::f64::consts::FRAC_PI_2;
const FULL_TURN: f64 = std::f64::consts::TAU;
const ARC_HANDLE: f64 = 4.0 / 3.0;

#[derive(Debug, Clone, Copy)]
struct Affine([f64; 6]);

impl Affine {
    const IDENTITY: Self = Self([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);

    fn then(self, other: Self) -> Self {
        let [sxx, syx, sxy, syy, stx, sty] = self.0;
        let [oxx, oyx, oxy, oyy, otx, oty] = other.0;
        Self([
            sxx.mul_add(oxx, sxy * oyx),
            syx.mul_add(oxx, syy * oyx),
            sxx.mul_add(oxy, sxy * oyy),
            syx.mul_add(oxy, syy * oyy),
            sxx.mul_add(otx, sxy.mul_add(oty, stx)),
            syx.mul_add(otx, syy.mul_add(oty, sty)),
        ])
    }

    const fn map(self, x: f64, y: f64) -> PointF {
        let [xx, yx, xy, yy, tx, ty] = self.0;
        PointF::new(
            narrow(xx.mul_add(x, xy.mul_add(y, tx))),
            narrow(yx.mul_add(x, yy.mul_add(y, ty))),
        )
    }

    const fn matrix33(self) -> Matrix33 {
        let [xx, yx, xy, yy, tx, ty] = self.0;
        Matrix33::new([
            narrow(xx),
            narrow(xy),
            narrow(tx),
            narrow(yx),
            narrow(yy),
            narrow(ty),
            0.0,
            0.0,
            1.0,
        ])
    }

    const fn matrix(self) -> Matrix {
        let [xx, yx, xy, yy, tx, ty] = self.0;
        Matrix([
            xx, yx, 0.0, 0.0, xy, yy, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, tx, ty, 0.0, 1.0,
        ])
    }
}

#[derive(Debug, Clone)]
struct State {
    transform: Affine,
    fill: Fill,
    stroke: Fill,
    line_width: f64,
    cap: Cap,
    join: Join,
    dash: (Vec<f32>, f32),
    alpha: f64,
    composite: Composite,
    shadow: Option<(Color, f64, f64, f64)>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            transform: Affine::IDENTITY,
            fill: Fill::Color(Color::BLACK),
            stroke: Fill::Color(Color::BLACK),
            line_width: 1.0,
            cap: Cap::Butt,
            join: Join::Miter,
            dash: (Vec::new(), 0.0),
            alpha: 1.0,
            composite: Composite::SourceOver,
            shadow: None,
        }
    }
}

const fn blend_of(composite: Composite) -> BlendMode {
    match composite {
        Composite::SourceOver => BlendMode::SrcOver,
        Composite::SourceIn => BlendMode::SrcIn,
        Composite::SourceOut => BlendMode::SrcOut,
        Composite::SourceAtop => BlendMode::SrcATop,
        Composite::DestinationOver => BlendMode::DstOver,
        Composite::DestinationIn => BlendMode::DstIn,
        Composite::DestinationOut => BlendMode::DstOut,
        Composite::DestinationAtop => BlendMode::DstATop,
        Composite::Lighter => BlendMode::Plus,
        Composite::Copy => BlendMode::Src,
        Composite::Xor => BlendMode::Xor,
        Composite::Multiply => BlendMode::Multiply,
        Composite::Screen => BlendMode::Screen,
        Composite::Overlay => BlendMode::Overlay,
        Composite::Darken => BlendMode::Darken,
        Composite::Lighten => BlendMode::Lighten,
        Composite::ColorDodge => BlendMode::ColorDodge,
        Composite::ColorBurn => BlendMode::ColorBurn,
        Composite::HardLight => BlendMode::HardLight,
        Composite::SoftLight => BlendMode::SoftLight,
        Composite::Difference => BlendMode::Difference,
        Composite::Exclusion => BlendMode::Exclusion,
        Composite::Hue => BlendMode::Hue,
        Composite::Saturation => BlendMode::Saturation,
        Composite::Color => BlendMode::Color,
        Composite::Luminosity => BlendMode::Luminosity,
    }
}

fn stops_of(stops: &[(opsis_scene::values::Num, Color)]) -> (Vec<Rgba>, Vec<f32>) {
    stops
        .iter()
        .map(|(at, color)| (resolve(color, &Color::BLACK).extended(), narrow(at.0)))
        .unzip()
}

fn paint_for(fill: &Fill, state: &State) -> Paint {
    let mut paint = Paint::new();
    match fill {
        Fill::Color(color) => paint.set_color(resolve(color, &Color::BLACK).rgba()),
        Fill::Linear { from, to, stops } => {
            let (colors, positions) = stops_of(stops);
            let mut gradient = Gradient::new(GradientSpace::Srgb);
            gradient.local = state.transform.matrix33();
            if let Some(shader) = Shader::linear(
                PointF::new(narrow(from.0.0), narrow(from.1.0)),
                PointF::new(narrow(to.0.0), narrow(to.1.0)),
                &colors,
                &positions,
                gradient,
            ) {
                paint.set_shader(&shader);
            }
        }
        Fill::Radial { from, to, stops } => {
            let (colors, positions) = stops_of(stops);
            let mut gradient = Gradient::new(GradientSpace::Srgb);
            gradient.local = state.transform.matrix33();
            if let Some(shader) = Shader::conical(
                (
                    PointF::new(narrow(from.0.0), narrow(from.1.0)),
                    narrow(from.2.0),
                ),
                (PointF::new(narrow(to.0.0), narrow(to.1.0)), narrow(to.2.0)),
                &colors,
                &positions,
                gradient,
            ) {
                paint.set_shader(&shader);
            }
        }
    }
    paint.set_alpha(narrow(state.alpha));
    paint.set_blend(blend_of(state.composite));
    paint
}

fn stroked(mut paint: Paint, state: &State) -> Paint {
    let cap = match state.cap {
        Cap::Butt => StrokeCap::Butt,
        Cap::Round => StrokeCap::Round,
        Cap::Square => StrokeCap::Square,
    };
    let join = match state.join {
        Join::Miter => StrokeJoin::Miter,
        Join::Round => StrokeJoin::Round,
        Join::Bevel => StrokeJoin::Bevel,
    };
    let scale = state.transform.0[0].hypot(state.transform.0[1]);
    paint.set_stroke(narrow(state.line_width * scale), cap, join, 10.0);
    if !state.dash.0.is_empty() {
        let scaled: Vec<f32> = state
            .dash
            .0
            .iter()
            .map(|d| narrow(f64::from(*d) * scale))
            .collect();
        paint.set_dash(&scaled, narrow(f64::from(state.dash.1) * scale));
    }
    paint
}

fn arc(
    path: &mut Path,
    state: &State,
    (cx, cy, r): (f64, f64, f64),
    (start, end, anticlockwise): (f64, f64, bool),
    fresh: bool,
) {
    let mut sweep = end - start;
    if anticlockwise {
        if sweep > 0.0 || sweep <= -FULL_TURN {
            sweep = (sweep % FULL_TURN)
                - if sweep % FULL_TURN > 0.0 {
                    FULL_TURN
                } else {
                    0.0
                };
        }
        if (end - start).abs() >= FULL_TURN {
            sweep = -FULL_TURN;
        }
    } else {
        if sweep < 0.0 || sweep >= FULL_TURN {
            sweep = (sweep % FULL_TURN)
                + if sweep % FULL_TURN < 0.0 {
                    FULL_TURN
                } else {
                    0.0
                };
        }
        if (end - start).abs() >= FULL_TURN {
            sweep = FULL_TURN;
        }
    }
    let point = |angle: f64| (r.mul_add(angle.cos(), cx), r.mul_add(angle.sin(), cy));
    let (sx, sy) = point(start);
    let first = state.transform.map(sx, sy);
    if fresh {
        path.move_to(first);
    } else {
        path.line_to(first);
    }
    let pieces = opsis_types::numbers::whole((sweep.abs() / QUARTER_TURN).ceil()).max(1);
    let step = sweep / f64::from(pieces);
    let handle = ARC_HANDLE * (step / 4.0).tan() * r;
    let mut angle = start;
    for _ in 0..pieces {
        let next = angle + step;
        let (x0, y0) = point(angle);
        let (x1, y1) = point(next);
        let c0 = (
            (-handle).mul_add(angle.sin(), x0),
            handle.mul_add(angle.cos(), y0),
        );
        let c1 = (
            handle.mul_add(next.sin(), x1),
            (-handle).mul_add(next.cos(), y1),
        );
        path.cubic_to(
            state.transform.map(c0.0, c0.1),
            state.transform.map(c1.0, c1.1),
            state.transform.map(x1, y1),
        );
        angle = next;
    }
}

fn rect_path(state: &State, [x, y, w, h]: [f64; 4]) -> Path {
    let mut path = Path::new();
    path.move_to(state.transform.map(x, y));
    path.line_to(state.transform.map(x + w, y));
    path.line_to(state.transform.map(x + w, y + h));
    path.line_to(state.transform.map(x, y + h));
    path.close();
    path
}

struct Replay<'p, 'a, 'b, 'c> {
    painter: &'p mut Painter<'a, 'b, 'c>,
    base: Matrix,
    placed: &'p Placed,
}

impl Replay<'_, '_, '_, '_> {
    fn reset(&mut self) {
        let base = self.base;
        self.painter.set_matrix(&base);
    }

    fn shadowed(&mut self, state: &State, draw: &mut dyn FnMut(&mut Self, &Paint)) {
        if let Some((color, blur, x, y)) = &state.shadow {
            let rgba = resolve(color, &Color::BLACK).rgba();
            if rgba.alpha > 0.0 {
                let mut paint = Paint::filled(rgba);
                paint.set_alpha(narrow(f64::from(rgba.alpha) * state.alpha));
                paint.set_mask_blur(narrow(blur / 2.0));
                let base = self.base;
                self.painter
                    .set_matrix(&base.then(&Matrix::translation(*x, *y, 0.0)));
                draw(self, &paint);
                self.reset();
            }
        }
    }

    fn path(&mut self, path: &Path, paint: &Paint, state: &State) {
        self.shadowed(state, &mut |me, shadow| {
            me.painter.surface.draw_path(path, shadow);
        });
        self.painter.surface.draw_path(path, paint);
    }

    fn text(&mut self, index: usize, paint: &Paint, state: &State) {
        let Some((_, runs)) = self.placed.drawn.iter().find(|(i, _)| *i == index) else {
            return;
        };
        let runs = runs.clone();
        let transform = state.transform.matrix();
        let draw = |me: &mut Self, paint: &Paint, shift: (f64, f64)| {
            let base = me.base.then(&Matrix::translation(shift.0, shift.1, 0.0));
            me.painter.set_matrix(&base.then(&transform));
            for run in &runs {
                let (glyphs, positions): (Vec<u16>, Vec<PointF>) = run
                    .glyphs
                    .iter()
                    .map(|g| (g.id, PointF::new(narrow(g.x), narrow(g.y))))
                    .unzip();
                let font = font_of(run);
                let Painter {
                    surface, resources, ..
                } = &mut *me.painter;
                if let Some(typeface) = resources.typeface(run) {
                    surface.draw_glyphs(typeface, font, (&glyphs, &positions), PointF::ZERO, paint);
                }
            }
            me.reset();
        };
        if let Some((color, blur, x, y)) = state.shadow.clone() {
            let rgba = resolve(&color, &Color::BLACK).rgba();
            if rgba.alpha > 0.0 {
                let mut shadow = Paint::filled(rgba);
                shadow.set_mask_blur(narrow(blur / 2.0));
                draw(self, &shadow, (x, y));
            }
        }
        draw(self, paint, (0.0, 0.0));
    }

    fn image_data(&mut self, (x, y): (i32, i32), (width, height): (u32, u32), pixels: &[u8]) {
        let Some(image) = Image::from_rgba(width, height, pixels) else {
            return;
        };
        let mut paint = Paint::new();
        paint.set_blend(BlendMode::Src);
        let full = RectF::new(
            0.0,
            0.0,
            narrow(f64::from(width)),
            narrow(f64::from(height)),
        );
        let target = RectF::new(
            narrow(f64::from(x)),
            narrow(f64::from(y)),
            narrow(f64::from(width)),
            narrow(f64::from(height)),
        );
        self.painter
            .surface
            .draw_image_rect(&image, (full, target), Sampling::Nearest, &paint);
    }

    fn picture(&mut self, source: &str, rect: [f64; 4], state: &State) {
        let transform = state.transform.matrix();
        let device = self.painter.placement();
        let base = self.base;
        let Painter {
            surface, resources, ..
        } = &mut *self.painter;
        let Some(image) = resources.image(source) else {
            return;
        };
        let full = RectF::new(
            0.0,
            0.0,
            narrow(f64::from(image.width())),
            narrow(f64::from(image.height())),
        );
        let target = RectF::new(
            narrow(rect[0]),
            narrow(rect[1]),
            narrow(rect[2]),
            narrow(rect[3]),
        );
        let mut paint = Paint::new();
        paint.set_alpha(narrow(state.alpha));
        paint.set_blend(blend_of(state.composite));
        device.set(surface, &base.then(&transform));
        surface.draw_image_rect(&image, (full, target), Sampling::Linear, &paint);
        device.set(surface, &base);
    }
}

const fn number(n: opsis_scene::values::Num) -> f64 {
    n.0
}

pub fn draw(painter: &mut Painter<'_, '_, '_>, placed: &Placed, style: &Style, canvas: &Drawing) {
    if canvas.width == 0 || canvas.height == 0 {
        return;
    }
    let areas = Areas::of(placed, style, painter);
    let content: Rect = areas.content;
    let base = placed
        .matrix
        .then(&Matrix::translation(content.x, content.y, 0.0))
        .then(&Matrix::scaling(
            content.width / f64::from(canvas.width),
            content.height / f64::from(canvas.height),
            1.0,
        ));
    let saved = painter.surface.save();
    painter.surface.clip_rect(rect_f(content), true);
    painter.surface.save_layer(None, &Paint::new());
    let mut replay = Replay {
        painter,
        base,
        placed,
    };
    replay.reset();
    let mut state = State::default();
    let mut stack: Vec<State> = Vec::new();
    let mut path = Path::new();
    let mut fresh = true;
    for (index, command) in canvas.commands.iter().enumerate() {
        match command {
            Command::Save => {
                stack.push(state.clone());
                replay.painter.surface.save();
            }
            Command::Restore => {
                if let Some(previous) = stack.pop() {
                    state = previous;
                    replay.painter.surface.restore();
                    replay.reset();
                }
            }
            Command::Translate(x, y) => {
                state.transform =
                    state
                        .transform
                        .then(Affine([1.0, 0.0, 0.0, 1.0, number(*x), number(*y)]));
            }
            Command::Rotate(angle) => {
                let (sin, cos) = angle.0.sin_cos();
                state.transform = state
                    .transform
                    .then(Affine([cos, sin, -sin, cos, 0.0, 0.0]));
            }
            Command::Scale(x, y) => {
                state.transform =
                    state
                        .transform
                        .then(Affine([number(*x), 0.0, 0.0, number(*y), 0.0, 0.0]));
            }
            Command::Transform(m) => {
                state.transform = state.transform.then(Affine(m.map(number)));
            }
            Command::SetTransform(m) => state.transform = Affine(m.map(number)),
            Command::FillStyle(fill) => state.fill = fill.clone(),
            Command::StrokeStyle(fill) => state.stroke = fill.clone(),
            Command::LineWidth(width) => state.line_width = width.0,
            Command::LineCap(cap) => state.cap = *cap,
            Command::LineJoin(join) => state.join = *join,
            Command::LineDash(intervals, offset) => {
                state.dash = (
                    intervals.iter().map(|n| narrow(n.0)).collect(),
                    narrow(offset.0),
                );
            }
            Command::GlobalAlpha(alpha) => state.alpha = alpha.0.clamp(0.0, 1.0),
            Command::Composite(composite) => state.composite = *composite,
            Command::Shadow { color, blur, x, y } => {
                state.shadow = Some((color.clone(), blur.0, x.0, y.0));
            }
            Command::Font(_) | Command::TextAlign(_) | Command::TextBaseline(_) => {}
            Command::BeginPath => {
                path = Path::new();
                fresh = true;
            }
            Command::MoveTo(x, y) => {
                path.move_to(state.transform.map(x.0, y.0));
                fresh = false;
            }
            Command::LineTo(x, y) => {
                let point = state.transform.map(x.0, y.0);
                if fresh {
                    path.move_to(point);
                } else {
                    path.line_to(point);
                }
                fresh = false;
            }
            Command::QuadraticTo(cx, cy, x, y) => {
                path.quad_to(
                    state.transform.map(cx.0, cy.0),
                    state.transform.map(x.0, y.0),
                );
                fresh = false;
            }
            Command::BezierTo([first_x, first_y, second_x, second_y, end_x, end_y]) => {
                path.cubic_to(
                    state.transform.map(first_x.0, first_y.0),
                    state.transform.map(second_x.0, second_y.0),
                    state.transform.map(end_x.0, end_y.0),
                );
                fresh = false;
            }
            Command::Arc {
                center,
                radius,
                start,
                end,
                anticlockwise,
            } => {
                arc(
                    &mut path,
                    &state,
                    (center.0.0, center.1.0, radius.0),
                    (start.0, end.0, *anticlockwise),
                    fresh,
                );
                fresh = false;
            }
            Command::Rect(r) => {
                path.add_path(&rect_path(&state, r.map(number)), Matrix33::IDENTITY);
                fresh = false;
            }
            Command::ClosePath => path.close(),
            Command::Fill => {
                let paint = paint_for(&state.fill.clone(), &state);
                replay.path(&path, &paint, &state);
            }
            Command::Stroke => {
                let paint = stroked(paint_for(&state.stroke.clone(), &state), &state);
                replay.path(&path, &paint, &state);
            }
            Command::Clip => replay.painter.surface.clip_path(&path, true),
            Command::FillRect(r) => {
                let shape = rect_path(&state, r.map(number));
                let paint = paint_for(&state.fill.clone(), &state);
                replay.path(&shape, &paint, &state);
            }
            Command::StrokeRect(r) => {
                let shape = rect_path(&state, r.map(number));
                let paint = stroked(paint_for(&state.stroke.clone(), &state), &state);
                replay.path(&shape, &paint, &state);
            }
            Command::ClearRect(r) => {
                let shape = rect_path(&state, r.map(number));
                let mut paint = Paint::new();
                paint.set_blend(BlendMode::Clear);
                replay.painter.surface.draw_path(&shape, &paint);
            }
            Command::FillText { .. } => {
                let paint = paint_for(&state.fill.clone(), &state);
                replay.text(index, &paint, &state);
            }
            Command::StrokeText { .. } => {
                let paint = stroked(paint_for(&state.stroke.clone(), &state), &state);
                replay.text(index, &paint, &state);
            }
            Command::PutImageData {
                at,
                width,
                height,
                pixels,
            } => replay.image_data(*at, (*width, *height), pixels),
            Command::DrawImage { source, rect } => {
                replay.picture(&source.0, rect.map(number), &state);
            }
        }
    }
    replay.painter.surface.restore_to(saved);
}
