use opsis_layout::output::{Placed, PlacedKind, Rect};
use opsis_layout::transform::Matrix;
use opsis_scene::color::Color;
use opsis_scene::paint::{AssetRef, BoxArea, Image as LayerImage, ImageLayer, LayerSize, Repeat};
use opsis_scene::prop;
use opsis_scene::style::{BorderSide, BorderStyle, ImageRendering, ObjectFit, Style};
use opsis_scene::values::{Length, Position2};
use opsis_skia::{BlendMode, Paint, Path, RectF, Rgba, Sampling, StrokeCap, StrokeJoin};
use opsis_types::numbers::{exact_eq, narrow};

use crate::Painter;
use crate::color::resolve;
use crate::geometry::{
    Radii, is_rounded, moved, outset, point, px, radii, rect_f, round_rect, shrink, spread,
};
use crate::gradients::shader;
use crate::resources::Resources;
use crate::svg::SvgPainter;

const FAR: f32 = 1.0e6;
const THICK_BORDER: f64 = 3.0;

#[derive(Debug, Clone, Copy)]
pub struct Areas {
    pub border: Rect,
    pub padding: Rect,
    pub content: Rect,
    pub outer: Radii,
    pub inner: Radii,
    pub content_radii: Radii,
}

impl Areas {
    pub fn of(placed: &Placed, style: &Style, painter: &Painter<'_, '_, '_>) -> Self {
        let border = snapped(placed.border_box, placed.snap_origin);
        let padding = placed.border.inset(border);
        let content = placed.padding.inset(padding);
        let outer = radii(
            prop!(style, BorderRadius),
            border,
            &placed.text,
            &painter.context,
        );
        let b = &placed.border;
        let inner = shrink(&outer, [b.top, b.right, b.bottom, b.left]);
        let p = &placed.padding;
        let content_radii = shrink(&inner, [p.top, p.right, p.bottom, p.left]);
        Self {
            border,
            padding,
            content,
            outer,
            inner,
            content_radii,
        }
    }

    pub const fn area(&self, area: BoxArea) -> (Rect, &Radii) {
        match area {
            BoxArea::BorderBox | BoxArea::Text => (self.border, &self.outer),
            BoxArea::PaddingBox => (self.padding, &self.inner),
            BoxArea::ContentBox => (self.content, &self.content_radii),
        }
    }
}

pub const fn blend_of(mode: opsis_scene::paint::BlendMode) -> BlendMode {
    use opsis_scene::paint::BlendMode as Scene;
    match mode {
        Scene::Normal => BlendMode::SrcOver,
        Scene::Multiply => BlendMode::Multiply,
        Scene::Screen => BlendMode::Screen,
        Scene::Overlay => BlendMode::Overlay,
        Scene::Darken => BlendMode::Darken,
        Scene::Lighten => BlendMode::Lighten,
        Scene::ColorDodge => BlendMode::ColorDodge,
        Scene::ColorBurn => BlendMode::ColorBurn,
        Scene::HardLight => BlendMode::HardLight,
        Scene::SoftLight => BlendMode::SoftLight,
        Scene::Difference => BlendMode::Difference,
        Scene::Exclusion => BlendMode::Exclusion,
        Scene::Hue => BlendMode::Hue,
        Scene::Saturation => BlendMode::Saturation,
        Scene::Color => BlendMode::Color,
        Scene::Luminosity => BlendMode::Luminosity,
        Scene::PlusLighter => BlendMode::Plus,
    }
}

fn outside(shape: &opsis_skia::RoundRect) -> Path {
    let mut path = Path::new();
    path.add_rect(RectF::new(-FAR, -FAR, 2.0 * FAR, 2.0 * FAR));
    path.add_round_rect(*shape);
    path.set_even_odd(true);
    path
}

