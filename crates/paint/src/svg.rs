use opsis_layout::transform::Matrix;
use opsis_skia::{
    BlendMode, Gradient as SkiaGradient, GradientSpace, Image, ImageFilter, Matrix33, Paint, Path,
    PointF, RectF, Rgba, Sampling, Shader, StrokeCap, StrokeJoin, Surface, TileMode,
};
use usvg::tiny_skia_path::PathSegment;

pub fn matrix_of(t: usvg::Transform) -> Matrix {
    Matrix([
        f64::from(t.sx),
        f64::from(t.ky),
        0.0,
        0.0,
        f64::from(t.kx),
        f64::from(t.sy),
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
        0.0,
        f64::from(t.tx),
        f64::from(t.ty),
        0.0,
        1.0,
    ])
}

const fn matrix33(t: usvg::Transform) -> Matrix33 {
    Matrix33::new([t.sx, t.kx, t.tx, t.ky, t.sy, t.ty, 0.0, 0.0, 1.0])
}

pub fn path_of(data: &usvg::tiny_skia_path::Path) -> Path {
    let mut path = Path::new();
    for segment in data.segments() {
        match segment {
            PathSegment::MoveTo(p) => path.move_to(PointF::new(p.x, p.y)),
            PathSegment::LineTo(p) => path.line_to(PointF::new(p.x, p.y)),
            PathSegment::QuadTo(c, p) => {
                path.quad_to(PointF::new(c.x, c.y), PointF::new(p.x, p.y));
            }
            PathSegment::CubicTo(a, b, p) => path.cubic_to(
                PointF::new(a.x, a.y),
                PointF::new(b.x, b.y),
                PointF::new(p.x, p.y),
            ),
            PathSegment::Close => path.close(),
        }
    }
    path
}

pub const fn blend_of(mode: usvg::BlendMode) -> BlendMode {
    match mode {
        usvg::BlendMode::Normal => BlendMode::SrcOver,
        usvg::BlendMode::Multiply => BlendMode::Multiply,
        usvg::BlendMode::Screen => BlendMode::Screen,
        usvg::BlendMode::Overlay => BlendMode::Overlay,
        usvg::BlendMode::Darken => BlendMode::Darken,
        usvg::BlendMode::Lighten => BlendMode::Lighten,
        usvg::BlendMode::ColorDodge => BlendMode::ColorDodge,
        usvg::BlendMode::ColorBurn => BlendMode::ColorBurn,
        usvg::BlendMode::HardLight => BlendMode::HardLight,
        usvg::BlendMode::SoftLight => BlendMode::SoftLight,
        usvg::BlendMode::Difference => BlendMode::Difference,
        usvg::BlendMode::Exclusion => BlendMode::Exclusion,
        usvg::BlendMode::Hue => BlendMode::Hue,
        usvg::BlendMode::Saturation => BlendMode::Saturation,
        usvg::BlendMode::Color => BlendMode::Color,
        usvg::BlendMode::Luminosity => BlendMode::Luminosity,
        usvg::BlendMode::PlusLighter => BlendMode::Plus,
    }
}

fn color_of(color: usvg::Color, opacity: f32) -> Rgba {
    Rgba::new(
        f32::from(color.red) / 255.0,
        f32::from(color.green) / 255.0,
        f32::from(color.blue) / 255.0,
        opacity,
    )
}

fn stops(base: &usvg::BaseGradient, opacity: f32) -> (Vec<Rgba>, Vec<f32>) {
    base.stops()
        .iter()
        .map(|s| {
            (
                color_of(s.color(), s.opacity().get() * opacity),
                s.offset().get(),
            )
        })
        .unzip()
}

fn settings(base: &usvg::BaseGradient) -> SkiaGradient {
    let mut settings = SkiaGradient::new(GradientSpace::Destination);
    settings.premul = true;
    settings.tile = match base.spread_method() {
        usvg::SpreadMethod::Pad => TileMode::Clamp,
        usvg::SpreadMethod::Reflect => TileMode::Mirror,
        usvg::SpreadMethod::Repeat => TileMode::Repeat,
    };
    settings.local = matrix33(base.transform());
    settings
}

