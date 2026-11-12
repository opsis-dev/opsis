use opsis_scene::css::with_background;
use opsis_scene::node::{Node, SvgElement, div, svg};
use opsis_scene::paint::BlendMode;
use opsis_scene::style::{Align, Display, Position, Style};
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::Length;

const GRAIN_FREQUENCY: f64 = 0.9;
pub const GRAIN_SEED: u32 = 3;
const TRANSPARENT: &str = "transparent";

pub fn grain(amount: f64, size: f64, seed: u32) -> Option<Node> {
    if amount <= 0.0 {
        return None;
    }
    let id = format!("grain-{seed}");
    let filter = SvgElement::new("filter")
        .attr("id", id.as_str())
        .child(
            SvgElement::new("feTurbulence")
                .attr("type", "fractalNoise")
                .attr("baseFrequency", GRAIN_FREQUENCY / size)
                .attr("numOctaves", 2)
                .attr("seed", seed)
                .attr("stitchTiles", "stitch"),
        )
        .child(
            SvgElement::new("feColorMatrix")
                .attr("type", "saturate")
                .attr("values", "0"),
        );
    let rect = SvgElement::new("rect")
        .attr("width", "100%")
        .attr("height", "100%")
        .attr("filter", format!("url(#{id})"));
    let root = SvgElement::new("svg")
        .attr("width", "100%")
        .attr("height", "100%")
        .child(filter)
        .child(rect);
    Some(
        div(absolute_fill()
            .opacity(amount)
            .mix_blend_mode(BlendMode::Overlay))
        .child(svg(root, Style::default()))
        .into(),
    )
}

pub fn visible_background(background: &str) -> Option<&str> {
    let background = background.trim();
    (!background.is_empty() && background != TRANSPARENT).then_some(background)
}

pub fn backgrounded(background: &str, children: Vec<Node>) -> Node {
    let style = visible_background(background)
        .map_or_else(absolute_fill, |b| with_background(absolute_fill(), b));
    div(style).children(children).into()
}

pub fn centered_stage(background: &str) -> Style {
    with_background(
        Style::default()
            .position(Position::Absolute)
            .inset(Length::ZERO)
            .display(Display::Flex)
            .align_items(Align::Center)
            .justify_content(Align::Center),
        background,
    )
}