pub fn draw_box(painter: &mut Painter<'_, '_, '_>, placed: &Placed, style: &Style) {
    let areas = Areas::of(placed, style, painter);
    let current = placed.text.color.clone();
    draw_shadows(painter, placed, style, &areas, &current, false);
    if prop!(style, BackgroundClipText).copied().unwrap_or(false) {
        let laid = painter.laid;
        let mut outline = Path::new();
        for inner in laid
            .boxes
            .iter()
            .filter(|b| b.path.starts_with(&placed.path))
            .filter(|b| b.text.visibility != opsis_scene::style::Visibility::Hidden)
        {
            if let PlacedKind::Text { runs, .. } = &inner.kind {
                crate::text::outlines(painter, runs, &mut outline);
            }
        }
        let saved = painter.surface.save();
        painter.surface.clip_path(&outline, true);
        draw_backgrounds(painter, placed, style, &areas, &current);
        painter.surface.restore_to(saved);
    } else {
        draw_backgrounds(painter, placed, style, &areas, &current);
    }
    draw_shadows(painter, placed, style, &areas, &current, true);
    draw_borders(painter, placed, style, &areas, &current);
    draw_outline(painter, placed, style, &areas, &current);
}

fn draw_shadows(
    painter: &mut Painter<'_, '_, '_>,
    placed: &Placed,
    style: &Style,
    areas: &Areas,
    current: &Color,
    inset: bool,
) {
    let Some(shadows) = prop!(style, BoxShadow) else {
        return;
    };
    let text = &placed.text;
    for shadow in shadows.iter().rev().filter(|s| s.inset == inset) {
        let color = resolve(&shadow.color, current).rgba();
        if color.alpha <= 0.0 {
            continue;
        }
        let dx = px(&shadow.x, text, &painter.context, 0.0);
        let dy = px(&shadow.y, text, &painter.context, 0.0);
        let blur = px(&shadow.blur, text, &painter.context, 0.0).max(0.0);
        let grow = px(&shadow.spread, text, &painter.context, 0.0);
        let mut paint = Paint::filled(color);
        paint.set_mask_blur(narrow(blur / 2.0));
        let surface = &mut *painter.surface;
        let saved = surface.save();
        if inset {
            surface.clip_round_rect(round_rect(areas.padding, &areas.inner), true);
            let hole = round_rect(
                moved(outset(areas.padding, -grow), dx, dy),
                &spread(&areas.inner, -grow),
            );
            let mut ring = outside(&hole);
            ring.set_even_odd(true);
            surface.draw_path(&ring, &paint);
        } else {
            surface.clip_path(&outside(&round_rect(areas.border, &areas.outer)), true);
            let shape = round_rect(
                moved(outset(areas.border, grow), dx, dy),
                &spread(&areas.outer, grow),
            );
            surface.draw_round_rect(shape, &paint);
        }
        surface.restore_to(saved);
    }
}

pub fn draw_backgrounds(
    painter: &mut Painter<'_, '_, '_>,
    placed: &Placed,
    style: &Style,
    areas: &Areas,
    current: &Color,
) {
    let layers: &[ImageLayer] = prop!(style, Background).map_or(&[], Vec::as_slice);
    let isolated = layers
        .iter()
        .any(|l| l.blend != opsis_scene::paint::BlendMode::Normal);
    let saved = painter.surface.save();
    if isolated {
        painter.surface.save_layer(None, &Paint::new());
    }
    if let Some(color) = prop!(style, BackgroundColor) {
        let rgba = resolve(color, current).rgba();
        if rgba.alpha > 0.0 {
            let clip = layers.last().map_or(BoxArea::BorderBox, |l| l.clip);
            let (rect, radii) = areas.area(clip);
            fill_shape(painter, rect, radii, &Paint::filled(rgba));
        }
    }
    for layer in layers.iter().rev() {
        draw_layer(painter, placed, layer, areas, current);
    }
    painter.surface.restore_to(saved);
}

fn fill_shape(painter: &mut Painter<'_, '_, '_>, rect: Rect, radii: &Radii, paint: &Paint) {
    if is_rounded(radii) {
        painter
            .surface
            .draw_round_rect(round_rect(rect, radii), paint);
    } else {
        painter.surface.draw_rect(rect_f(rect), paint);
    }
}