pub struct SvgPainter<'s> {
    pub surface: &'s mut Surface,
    pub decode: &'s mut dyn FnMut(&[u8]) -> Option<Image>,
    pub device: crate::Placement,
}

impl std::fmt::Debug for SvgPainter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SvgPainter").finish_non_exhaustive()
    }
}

impl SvgPainter<'_> {
    fn set(&mut self, base: &Matrix) {
        self.device.set(self.surface, base);
    }

    fn paint_for(&mut self, paint: &usvg::Paint, opacity: f32, base: &Matrix) -> Option<Paint> {
        let mut out = Paint::new();
        match paint {
            usvg::Paint::Color(color) => out.set_color(color_of(*color, opacity)),
            usvg::Paint::LinearGradient(g) => {
                let (colors, positions) = stops(g, opacity);
                let shader = Shader::linear(
                    PointF::new(g.x1(), g.y1()),
                    PointF::new(g.x2(), g.y2()),
                    &colors,
                    &positions,
                    settings(g),
                )?;
                out.set_shader(&shader);
            }
            usvg::Paint::RadialGradient(g) => {
                let (colors, positions) = stops(g, opacity);
                let shader = Shader::conical(
                    (PointF::new(g.fx(), g.fy()), g.fr().get()),
                    (PointF::new(g.cx(), g.cy()), g.r().get()),
                    &colors,
                    &positions,
                    settings(g),
                )?;
                out.set_shader(&shader);
            }
            usvg::Paint::Pattern(pattern) => {
                let shader = self.pattern(pattern, base)?;
                out.set_shader(&shader);
                out.set_alpha(opacity);
            }
        }
        Some(out)
    }

    fn pattern(&mut self, pattern: &usvg::Pattern, base: &Matrix) -> Option<Shader> {
        let rect = pattern.rect();
        let scale = base.0[0].hypot(base.0[1]).max(1.0);
        let width = (f64::from(rect.width()) * scale).ceil().max(1.0);
        let height = (f64::from(rect.height()) * scale).ceil().max(1.0);
        let mut tile = Surface::new(
            opsis_types::numbers::whole(width),
            opsis_types::numbers::whole(height),
        )
        .ok()?;
        tile.clear(Rgba::TRANSPARENT);
        let tile_base = Matrix::scaling(
            width / f64::from(rect.width()),
            height / f64::from(rect.height()),
            1.0,
        );
        {
            let decode = &mut *self.decode;
            let mut inner = SvgPainter {
                surface: &mut tile,
                decode,
                device: crate::Placement::DEVICE,
            };
            inner.group(pattern.root(), &tile_base);
        }
        let image = tile.snapshot()?;
        let t = pattern.transform();
        let local = matrix_of(t)
            .then(&Matrix::translation(
                f64::from(rect.x()),
                f64::from(rect.y()),
                0.0,
            ))
            .then(&Matrix::scaling(
                f64::from(rect.width()) / width,
                f64::from(rect.height()) / height,
                1.0,
            ));
        let m = &local.0;
        let local = Matrix33::new([
            opsis_types::numbers::narrow(m[0]),
            opsis_types::numbers::narrow(m[4]),
            opsis_types::numbers::narrow(m[12]),
            opsis_types::numbers::narrow(m[1]),
            opsis_types::numbers::narrow(m[5]),
            opsis_types::numbers::narrow(m[13]),
            0.0,
            0.0,
            1.0,
        ]);
        Shader::image(
            &image,
            (TileMode::Repeat, TileMode::Repeat),
            Sampling::Linear,
            local,
        )
    }

    pub fn clip(&mut self, clip: &usvg::ClipPath, base: &Matrix) {
        if let Some(inner) = clip.clip_path() {
            self.clip(inner, base);
        }
        let clip_base = base.then(&matrix_of(clip.transform()));
        let mut shape = Path::new();
        collect_paths(clip.root(), &Matrix::IDENTITY, &mut shape);
        self.set(&clip_base);
        self.surface.clip_path(&shape, true);
    }

    pub fn clip_to_path(&mut self, group: &usvg::Group, base: &Matrix) {
        let mut shape = Path::new();
        collect_paths(group, &Matrix::IDENTITY, &mut shape);
        self.set(base);
        self.surface.clip_path(&shape, true);
    }

    fn masked(&mut self, mask: &usvg::Mask, base: &Matrix) {
        let mut paint = Paint::new();
        paint.set_blend(BlendMode::DstIn);
        self.surface.reset_matrix();
        self.surface.save_layer(None, &paint);
        if let Some(inner) = mask.mask() {
            self.masked(inner, base);
        }
        let mut luminance = Paint::new();
        if mask.kind() == usvg::MaskType::Luminance {
            luminance.set_luma();
        }
        self.surface.save_layer(None, &luminance);
        self.group(mask.root(), base);
        self.surface.restore();
        self.surface.restore();
    }

    pub fn group(&mut self, group: &usvg::Group, outer: &Matrix) {
        let inner = outer.then(&matrix_of(group.transform()));
        let base = &inner;
        let saved = self.surface.save();
        if let Some(clip) = group.clip_path() {
            self.clip(clip, base);
        }
        if !group.filters().is_empty() {
            let mut chain = ImageFilter::source();
            for filter in group.filters() {
                let decode = &mut *self.decode;
                let mut render =
                    |root: &usvg::Group, rect: RectF| render_picture(root, rect, decode);
                chain = ImageFilter::compose(&crate::filters::chain(filter, &mut render), &chain);
            }
            let mut paint = Paint::new();
            paint.set_image_filter(&chain);
            self.set(base);
            self.surface.save_layer(None, &paint);
        }
        let opacity = group.opacity().get();
        let layered = opacity < 1.0
            || group.isolate()
            || group.mask().is_some()
            || group.blend_mode() != usvg::BlendMode::Normal;
        if layered {
            let mut paint = Paint::new();
            paint.set_alpha(opacity);
            paint.set_blend(blend_of(group.blend_mode()));
            self.surface.reset_matrix();
            self.surface.save_layer(None, &paint);
        }
        for child in group.children() {
            self.node(child, base);
        }
        if let Some(mask) = group.mask() {
            self.masked(mask, base);
        }
        self.surface.restore_to(saved);
    }

    fn node(&mut self, node: &usvg::Node, base: &Matrix) {
        match node {
            usvg::Node::Group(group) => self.group(group, base),
            usvg::Node::Path(path) => self.path(path, base),
            usvg::Node::Image(image) => self.image(image, base),
            usvg::Node::Text(text) => self.group(text.flattened(), base),
        }
    }

    fn path(&mut self, path: &usvg::Path, base: &Matrix) {
        if !path.is_visible() {
            return;
        }
        let mut shape = path_of(path.data());
        self.set(base);
        let fill = path.fill().and_then(|fill| {
            shape.set_even_odd(fill.rule() == usvg::FillRule::EvenOdd);
            self.paint_for(fill.paint(), fill.opacity().get(), base)
        });
        let stroke = path.stroke().and_then(|stroke| {
            let mut paint = self.paint_for(stroke.paint(), stroke.opacity().get(), base)?;
            let cap = match stroke.linecap() {
                usvg::LineCap::Butt => StrokeCap::Butt,
                usvg::LineCap::Round => StrokeCap::Round,
                usvg::LineCap::Square => StrokeCap::Square,
            };
            let join = match stroke.linejoin() {
                usvg::LineJoin::Miter | usvg::LineJoin::MiterClip => StrokeJoin::Miter,
                usvg::LineJoin::Round => StrokeJoin::Round,
                usvg::LineJoin::Bevel => StrokeJoin::Bevel,
            };
            paint.set_stroke(stroke.width().get(), cap, join, stroke.miterlimit().get());
            if let Some(dashes) = stroke.dasharray() {
                paint.set_dash(dashes, stroke.dashoffset());
            }
            Some(paint)
        });
        self.set(base);
        let stroke_first = path.paint_order() == usvg::PaintOrder::StrokeAndFill;
        if stroke_first && let Some(stroke) = &stroke {
            self.surface.draw_path(&shape, stroke);
        }
        if let Some(fill) = &fill {
            self.surface.draw_path(&shape, fill);
        }
        if !stroke_first && let Some(stroke) = &stroke {
            self.surface.draw_path(&shape, stroke);
        }
    }

    fn image(&mut self, image: &usvg::Image, base: &Matrix) {
        if !image.is_visible() {
            return;
        }
        let size = image.size();
        match image.kind() {
            usvg::ImageKind::SVG(tree) => {
                let inner = base.then(&Matrix::scaling(
                    f64::from(size.width() / tree.size().width()),
                    f64::from(size.height() / tree.size().height()),
                    1.0,
                ));
                self.group(tree.root(), &inner);
            }
            usvg::ImageKind::JPEG(bytes)
            | usvg::ImageKind::PNG(bytes)
            | usvg::ImageKind::GIF(bytes)
            | usvg::ImageKind::WEBP(bytes) => {
                let Some(picture) = (self.decode)(bytes) else {
                    return;
                };
                self.set(base);
                let full = RectF::new(
                    0.0,
                    0.0,
                    opsis_types::numbers::narrow(f64::from(picture.width())),
                    opsis_types::numbers::narrow(f64::from(picture.height())),
                );
                let target = RectF::new(0.0, 0.0, size.width(), size.height());
                self.surface.draw_image_rect(
                    &picture,
                    (full, target),
                    Sampling::Linear,
                    &Paint::new(),
                );
            }
        }
    }
}

