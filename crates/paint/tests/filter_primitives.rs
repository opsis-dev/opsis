use opsis_layout::fonts::Fonts;
use opsis_layout::{Images, Viewport, lay_out};
use opsis_paint::{Resources, paint};
use opsis_scene::node::{Node, SvgElement, div, svg};
use opsis_scene::style::{Position, Style};
use opsis_scene::values::Length;
use opsis_skia::{Rgba, Surface};

const SIDE: f64 = 100.0;

fn filled() -> Style {
    Style::default()
        .position(Position::Absolute)
        .left(Length::ZERO)
        .top(Length::ZERO)
        .width(Length::px(SIDE))
        .height(Length::px(SIDE))
}

fn square(filter: Option<SvgElement>) -> Node {
    let mut root = SvgElement::new("svg")
        .attr("width", SIDE)
        .attr("height", SIDE);
    let mut shape = SvgElement::new("rect")
        .attr("x", 30.0)
        .attr("y", 30.0)
        .attr("width", 40.0)
        .attr("height", 40.0)
        .attr("fill", "#3366cc");
    if let Some(filter) = filter {
        root = root.child(SvgElement::new("defs").child(filter.attr("id", "f")));
        shape = shape.attr("filter", "url(#f)");
    }
    div(filled()).child(svg(root.child(shape), filled())).into()
}

fn filter(primitive: SvgElement) -> SvgElement {
    SvgElement::new("filter")
        .attr("x", 0.0)
        .attr("y", 0.0)
        .attr("width", SIDE)
        .attr("height", SIDE)
        .attr("filterUnits", "userSpaceOnUse")
        .attr("color-interpolation-filters", "sRGB")
        .child(primitive)
}

fn drawn(root: &Node) -> Vec<u8> {
    let files = |_: &str| None;
    let mut fonts = Fonts::new();
    let viewport = Viewport {
        width: SIDE,
        height: SIDE,
    };
    let laid = lay_out(root, viewport, &mut fonts, &Images { size_of: &|_| None });
    let mut surface = Surface::new(100, 100).unwrap();
    surface.clear(Rgba::TRANSPARENT);
    let mut resources = Resources::new(&files, fonts.faces());
    paint(root, &laid, viewport, &mut surface, &mut resources);
    surface.rgba()
}

fn covered(rgba: &[u8]) -> usize {
    rgba.as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[3] > 127)
        .count()
}

const fn pixel(rgba: &[u8], x: usize, y: usize) -> [u8; 4] {
    let at = (y * 100 + x) * 4;
    [rgba[at], rgba[at + 1], rgba[at + 2], rgba[at + 3]]
}

#[test]
fn morphology_grows_and_shrinks_the_shape() {
    let plain = covered(&drawn(&square(None)));
    let morph = |operator: &str| {
        covered(&drawn(&square(Some(filter(
            SvgElement::new("feMorphology")
                .attr("operator", operator)
                .attr("radius", 5.0),
        )))))
    };
    assert!(morph("dilate") > plain);
    assert!(morph("erode") < plain);
}

#[test]
fn a_component_table_inverts_the_colour() {
    let inverted = drawn(&square(Some(filter(
        SvgElement::new("feComponentTransfer")
            .child(
                SvgElement::new("feFuncR")
                    .attr("type", "table")
                    .attr("tableValues", "1 0"),
            )
            .child(
                SvgElement::new("feFuncG")
                    .attr("type", "table")
                    .attr("tableValues", "1 0"),
            )
            .child(
                SvgElement::new("feFuncB")
                    .attr("type", "table")
                    .attr("tableValues", "1 0"),
            ),
    ))));
    let [r, g, b, a] = pixel(&inverted, 50, 50);
    assert_eq!(a, 255);
    assert!(r.abs_diff(0xcc) <= 2 && g.abs_diff(0x99) <= 2 && b.abs_diff(0x33) <= 2);
}

#[test]
fn an_identity_kernel_keeps_the_picture() {
    let kept = drawn(&square(Some(filter(
        SvgElement::new("feConvolveMatrix")
            .attr("order", "3")
            .attr("kernelMatrix", "0 0 0 0 1 0 0 0 0"),
    ))));
    assert_eq!(pixel(&kept, 50, 50), pixel(&drawn(&square(None)), 50, 50));
}

#[test]
fn a_tile_repeats_its_input_over_the_region() {
    let tiled = drawn(&square(Some(
        SvgElement::new("filter")
            .attr("x", 0.0)
            .attr("y", 0.0)
            .attr("width", SIDE)
            .attr("height", SIDE)
            .attr("filterUnits", "userSpaceOnUse")
            .attr("primitiveUnits", "userSpaceOnUse")
            .child(
                SvgElement::new("feOffset")
                    .attr("x", 30.0)
                    .attr("y", 30.0)
                    .attr("width", 20.0)
                    .attr("height", 20.0)
                    .attr("result", "piece"),
            )
            .child(SvgElement::new("feTile").attr("in", "piece")),
    )));
    assert!(pixel(&tiled, 5, 5)[3] > 127);
}

#[test]
fn lighting_lights_the_surface() {
    let lit = |name: &str| {
        drawn(&square(Some(filter(
            SvgElement::new(name)
                .attr("lighting-color", "#ffffff")
                .attr("surfaceScale", 2.0)
                .child(
                    SvgElement::new("feDistantLight")
                        .attr("azimuth", 45.0)
                        .attr("elevation", 45.0),
                ),
        ))))
    };
    assert!(pixel(&lit("feDiffuseLighting"), 50, 50)[0] > 100);
    assert!(covered(&lit("feSpecularLighting")) > 0);
}