fn layer_size(
    layer: &ImageLayer,
    area: Rect,
    intrinsic: Option<(f64, f64)>,
    placed: &Placed,
    painter: &Painter<'_, '_, '_>,
) -> (f64, f64) {
    let text = &placed.text;
    let fill = (area.width, area.height);
    match (&layer.size, intrinsic) {
        (LayerSize::Auto | LayerSize::Cover | LayerSize::Contain, None) => fill,
        (LayerSize::Auto, Some(size)) => size,
        (LayerSize::Cover, Some((w, h))) => {
            let scale = (area.width / w).max(area.height / h);
            (w * scale, h * scale)
        }
        (LayerSize::Contain, Some((w, h))) => {
            let scale = (area.width / w).min(area.height / h);
            (w * scale, h * scale)
        }
        (LayerSize::Explicit(x, y), intrinsic) => {
            let width = (*x != Length::Auto).then(|| px(x, text, &painter.context, area.width));
            let height = (*y != Length::Auto).then(|| px(y, text, &painter.context, area.height));
            match (width, height, intrinsic) {
                (Some(w), Some(h), _) => (w, h),
                (Some(w), None, Some((iw, ih))) if iw > 0.0 => (w, w * ih / iw),
                (None, Some(h), Some((iw, ih))) if ih > 0.0 => (h * iw / ih, h),
                (Some(w), None, _) => (w, intrinsic.map_or(area.height, |s| s.1)),
                (None, Some(h), _) => (intrinsic.map_or(area.width, |s| s.0), h),
                (None, None, Some(size)) => size,
                (None, None, None) => fill,
            }
        }
    }
}

fn starts(position: f64, size: f64, from: f64, to: f64, repeat: Repeat) -> Vec<f64> {
    if repeat == Repeat::NoRepeat || size <= 0.0 {
        return vec![position];
    }
    let before = ((position - from) / size).ceil().max(0.0);
    let first = (-before).mul_add(size, position);
    let count = ((to - first) / size).ceil().max(0.0);
    (0..opsis_types::numbers::whole(count))
        .map(|k| f64::from(k).mul_add(size, first))
        .collect()
}

pub fn draw_layer(
    painter: &mut Painter<'_, '_, '_>,
    placed: &Placed,
    layer: &ImageLayer,
    areas: &Areas,
    current: &Color,
) {
    let (origin, _) = areas.area(layer.origin);
    let (clip, clip_radii) = areas.area(layer.clip);
    let intrinsic = match &layer.image {
        LayerImage::Url(AssetRef(source)) if Resources::is_svg(source) => painter
            .resources
            .svg(source)
            .map(|t| (f64::from(t.size().width()), f64::from(t.size().height()))),
        LayerImage::Url(AssetRef(source)) => painter
            .resources
            .image(source)
            .map(|i| (f64::from(i.width()), f64::from(i.height()))),
        LayerImage::Gradient(_) => None,
    };
    let (w, h) = layer_size(layer, origin, intrinsic, placed, painter);
    let text = &placed.text;
    let x = origin.x + px(&layer.position.x, text, &painter.context, origin.width - w);
    let y = origin.y + px(&layer.position.y, text, &painter.context, origin.height - h);
    let xs = starts(x, w, clip.x, clip.x + clip.width, layer.repeat.0);
    let ys = starts(y, h, clip.y, clip.y + clip.height, layer.repeat.1);
    let saved = painter.surface.save();
    if is_rounded(clip_radii) {
        painter
            .surface
            .clip_round_rect(round_rect(clip, clip_radii), true);
    } else {
        painter.surface.clip_rect(rect_f(clip), true);
    }
    for ty in &ys {
        for tx in &xs {
            let tile = Rect {
                x: *tx,
                y: *ty,
                width: w,
                height: h,
            };
            match &layer.image {
                LayerImage::Gradient(gradient) => {
                    let Some(shader) = shader(gradient, tile, text, &painter.context, current)
                    else {
                        continue;
                    };
                    let mut paint = Paint::new();
                    paint.set_shader(&shader);
                    paint.set_dither(true);
                    paint.set_blend(blend_of(layer.blend));
                    painter.surface.draw_rect(rect_f(tile), &paint);
                }
                LayerImage::Url(AssetRef(source)) if Resources::is_svg(source) => {
                    let matrix = placed.matrix;
                    let device = painter.placement();
                    let Painter {
                        surface, resources, ..
                    } = painter;
                    if let Some(tree) = resources.svg(source) {
                        let size = tree.size();
                        let base = matrix.then(&Matrix::translation(tile.x, tile.y, 0.0)).then(
                            &Matrix::scaling(
                                tile.width / f64::from(size.width()),
                                tile.height / f64::from(size.height()),
                                1.0,
                            ),
                        );
                        let inner = surface.save();
                        let mut decode = |bytes: &[u8]| opsis_skia::Image::decode(bytes);
                        SvgPainter {
                            surface,
                            decode: &mut decode,
                            device,
                        }
                        .group(tree.root(), &base);
                        surface.restore_to(inner);
                    }
                }
                LayerImage::Url(AssetRef(source)) => {
                    let mut paint = Paint::new();
                    paint.set_blend(blend_of(layer.blend));
                    let Painter {
                        surface, resources, ..
                    } = painter;
                    if let Some(image) = resources.image(source) {
                        let full = RectF::new(
                            0.0,
                            0.0,
                            narrow(f64::from(image.width())),
                            narrow(f64::from(image.height())),
                        );
                        surface.draw_image_rect(
                            &image,
                            (full, rect_f(tile)),
                            Sampling::Linear,
                            &paint,
                        );
                    }
                }
            }
        }
    }
    painter.surface.restore_to(saved);
}

