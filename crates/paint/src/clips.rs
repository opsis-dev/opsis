use std::fmt::Write;

use opsis_layout::output::{Placed, Rect};
use opsis_layout::transform::Matrix;
use opsis_scene::node::{Kind, Node, SvgChild, SvgElement};
use opsis_scene::paint::{ClipPath, FillRule, ShapeRadius};
use opsis_scene::values::Length;
use opsis_skia::Path;

use crate::Painter;
use crate::geometry::{point, px, radii, round_rect};
use crate::svg::SvgPainter;

const SVG_NAMESPACE: &str = "http://www.w3.org/2000/svg";

fn escaped(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn write(element: &SvgElement, root: bool, size: Option<(f64, f64)>, out: &mut String) {
    out.push('<');
    out.push_str(&element.tag);
    if root && !element.attributes.iter().any(|(k, _)| k == "xmlns") {
        let _ = write!(out, " xmlns=\"{SVG_NAMESPACE}\"");
    }
    for (name, value) in &element.attributes {
        if root && size.is_some() && (name == "width" || name == "height") {
            continue;
        }
        let _ = write!(out, " {name}=\"{}\"", escaped(value));
    }
    if root && let Some((w, h)) = size {
        let _ = write!(out, " width=\"{w}\" height=\"{h}\"");
    }
    out.push('>');
    for child in &element.children {
        match child {
            SvgChild::Element(inner) => write(inner, false, None, out),
            SvgChild::Text(text) => out.push_str(&escaped(text)),
            SvgChild::Html(_) => {}
        }
    }
    out.push_str("</");
    out.push_str(&element.tag);
    out.push('>');
}

pub fn markup(element: &SvgElement, size: Option<(f64, f64)>) -> String {
    let mut out = String::new();
    write(element, true, size, &mut out);
    out
}

fn find_in_svg<'n>(element: &'n SvgElement, id: &str) -> Option<&'n SvgElement> {
    if element.attributes.iter().any(|(k, v)| k == "id" && v == id) {
        return Some(element);
    }
    element.children.iter().find_map(|child| match child {
        SvgChild::Element(inner) => find_in_svg(inner, id),
        _ => None,
    })
}

pub fn find_by_id<'n>(node: &'n Node, id: &str) -> Option<&'n SvgElement> {
    let Node::Element(element) = node else {
        return None;
    };
    if let Kind::Svg(svg) = &element.kind
        && let Some(found) = find_in_svg(svg, id)
    {
        return Some(found);
    }
    element
        .children
        .iter()
        .find_map(|child| find_by_id(child, id))
}

fn shape_radius(
    radius: &ShapeRadius,
    distances: [f64; 2],
    basis: f64,
    painter: &Painter<'_, '_, '_>,
    placed: &Placed,
) -> f64 {
    match radius {
        ShapeRadius::ClosestSide => distances[0],
        ShapeRadius::FarthestSide => distances[1],
        ShapeRadius::Length(l) => px(l, &placed.text, &painter.context, basis),
    }
}

fn shape(painter: &Painter<'_, '_, '_>, placed: &Placed, clip: &ClipPath) -> Option<Path> {
    let b = placed.border_box;
    let text = &placed.text;
    let at = |l: &Length, basis: f64| px(l, text, &painter.context, basis);
    let mut path = Path::new();
    match clip {
        ClipPath::Inset { edges, round } => {
            let top = at(&edges.top, b.height);
            let right = at(&edges.right, b.width);
            let bottom = at(&edges.bottom, b.height);
            let left = at(&edges.left, b.width);
            let rect = Rect {
                x: b.x + left,
                y: b.y + top,
                width: (b.width - left - right).max(0.0),
                height: (b.height - top - bottom).max(0.0),
            };
            let r = radii(round.as_ref(), rect, text, &painter.context);
            path.add_round_rect(round_rect(rect, &r));
        }
        ClipPath::Circle { radius, center } => {
            let cx = b.x + at(&center.x, b.width);
            let cy = b.y + at(&center.y, b.height);
            let dx = [cx - b.x, b.x + b.width - cx];
            let dy = [cy - b.y, b.y + b.height - cy];
            let closest = dx[0].min(dx[1]).min(dy[0]).min(dy[1]);
            let farthest = dx[0].max(dx[1]).max(dy[0]).max(dy[1]);
            let basis = b.width.hypot(b.height) / std::f64::consts::SQRT_2;
            let r = shape_radius(radius, [closest, farthest], basis, painter, placed);
            path.add_oval(crate::geometry::rect_f(Rect {
                x: cx - r,
                y: cy - r,
                width: 2.0 * r,
                height: 2.0 * r,
            }));
        }
        ClipPath::Ellipse {
            radii: (rx, ry),
            center,
        } => {
            let cx = b.x + at(&center.x, b.width);
            let cy = b.y + at(&center.y, b.height);
            let dx = [cx - b.x, b.x + b.width - cx];
            let dy = [cy - b.y, b.y + b.height - cy];
            let rx = shape_radius(
                rx,
                [dx[0].min(dx[1]), dx[0].max(dx[1])],
                b.width,
                painter,
                placed,
            );
            let ry = shape_radius(
                ry,
                [dy[0].min(dy[1]), dy[0].max(dy[1])],
                b.height,
                painter,
                placed,
            );
            path.add_oval(crate::geometry::rect_f(Rect {
                x: cx - rx,
                y: cy - ry,
                width: 2.0 * rx,
                height: 2.0 * ry,
            }));
        }
        ClipPath::Polygon { fill, points } => {
            for (i, (x, y)) in points.iter().enumerate() {
                let p = point(b.x + at(x, b.width), b.y + at(y, b.height));
                if i == 0 {
                    path.move_to(p);
                } else {
                    path.line_to(p);
                }
            }
            path.close();
            path.set_even_odd(*fill == FillRule::Evenodd);
        }
        ClipPath::Path { .. } | ClipPath::Url(_) => return None,
    }
    Some(path)
}

