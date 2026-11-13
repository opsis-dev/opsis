use opsis_macros::settings;
use opsis_scene::node::{Node, div};
use opsis_scene::paint::Filter;
use opsis_scene::style::{Position, Style};
use opsis_scene::timeline::{ChildAt, absolute_fill};
use opsis_scene::values::Length;
use opsis_types::looks::NodeLook;

use crate::elements::draw::{Cx, DrawFailure, css_color};
use crate::elements::entry::{element, meta};
use crate::elements::shader::{generated_from, quads, value};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};
use crate::nodes::image::RedactStyle;

#[settings(export)]
pub struct RedactBox {
    #[setting(range(min = 0, max = 1))]
    pub x: f64,
    #[setting(range(min = 0, max = 1))]
    pub y: f64,
    #[setting(range(min = 0, max = 1))]
    pub width: f64,
    #[setting(range(min = 0, max = 1))]
    pub height: f64,
}

#[settings(export)]
pub struct Redact {
    #[setting(
        "Areas to cover",
        default = vec![RedactBox { x: 0.62, y: 0.06, width: 0.3, height: 0.08 }],
        length(max = 8),
        widget = "boxes",
        time_key = "frameMs",
        help = "Drag on the picture to add an area. The areas stay in place on screen; they don't follow the camera. To hide something on a recording while the camera moves, use Redact take on the recording instead."
    )]
    pub boxes: Vec<RedactBox>,
    #[setting(
        "Frame to draw on",
        range(min = 0, max = 3_600_000),
        step = 1,
        unit = "ms",
        hidden
    )]
    pub frame_ms: u32,
    #[setting("Hide with")]
    pub style: RedactStyle,
    #[setting(
        "Pixel size",
        default = 24,
        range(min = 6, max = 72),
        step = 2,
        unit = "px",
        show_if = { "style": ["pixelate"] }
    )]
    pub block_size: u32,
    #[setting(
        "Blur",
        default = 16.0,
        range(min = 1, max = 100),
        step = 0.5,
        unit = "px",
        show_if = { "style": ["blur"] }
    )]
    pub blur: f64,
    #[setting(
        "Colour",
        default = "#000000",
        widget = "color",
        check = colour,
        show_if = { "style": ["solid"] }
    )]
    pub color: String,
    #[setting(
        "Corner radius",
        range(min = 0, max = 200),
        step = 1,
        unit = "px",
        show_if = { "style": ["solid"] },
        advanced
    )]
    pub radius: f64,
}

pub fn definition() -> ElementMeta {
    meta::<Redact>(
        element(
            ElementKind::Wrapper,
            "redact",
            "Redact",
            "Hides parts of the screen with large pixels, a blur or a solid colour, to keep keys, emails and personal details out of the film.",
            ElementCategory::Effect,
            90,
        )
        .look(NodeLook::of("lucide:grid-3x3")),
    )
}

const MAX_REGIONS: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Region {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

fn cover(region: Region, style: Style) -> Node {
    div(style
        .position(Position::Absolute)
        .left(Length::px(region.x))
        .top(Length::px(region.y))
        .width(Length::px(region.width))
        .height(Length::px(region.height)))
    .into()
}

pub fn wrap(props: &Redact, child: ChildAt<'_>, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let inner = child(cx.at.stage)?;
    let (width, height) = (cx.at.stage.width, cx.at.stage.height);
    let regions: Vec<Region> = props
        .boxes
        .iter()
        .filter(|b| b.width > 0.0 && b.height > 0.0)
        .take(MAX_REGIONS)
        .map(|b| Region {
            x: b.x * width,
            y: b.y * height,
            width: b.width * width,
            height: b.height * height,
        })
        .collect();
    if regions.is_empty() {
        return Ok(inner);
    }
    if props.style == RedactStyle::Pixelate {
        let cell = f64::from(props.block_size);
        let packed: Vec<f64> = regions
            .iter()
            .flat_map(|r| {
                [
                    r.x / width,
                    r.y / height,
                    r.width / width,
                    r.height / height,
                ]
            })
            .collect();
        return Ok(generated_from(
            "redact-pixelate",
            vec![
                quads("regions", packed),
                value(
                    "regionCount",
                    vec![opsis_types::numbers::float(regions.len())],
                ),
                value("cell", vec![cell / width, cell / height]),
            ],
            inner,
        ));
    }
    let covers = regions.iter().map(|r| {
        let style = if props.style == RedactStyle::Blur {
            Style::default().backdrop_filter(vec![Filter::Blur(Length::px(props.blur))])
        } else {
            Style::default()
                .background_color(css_color(&props.color))
                .border_radius_all(props.radius)
        };
        cover(*r, style)
    });
    Ok(div(absolute_fill())
        .child(inner)
        .child(div(absolute_fill()).children(covers))
        .into())
}