fn side_color(side: &BorderSide, current: &Color) -> Rgba {
    resolve(side.color.as_ref().unwrap_or(current), current).rgba()
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the colour matrices and gradient geometry follow the browser's arithmetic, one product and sum at a time, and the pictures have to match it"
)]
fn draw_borders(
    painter: &mut Painter<'_, '_, '_>,
    placed: &Placed,
    style: &Style,
    areas: &Areas,
    current: &Color,
) {
    let Some(border) = prop!(style, Border) else {
        return;
    };
    let widths = [
        placed.border.top,
        placed.border.right,
        placed.border.bottom,
        placed.border.left,
    ];
    if widths.iter().all(|w| *w <= 0.0) {
        return;
    }
    let sides = [&border.top, &border.right, &border.bottom, &border.left];
    let colors = sides.map(|s| side_color(s, current));
    let outer = round_rect(areas.border, &areas.outer);
    let inner = round_rect(areas.padding, &areas.inner);
    let uniform = sides
        .iter()
        .all(|s| s.style == BorderStyle::Solid || s.style == BorderStyle::Double)
        && colors.iter().all(|c| *c == colors[0]);
    if uniform {
        painter
            .surface
            .draw_nested_round_rect(outer, inner, &Paint::filled(colors[0]));
        return;
    }
    let b = areas.border;
    let p = areas.padding;
    let same_stroke = sides.iter().all(|s| s.style == sides[0].style)
        && matches!(sides[0].style, BorderStyle::Dashed | BorderStyle::Dotted)
        && colors.iter().all(|c| *c == colors[0])
        && widths.iter().all(|w| exact_eq(*w, widths[0]));
    let rounded = areas.outer.iter().any(|(x, y)| *x > 0.0 || *y > 0.0);
    if same_stroke && rounded {
        let path = stroke_path(b, areas, widths[0]);
        let length = f64::from(path.length());
        painter.surface.draw_path(
            &path,
            &dash_paint(sides[0].style, widths[0], colors[0], length, true),
        );
        return;
    }
    let reach = areas
        .outer
        .iter()
        .map(|(x, y)| x.max(*y))
        .fold(0.0_f64, f64::max)
        .mul_add(2.0, widths.iter().fold(0.0_f64, |m, w| m.max(*w)) * 2.0)
        .min(b.width.min(b.height) / 2.0);
    let inward = |(ox, oy): (f64, f64), (ix, iy): (f64, f64)| {
        let (dx, dy) = (ix - ox, iy - oy);
        let length = dx.hypot(dy);
        if length <= 0.0 {
            return point(ix, iy);
        }
        point(ix + dx / length * reach, iy + dy / length * reach)
    };
    let corners_outer = [
        (b.x, b.y),
        (b.x + b.width, b.y),
        (b.x + b.width, b.y + b.height),
        (b.x, b.y + b.height),
    ];
    let corners_inner = [
        (p.x, p.y),
        (p.x + p.width, p.y),
        (p.x + p.width, p.y + p.height),
        (p.x, p.y + p.height),
    ];
    for (index, side) in sides.iter().enumerate() {
        if widths[index] <= 0.0 || colors[index].alpha <= 0.0 || side.style == BorderStyle::None {
            continue;
        }
        let next = (index + 1) % 4;
        let mut wedge = Path::new();
        let far = |(x, y): (f64, f64), (cx, cy): (f64, f64)| {
            point(x + (x - cx) * 4.0, y + (y - cy) * 4.0)
        };
        wedge.move_to(inward(corners_outer[index], corners_inner[index]));
        wedge.line_to(far(corners_outer[index], corners_inner[index]));
        wedge.line_to(far(corners_outer[next], corners_inner[next]));
        wedge.line_to(inward(corners_outer[next], corners_inner[next]));
        wedge.close();
        let saved = painter.surface.save();
        painter.surface.clip_path(&wedge, true);
        match side.style {
            BorderStyle::Dashed | BorderStyle::Dotted if rounded => {
                let width = widths[index];
                let path = stroke_path(b, areas, width);
                let length = f64::from(path.length());
                painter.surface.draw_path(
                    &path,
                    &dash_paint(side.style, width, colors[index], length, true),
                );
            }
            BorderStyle::Dashed | BorderStyle::Dotted => {
                let width = widths[index];
                let (from, to) = side_line(b, index, width);
                let length = (to.0 - from.0).hypot(to.1 - from.1);
                let mut path = Path::new();
                path.move_to(point(from.0, from.1));
                path.line_to(point(to.0, to.1));
                painter.surface.draw_path(
                    &path,
                    &dash_paint(side.style, width, colors[index], length, true),
                );
            }
            _ => {
                painter
                    .surface
                    .draw_nested_round_rect(outer, inner, &Paint::filled(colors[index]));
            }
        }
        painter.surface.restore_to(saved);
    }
}