pub fn render_picture(
    root: &usvg::Group,
    rect: RectF,
    decode: &mut dyn FnMut(&[u8]) -> Option<Image>,
) -> Option<Image> {
    let width = opsis_types::numbers::whole(f64::from(rect.width).ceil());
    let height = opsis_types::numbers::whole(f64::from(rect.height).ceil());
    let mut surface = Surface::new(width, height).ok()?;
    surface.clear(Rgba::TRANSPARENT);
    {
        let mut painter = SvgPainter {
            surface: &mut surface,
            decode,
            device: crate::Placement::DEVICE,
        };
        painter.group(
            root,
            &Matrix::translation(-f64::from(rect.left), -f64::from(rect.top), 0.0),
        );
    }
    surface.snapshot()
}

const fn matrix33_of(m: &Matrix) -> Matrix33 {
    let v = &m.0;
    Matrix33::new([
        opsis_types::numbers::narrow(v[0]),
        opsis_types::numbers::narrow(v[4]),
        opsis_types::numbers::narrow(v[12]),
        opsis_types::numbers::narrow(v[1]),
        opsis_types::numbers::narrow(v[5]),
        opsis_types::numbers::narrow(v[13]),
        0.0,
        0.0,
        1.0,
    ])
}

fn collect_paths(group: &usvg::Group, outer: &Matrix, into: &mut Path) {
    let inner = outer.then(&matrix_of(group.transform()));
    for child in group.children() {
        match child {
            usvg::Node::Group(g) => collect_paths(g, &inner, into),
            usvg::Node::Path(p) => {
                let mut shape = path_of(p.data());
                if p.fill()
                    .is_some_and(|f| f.rule() == usvg::FillRule::EvenOdd)
                {
                    shape.set_even_odd(true);
                }
                into.add_path(&shape, matrix33_of(&inner));
            }
            usvg::Node::Text(t) => collect_paths(t.flattened(), &inner, into),
            usvg::Node::Image(_) => {}
        }
    }
}
