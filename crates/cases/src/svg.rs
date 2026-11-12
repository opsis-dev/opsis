use opsis_scene::node::{Node, SvgElement, svg};
use opsis_scene::style::{Position, Style};
use opsis_scene::values::Length;

use crate::stage;

fn element(node: roxmltree::Node<'_, '_>, root: bool) -> SvgElement {
    let mut out = SvgElement::new(node.tag_name().name());
    for attribute in node.attributes() {
        if root && attribute.name() == "style" {
            continue;
        }
        out = out.attr(attribute.name(), attribute.value());
    }
    for child in node.children() {
        if child.is_element() {
            out = out.child(element(child, false));
        } else if let Some(text) = child.text()
            && !text.is_empty()
        {
            out = out.text(text);
        }
    }
    out
}

pub fn parse(markup: &str) -> SvgElement {
    let document = roxmltree::Document::parse(markup).expect("a case's SVG markup");
    element(document.root_element(), true)
}

fn filling(markup: &str) -> Node {
    stage(vec![
        svg(
            parse(markup),
            Style::default()
                .position(Position::Absolute)
                .inset(Length::ZERO),
        )
        .into(),
    ])
}

pub const MARKUPS: &[(&str, &str)] = &[
    ("svg/clip-path", include_str!("../svg/svg__clip-path.svg")),
    ("svg/fill-rule", include_str!("../svg/svg__fill-rule.svg")),
    ("svg/gradients", include_str!("../svg/svg__gradients.svg")),
    (
        "svg/group-transform",
        include_str!("../svg/svg__group-transform.svg"),
    ),
    ("svg/mask", include_str!("../svg/svg__mask.svg")),
    ("svg/path", include_str!("../svg/svg__path.svg")),
    ("svg/pattern", include_str!("../svg/svg__pattern.svg")),
    ("svg/shapes", include_str!("../svg/svg__shapes.svg")),
    (
        "svg/stroke-dash",
        include_str!("../svg/svg__stroke-dash.svg"),
    ),
    ("svg/stroke", include_str!("../svg/svg__stroke.svg")),
    ("svg/text", include_str!("../svg/svg__text.svg")),
    (
        "svg-filters/fe-blend",
        include_str!("../svg/svg-filters__fe-blend.svg"),
    ),
    (
        "svg-filters/fe-color-matrix",
        include_str!("../svg/svg-filters__fe-color-matrix.svg"),
    ),
    (
        "svg-filters/fe-displacement-map",
        include_str!("../svg/svg-filters__fe-displacement-map.svg"),
    ),
    (
        "svg-filters/fe-gaussian-blur",
        include_str!("../svg/svg-filters__fe-gaussian-blur.svg"),
    ),
    (
        "svg-filters/fe-image",
        include_str!("../svg/svg-filters__fe-image.svg"),
    ),
    (
        "svg-filters/fe-merge",
        include_str!("../svg/svg-filters__fe-merge.svg"),
    ),
    (
        "svg-filters/fe-offset",
        include_str!("../svg/svg-filters__fe-offset.svg"),
    ),
    (
        "svg-filters/fe-turbulence",
        include_str!("../svg/svg-filters__fe-turbulence.svg"),
    ),
];

pub fn scene(id: &str) -> Option<Node> {
    MARKUPS
        .iter()
        .find(|(name, _)| *name == id)
        .map(|(_, markup)| filling(markup))
}

const ART: &str = "<rect x=\"0\" y=\"0\" width=\"100\" height=\"50\" fill=\"#f97316\"/><circle cx=\"50\" cy=\"25\" r=\"20\" fill=\"#0ea5e9\"/><path d=\"M0 50 L50 0 L100 50\" fill=\"none\" stroke=\"#f8fafc\" stroke-width=\"3\"/>";