fn stroke_path(b: Rect, areas: &Areas, width: f64) -> Path {
    let mut path = Path::new();
    path.add_round_rect(round_rect(
        outset(b, -width / 2.0),
        &spread(&areas.outer, -width / 2.0),
    ));
    path
}

fn side_line(b: Rect, index: usize, width: f64) -> ((f64, f64), (f64, f64)) {
    let half = width / 2.0;
    let (left, top, right, bottom) = (b.x, b.y, b.x + b.width, b.y + b.height);
    match index {
        0 => ((left, top + half), (right, top + half)),
        1 => ((right - half, top), (right - half, bottom)),
        2 => ((right, bottom - half), (left, bottom - half)),
        _ => ((left + half, bottom), (left + half, top)),
    }
}

fn best_dash_gap(length: f64, dash: f64, gap: f64, closed: bool) -> f64 {
    let available = if closed { length } else { length + gap };
    let fewest = (available / (dash + gap)).floor();
    let most = fewest + 1.0;
    let (fewest_gaps, most_gaps) = if closed {
        (fewest, most)
    } else {
        (fewest - 1.0, most - 1.0)
    };
    let small = fewest.mul_add(-dash, length) / fewest_gaps;
    let large = most.mul_add(-dash, length) / most_gaps;
    if large <= 0.0 || (small - gap).abs() < (large - gap).abs() {
        small
    } else {
        large
    }
}

