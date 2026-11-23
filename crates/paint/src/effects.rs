use opsis_layout::output::Placed;
use opsis_scene::paint::Filter;
use opsis_scene::prop;
use opsis_scene::style::Style;
use opsis_skia::{ImageFilter, Paint, TileMode};
use opsis_types::numbers::narrow;

use crate::Painter;
use crate::boxes::{Areas, blend_of};
use crate::color::resolve;
use crate::geometry::{point, px, round_rect};

const IDENTITY: [f32; 20] = [
    1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    0.0,
];

fn rgb_matrix(m: [f64; 9], offset: f64) -> [f32; 20] {
    let mut out = IDENTITY;
    for row in 0..3 {
        for column in 0..3 {
            out[row * 5 + column] = narrow(m[row * 3 + column]);
        }
        out[row * 5 + 4] = narrow(offset);
    }
    out
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the colour matrices and gradient geometry follow the browser's arithmetic, one product and sum at a time, and the pictures have to match it"
)]
fn saturation(s: f64) -> [f64; 9] {
    [
        0.213 + 0.787 * s,
        0.715 - 0.715 * s,
        0.072 - 0.072 * s,
        0.213 - 0.213 * s,
        0.715 + 0.285 * s,
        0.072 - 0.072 * s,
        0.213 - 0.213 * s,
        0.715 - 0.715 * s,
        0.072 + 0.928 * s,
    ]
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the colour matrices and gradient geometry follow the browser's arithmetic, one product and sum at a time, and the pictures have to match it"
)]
fn color_matrix(filter: &Filter) -> Option<[f32; 20]> {
    Some(match filter {
        Filter::Brightness(a) => rgb_matrix([a.0, 0.0, 0.0, 0.0, a.0, 0.0, 0.0, 0.0, a.0], 0.0),
        Filter::Contrast(a) => rgb_matrix(
            [a.0, 0.0, 0.0, 0.0, a.0, 0.0, 0.0, 0.0, a.0],
            (1.0 - a.0) / 2.0,
        ),
        Filter::Saturate(s) => rgb_matrix(saturation(s.0), 0.0),
        Filter::Grayscale(a) => rgb_matrix(saturation(1.0 - a.0.clamp(0.0, 1.0)), 0.0),
        Filter::Sepia(a) => {
            let k = 1.0 - a.0.clamp(0.0, 1.0);
            rgb_matrix(
                [
                    0.393 + 0.607 * k,
                    0.769 - 0.769 * k,
                    0.189 - 0.189 * k,
                    0.349 - 0.349 * k,
                    0.686 + 0.314 * k,
                    0.168 - 0.168 * k,
                    0.272 - 0.272 * k,
                    0.534 - 0.534 * k,
                    0.131 + 0.869 * k,
                ],
                0.0,
            )
        }
        Filter::Invert(a) => {
            let a = a.0.clamp(0.0, 1.0);
            let d = 1.0 - 2.0 * a;
            rgb_matrix([d, 0.0, 0.0, 0.0, d, 0.0, 0.0, 0.0, d], a)
        }
        Filter::Opacity(a) => {
            let mut out = IDENTITY;
            out[18] = narrow(a.0.clamp(0.0, 1.0));
            out
        }
        Filter::HueRotate(angle) => {
            let (sin, cos) = angle.degrees().to_radians().sin_cos();
            rgb_matrix(
                [
                    0.213 + cos * 0.787 - sin * 0.213,
                    0.715 - cos * 0.715 - sin * 0.715,
                    0.072 - cos * 0.072 + sin * 0.928,
                    0.213 - cos * 0.213 + sin * 0.143,
                    0.715 + cos * 0.285 + sin * 0.140,
                    0.072 - cos * 0.072 - sin * 0.283,
                    0.213 - cos * 0.213 - sin * 0.787,
                    0.715 - cos * 0.715 + sin * 0.715,
                    0.072 + cos * 0.928 + sin * 0.072,
                ],
                0.0,
            )
        }
        Filter::Blur(_) | Filter::DropShadow(_) | Filter::Url(_) => return None,
    })
}

pub fn filter_chain(
    painter: &Painter<'_, '_, '_>,
    placed: &Placed,
    filters: &[Filter],
) -> ImageFilter {
    let text = &placed.text;
    let mut chain = ImageFilter::source();
    for filter in filters {
        chain = match filter {
            Filter::Blur(radius) => {
                let sigma = narrow(px(radius, text, &painter.context, 0.0).max(0.0));
                ImageFilter::blur(sigma, sigma, TileMode::Decal, &chain)
            }
            Filter::DropShadow(shadow) => {
                let dx = px(&shadow.x, text, &painter.context, 0.0);
                let dy = px(&shadow.y, text, &painter.context, 0.0);
                let blur = px(&shadow.blur, text, &painter.context, 0.0).max(0.0);
                let color = resolve(&shadow.color, &text.color).rgba();
                ImageFilter::drop_shadow(point(dx, dy), narrow(blur), color, false, &chain)
            }
            Filter::Url(reference) => match url_filter(painter, placed, reference) {
                Some(found) => ImageFilter::compose(&found, &chain),
                None => chain,
            },
            other => match color_matrix(other) {
                Some(matrix) => ImageFilter::color_matrix(&matrix, &chain),
                None => chain,
            },
        };
    }
    chain
}