pub fn view_box() -> Node {
    let ratios = [
        "xMidYMid meet",
        "xMinYMin meet",
        "xMaxYMax meet",
        "xMidYMid slice",
        "none",
        "xMinYMid slice",
    ];
    let bg = || Style::default().background_color(crate::slate());
    let mut cells: Vec<Node> = ratios
        .iter()
        .enumerate()
        .map(|(i, ratio)| {
            crate::d(crate::cell(i, &bg()))
                .child(svg(
                    parse(&format!(
                        "<svg width=\"136\" height=\"148\" viewBox=\"0 0 100 50\" preserveAspectRatio=\"{ratio}\">{ART}</svg>"
                    )),
                    Style::default(),
                ))
                .into()
        })
        .collect();
    cells.push(
        crate::d(crate::cell(6, &bg()))
            .child(svg(
                parse(&format!("<svg viewBox=\"-50 -25 200 100\">{ART}</svg>")),
                Style::default()
                    .width(crate::pct(100.0))
                    .height(crate::pct(100.0)),
            ))
            .into(),
    );
    cells.push(
        crate::d(crate::cell(7, &bg()))
            .child(svg(
                parse(&format!(
                    "<svg width=\"136\" height=\"148\" viewBox=\"0 0 10 5\" overflow=\"visible\"><g transform=\"scale(0.1)\">{ART}</g></svg>"
                )),
                Style::default(),
            ))
            .into(),
    );
    stage(cells)
}

const CSS_FILTERS: &str = "<svg width=\"0\" height=\"0\"><defs><filter id=\"oracle-rough\"><feTurbulence type=\"fractalNoise\" baseFrequency=\"0.04\" numOctaves=\"3\" seed=\"2\" result=\"noise\"/><feDisplacementMap in=\"SourceGraphic\" in2=\"noise\" scale=\"8\" xChannelSelector=\"R\" yChannelSelector=\"G\"/></filter><filter id=\"oracle-grain\"><feTurbulence type=\"fractalNoise\" baseFrequency=\"0.8\" numOctaves=\"2\" seed=\"5\" result=\"grain\"/><feColorMatrix in=\"grain\" type=\"saturate\" values=\"0\" result=\"grey\"/><feBlend in=\"SourceGraphic\" in2=\"grey\" mode=\"overlay\"/></filter><filter id=\"oracle-split\"><feOffset in=\"SourceGraphic\" dx=\"-3\" result=\"left\"/><feColorMatrix in=\"left\" type=\"matrix\" values=\"1 0 0 0 0  0 0 0 0 0  0 0 0 0 0  0 0 0 1 0\" result=\"red\"/><feColorMatrix in=\"SourceGraphic\" type=\"matrix\" values=\"0 0 0 0 0  0 1 0 0 0  0 0 1 0 0  0 0 0 1 0\" result=\"rest\"/><feBlend in=\"red\" in2=\"rest\" mode=\"screen\"/></filter></defs></svg>";

pub fn css_url_filter() -> Node {
    use opsis_scene::paint::{Filter, Shadow};
    use opsis_scene::style::LineHeight;
    use opsis_scene::values::Num;

    use crate::paints::{hex, layer, linear, stop};

    let url = |id: &str| Filter::Url(format!("#{id}"));
    stage(vec![
        svg(
            parse(CSS_FILTERS),
            Style::default().position(Position::Absolute),
        )
        .into(),
        crate::d(crate::cell(
            0,
            &Style::default()
                .filter(vec![url("oracle-rough")])
                .background_color(hex(0xf9_7316))
                .border_radius_all(24.0),
        ))
        .into(),
        crate::d(crate::cell(
            1,
            &Style::default()
                .filter(vec![url("oracle-grain")])
                .background(vec![layer(linear(
                    180.0,
                    vec![stop(hex(0x0e_a5e9)), stop(hex(0x7c_3aed))],
                ))]),
        ))
        .into(),
        crate::d(crate::cell(
            2,
            &Style::default()
                .filter(vec![url("oracle-split")])
                .font_size(28.0)
                .font_weight(900.0)
                .line_height(LineHeight::Number(Num(1.1))),
        ))
        .child("Sphinx of black quartz")
        .into(),
        crate::d(crate::cell(
            3,
            &Style::default()
                .filter(vec![
                    url("oracle-rough"),
                    Filter::DropShadow(Shadow {
                        inset: false,
                        x: Length::ZERO,
                        y: Length::px(4.0),
                        blur: Length::px(4.0),
                        spread: Length::ZERO,
                        color: hex(0x00_0000),
                    }),
                ])
                .font_size(28.0)
                .font_weight(900.0)
                .color(hex(0x22_c55e)),
        ))
        .child("Wobbly")
        .into(),
    ])
}