fn dash_paint(style: BorderStyle, width: f64, color: Rgba, length: f64, closed: bool) -> Paint {
    let mut paint = Paint::filled(color);
    let (dash, gap, cap) = match (style, width >= THICK_BORDER) {
        (BorderStyle::Dotted, true) => (0.0, 2.0 * width, StrokeCap::Round),
        (BorderStyle::Dotted, false) => (width, width, StrokeCap::Butt),
        (_, true) => (2.0 * width, width, StrokeCap::Butt),
        (_, false) => (3.0 * width, 2.0 * width, StrokeCap::Butt),
    };
    paint.set_stroke(narrow(width), cap, StrokeJoin::Miter, 4.0);
    if style == BorderStyle::Dashed {
        if length > dash * 2.0 {
            let fitted = best_dash_gap(length, dash, gap, closed);
            paint.set_dash(&[narrow(dash), narrow(fitted)], 0.0);
        }
        return paint;
    }
    paint.set_dash(&[narrow(dash.max(0.001)), narrow(gap)], 0.0);
    paint
}

fn draw_outline(
    painter: &mut Painter<'_, '_, '_>,
    placed: &Placed,
    style: &Style,
    areas: &Areas,
    current: &Color,
) {
    let Some(outline) = prop!(style, Outline) else {
        return;
    };
    if outline.style == BorderStyle::None {
        return;
    }
    let text = &placed.text;
    let width = px(&outline.width, text, &painter.context, 0.0);
    if width <= 0.0 {
        return;
    }
    let offset = px(&outline.offset, text, &painter.context, 0.0);
    let color = resolve(outline.color.as_ref().unwrap_or(current), current).rgba();
    let grow = offset + width / 2.0;
    let shape = round_rect(outset(areas.border, grow), &spread(&areas.outer, grow));
    let mut paint = Paint::filled(color);
    paint.set_stroke(narrow(width), StrokeCap::Butt, StrokeJoin::Miter, 4.0);
    if matches!(outline.style, BorderStyle::Dashed | BorderStyle::Dotted) {
        paint.set_dash(&[narrow(width * 3.0), narrow(width * 3.0)], 0.0);
    }
    painter.surface.draw_round_rect(shape, &paint);
}

fn fitted(fit: ObjectFit, (iw, ih): (f64, f64), area: Rect) -> (f64, f64) {
    let contain = (area.width / iw).min(area.height / ih);
    let cover = (area.width / iw).max(area.height / ih);
    match fit {
        ObjectFit::Fill => (area.width, area.height),
        ObjectFit::Contain => (iw * contain, ih * contain),
        ObjectFit::Cover => (iw * cover, ih * cover),
        ObjectFit::None => (iw, ih),
        ObjectFit::ScaleDown => {
            let scale = contain.min(1.0);
            (iw * scale, ih * scale)
        }
    }
}

pub fn draw_lottie(
    painter: &mut Painter<'_, '_, '_>,
    placed: &Placed,
    style: &Style,
    frame: &opsis_scene::node::LottieFrame,
) {
    let areas = Areas::of(placed, style, painter);
    let content = areas.content;
    let fit = prop!(style, ObjectFit)
        .copied()
        .unwrap_or(ObjectFit::Contain);
    let position = prop!(style, ObjectPosition)
        .cloned()
        .unwrap_or_else(Position2::center);
    let text = placed.text.clone();
    let units_context = painter.context;
    let Painter {
        surface, resources, ..
    } = painter;
    let Some(animation) = resources.animation(&frame.source.0) else {
        return;
    };
    let (width, height) = animation.size();
    if width <= 0.0 || height <= 0.0 {
        return;
    }
    animation.seek(frame.frame.0);
    let (w, h) = fitted(fit, (f64::from(width), f64::from(height)), content);
    let x = content.x
        + px(
            &auto_half(&position.x),
            &text,
            &units_context,
            content.width - w,
        );
    let y = content.y
        + px(
            &auto_half(&position.y),
            &text,
            &units_context,
            content.height - h,
        );
    let saved = surface.save();
    clip_content(surface, &areas);
    surface.draw_lottie(
        animation,
        RectF::new(narrow(x), narrow(y), narrow(w), narrow(h)),
    );
    surface.restore_to(saved);
}