fn svg_clip(painter: &mut Painter<'_, '_, '_>, placed: &Placed, document: String) {
    let b = placed.border_box;
    let base = placed.matrix.then(&Matrix::translation(b.x, b.y, 0.0));
    let Painter {
        surface, resources, ..
    } = painter;
    let Some(tree) = resources.markup(document) else {
        return;
    };
    let Some(target) = first_path(tree.root()) else {
        return;
    };
    let mut decode = |bytes: &[u8]| opsis_skia::Image::decode(bytes);
    let mut svg = SvgPainter {
        surface,
        decode: &mut decode,
    };
    match target.clip_path() {
        Some(clip) => svg.clip(clip, &base),
        None => svg.clip_to_path(target, &base),
    }
}

fn first_path(group: &usvg::Group) -> Option<&usvg::Group> {
    for child in group.children() {
        if let usvg::Node::Group(inner) = child {
            if inner.clip_path().is_some() {
                return Some(inner);
            }
            if let Some(found) = first_path(inner) {
                return Some(found);
            }
        }
    }
    None
}

pub fn filter_document(definition: &SvgElement, id: &str, (w, h): (f64, f64)) -> String {
    let mut defs = String::new();
    write(definition, false, None, &mut defs);
    format!(
        "<svg xmlns=\"{SVG_NAMESPACE}\" width=\"{w}\" height=\"{h}\"><defs>{defs}</defs><g filter=\"url(#{id})\"><rect width=\"{w}\" height=\"{h}\"/></g></svg>"
    )
}

pub fn apply(painter: &mut Painter<'_, '_, '_>, placed: &Placed, clip: &ClipPath, root: &Node) {
    let b = placed.border_box;
    match clip {
        ClipPath::Url(reference) => {
            let id = reference.trim_start_matches('#');
            let Some(definition) = find_by_id(root, id) else {
                return;
            };
            let mut defs = String::new();
            write(definition, false, None, &mut defs);
            let document = format!(
                "<svg xmlns=\"{SVG_NAMESPACE}\" width=\"{w}\" height=\"{h}\"><defs>{defs}</defs><g clip-path=\"url(#{id})\"><rect width=\"{w}\" height=\"{h}\"/></g></svg>",
                w = b.width,
                h = b.height,
            );
            svg_clip(painter, placed, document);
        }
        ClipPath::Path { fill, data } => {
            let rule = if *fill == FillRule::Evenodd {
                "evenodd"
            } else {
                "nonzero"
            };
            let document = format!(
                "<svg xmlns=\"{SVG_NAMESPACE}\" width=\"{w}\" height=\"{h}\"><defs><clipPath id=\"c\"><path clip-rule=\"{rule}\" d=\"{d}\"/></clipPath></defs><g clip-path=\"url(#c)\"><rect width=\"{w}\" height=\"{h}\"/></g></svg>",
                w = b.width,
                h = b.height,
                d = escaped(data),
            );
            svg_clip(painter, placed, document);
        }
        other => {
            if let Some(path) = shape(painter, placed, other) {
                painter.surface.clip_path(&path, true);
            }
        }
    }
}