fn url_filter(
    painter: &Painter<'_, '_, '_>,
    placed: &Placed,
    reference: &str,
) -> Option<ImageFilter> {
    let id = reference.trim_start_matches('#');
    let definition = crate::clips::find_by_id(painter.root, id)?;
    let b = placed.border_box;
    let document = crate::clips::filter_document(definition, id, (b.width, b.height));
    let tree = usvg::Tree::from_str(&document, painter.resources.options()).ok()?;
    let group = filtered_group(tree.root())?;
    let mut chain = ImageFilter::source();
    for filter in group.filters() {
        let mut decode = |bytes: &[u8]| opsis_skia::Image::decode(bytes);
        let mut render = |root: &usvg::Group, rect: opsis_skia::RectF| {
            crate::svg::render_picture(root, rect, &mut decode)
        };
        chain = ImageFilter::compose(&crate::filters::chain(filter, &mut render), &chain);
    }
    Some(chain)
}

fn filtered_group(group: &usvg::Group) -> Option<&usvg::Group> {
    group.children().iter().find_map(|child| match child {
        usvg::Node::Group(inner) if !inner.filters().is_empty() => Some(inner.as_ref()),
        usvg::Node::Group(inner) => filtered_group(inner),
        _ => None,
    })
}

pub fn has_mask(style: &Style) -> bool {
    prop!(style, Mask).is_some_and(|m| !m.is_empty())
}

pub fn needs_layer(style: &Style) -> bool {
    prop!(style, Opacity).is_some_and(|a| a.0 < 1.0)
        || prop!(style, Filter).is_some_and(|f| !f.is_empty())
        || prop!(style, MixBlendMode).is_some_and(|m| *m != opsis_scene::paint::BlendMode::Normal)
        || prop!(style, Isolation).copied().unwrap_or(false)
        || has_mask(style)
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Opened {
    pub filter: bool,
}

pub fn open_layer(
    painter: &mut Painter<'_, '_, '_>,
    placed: &Placed,
    style: &Style,
    root: &opsis_scene::node::Node,
) -> Opened {
    if let Some(filters) = prop!(style, BackdropFilter)
        && !filters.is_empty()
    {
        let areas = Areas::of(placed, style, painter);
        let backdrop = filter_chain(painter, placed, filters);
        let saved = painter.surface.save();
        painter
            .surface
            .clip_round_rect(round_rect(areas.border, &areas.outer), true);
        painter
            .surface
            .save_layer_backdrop(None, &Paint::new(), &backdrop);
        painter.surface.restore_to(saved);
    }
    if let Some(clip) = prop!(style, ClipPath) {
        crate::clips::apply(painter, placed, clip, root);
        painter.set_matrix(&placed.matrix);
    }
    let alpha = prop!(style, Opacity).map_or(1.0, |a| a.0.clamp(0.0, 1.0));
    let blend = prop!(style, MixBlendMode).copied().unwrap_or_default();
    if alpha < 1.0
        || blend != opsis_scene::paint::BlendMode::Normal
        || has_mask(style)
        || prop!(style, Isolation).copied().unwrap_or(false)
    {
        let mut paint = Paint::new();
        paint.set_alpha(narrow(alpha));
        paint.set_blend(blend_of(blend));
        painter.surface.save_layer(None, &paint);
    }
    let mut opened = Opened::default();
    if let Some(filters) = prop!(style, Filter)
        && !filters.is_empty()
    {
        let chain = filter_chain(painter, placed, filters);
        let mut paint = Paint::new();
        paint.set_image_filter(&chain);
        let colours_only = filters.iter().all(|f| color_matrix(f).is_some());
        let ink = colours_only.then(|| painter.ink(&placed.path)).flatten();
        let b = placed.border_box;
        let local = placed
            .matrix
            .then(&opsis_layout::transform::Matrix::translation(b.x, b.y, 0.0));
        if let Some(ink) = ink {
            painter.save_layer_over(ink, &local, &paint);
        } else {
            painter.set_matrix(&local);
            painter.surface.save_layer(None, &paint);
        }
        painter.set_matrix(&placed.matrix);
        opened.filter = true;
    }
    opened
}

pub fn close_layer(
    painter: &mut Painter<'_, '_, '_>,
    placed: &Placed,
    style: &Style,
    opened: Opened,
) {
    if opened.filter {
        painter.surface.restore();
    }
    let Some(masks) = prop!(style, Mask) else {
        return;
    };
    if masks.is_empty() {
        return;
    }
    painter.set_matrix(&placed.matrix);
    let areas = Areas::of(placed, style, painter);
    let mut keep = Paint::new();
    keep.set_blend(opsis_skia::BlendMode::DstIn);
    painter.surface.save_layer(None, &keep);
    for mask in masks.iter().rev() {
        let mut layer_paint = Paint::new();
        if mask.mode == opsis_scene::paint::MaskMode::Luminance {
            layer_paint.set_luma();
        }
        painter.surface.save_layer(None, &layer_paint);
        let layer = opsis_scene::paint::ImageLayer {
            clip: mask.layer.clip,
            ..mask.layer.clone()
        };
        crate::boxes::draw_layer(painter, placed, &layer, &areas, &placed.text.color);
        painter.surface.restore();
    }
    painter.surface.restore();
}