fn snap(value: f64) -> f64 {
    (value + 0.5).floor()
}

fn snapped(rect: Rect, (ox, oy): (f64, f64)) -> Rect {
    let x = snap(rect.x - ox);
    let y = snap(rect.y - oy);
    Rect {
        x: x + ox,
        y: y + oy,
        width: snap(rect.x - ox + rect.width) - x,
        height: snap(rect.y - oy + rect.height) - y,
    }
}

fn auto_half(length: &Length) -> Length {
    if *length == Length::Auto {
        Length::percent(50.0)
    } else {
        length.clone()
    }
}

fn clip_content(surface: &mut opsis_skia::Surface, areas: &Areas) {
    if is_rounded(&areas.content_radii) {
        surface.clip_round_rect(round_rect(areas.content, &areas.content_radii), true);
    } else {
        surface.clip_rect(rect_f(areas.content), true);
    }
}

pub fn draw_replaced(
    painter: &mut Painter<'_, '_, '_>,
    placed: &Placed,
    style: &Style,
    source: &str,
) {
    let areas = Areas::of(placed, style, painter);
    let content = areas.content;
    let fit = prop!(style, ObjectFit).copied().unwrap_or_default();
    let position = prop!(style, ObjectPosition)
        .cloned()
        .unwrap_or_else(Position2::center);
    let filtering = match prop!(style, ImageRendering) {
        Some(ImageRendering::Pixelated) => Sampling::Nearest,
        _ => Sampling::Linear,
    };
    let text = placed.text.clone();
    let units_context = painter.context;
    let device = painter.placement();
    let Painter {
        surface, resources, ..
    } = painter;
    if Resources::is_svg(source) {
        let Some(tree) = resources.svg(source) else {
            return;
        };
        let size = tree.size();
        let intrinsic = (f64::from(size.width()), f64::from(size.height()));
        let (w, h) = fitted(fit, intrinsic, content);
        let x = content.x
            + px(
                &auto_half(&position.x),
                &text,
                &units_context,
                content.width - w,
            );
        let y = content.y
            + px(
                &auto_half(&position.y),
                &text,
                &units_context,
                content.height - h,
            );
        let saved = surface.save();
        clip_content(surface, &areas);
        let dest = snapped(
            Rect {
                x,
                y,
                width: w,
                height: h,
            },
            placed.snap_origin,
        );
        let base = placed
            .matrix
            .then(&Matrix::translation(dest.x, dest.y, 0.0))
            .then(&Matrix::scaling(
                dest.width / intrinsic.0,
                dest.height / intrinsic.1,
                1.0,
            ));
        let mut decode = |bytes: &[u8]| opsis_skia::Image::decode(bytes);
        SvgPainter {
            surface,
            decode: &mut decode,
            device,
        }
        .group(tree.root(), &base);
        surface.restore_to(saved);
        return;
    }
    let Some(image) = resources.image(source) else {
        return;
    };
    let intrinsic = (f64::from(image.width()), f64::from(image.height()));
    if intrinsic.0 <= 0.0 || intrinsic.1 <= 0.0 {
        return;
    }
    let (w, h) = fitted(fit, intrinsic, content);
    let x = content.x
        + px(
            &auto_half(&position.x),
            &text,
            &units_context,
            content.width - w,
        );
    let y = content.y
        + px(
            &auto_half(&position.y),
            &text,
            &units_context,
            content.height - h,
        );
    let saved = surface.save();
    clip_content(surface, &areas);
    let full = RectF::new(0.0, 0.0, narrow(intrinsic.0), narrow(intrinsic.1));
    let target = rect_f(snapped(
        Rect {
            x,
            y,
            width: w,
            height: h,
        },
        placed.snap_origin,
    ));
    surface.draw_image_rect(&image, (full, target), filtering, &Paint::new());
    surface.restore_to(saved);
}
