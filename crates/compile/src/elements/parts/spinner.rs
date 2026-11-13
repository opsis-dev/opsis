use opsis_scene::node::{Node, SvgElement, svg};
use opsis_scene::paint::Transform;
use opsis_scene::style::Style;
use opsis_scene::values::{Angle, Length};

pub fn spinner(size: f64, color: &str, rotation: f64) -> Node {
    svg(
        SvgElement::new("svg")
            .attr("width", size)
            .attr("height", size)
            .attr("viewBox", "0 0 24 24")
            .attr("fill", "none")
            .child(
                SvgElement::new("circle")
                    .attr("cx", "12")
                    .attr("cy", "12")
                    .attr("r", "9")
                    .attr("stroke", color)
                    .attr("stroke-width", 2.5)
                    .attr("stroke-linecap", "round")
                    .attr("stroke-dasharray", "44")
                    .attr("stroke-dashoffset", "33"),
            ),
        Style::default()
            .width(Length::px(size))
            .height(Length::px(size))
            .transform(vec![Transform::Rotate(Angle::deg(rotation))]),
    )
    .into()
}
