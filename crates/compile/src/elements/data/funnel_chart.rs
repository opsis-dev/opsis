use opsis_macros::settings;
use opsis_scene::clock::js_round;
use opsis_scene::node::{Node, SvgElement, div, svg};
use opsis_scene::random::js_number_text;
use opsis_scene::style::{Align, Style, TextAlign};
use opsis_scene::timeline::absolute_fill;
use opsis_types::looks::NodeLook;
use opsis_types::numbers::float;

use crate::elements::anim::clamped_eased;
use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::number_text::format_number;
use crate::elements::shared::NumberFormat;
use crate::elements::text_split::easing_enter;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export, eq)]
pub struct FunnelChartStage {
    #[setting("Label", default = "Upgraded")]
    pub label: String,
    #[setting("Value", default = 1750, range(min = 0, max = 1_000_000_000), step = 1)]
    pub value: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Color", widget = "color", zero_label = "None", "type" = "string")]
    pub color: Option<String>,
}

#[settings(export)]
pub struct FunnelChart {
    #[setting(
        "Stages",
        default = vec![
            FunnelChartStage { label: "Visited the site".into(), value: 42800, ..Default::default() },
            FunnelChartStage { label: "Signed up".into(), value: 18600, ..Default::default() },
            FunnelChartStage { label: "Created a project".into(), value: 9400, ..Default::default() },
            FunnelChartStage { label: "Invited a teammate".into(), value: 4100, ..Default::default() },
            FunnelChartStage::default(),
        ],
        length(min = 1)
    )]
    pub stages: Vec<FunnelChartStage>,
    #[setting(
        "Width",
        default = 1100.0,
        help = "Overall width: label gutter, funnel and drop-off gutter."
    )]
    pub width: f64,
    #[setting(
        "Height",
        default = 560.0,
        help = "Overall height. Bands split it evenly."
    )]
    pub height: f64,
    #[setting("Gap", default = 10.0, help = "Space between bands.", advanced)]
    pub gap: f64,
    #[setting(
        "Last bar opacity",
        default = 0.5,
        step = 0.01,
        help = "Opacity of the last band; earlier bands ramp toward 1.",
        advanced
    )]
    pub tail_opacity: f64,
    #[setting(
        "Show drop-off",
        default = true,
        help = "Drop-off chip between consecutive stages."
    )]
    pub show_dropoff: bool,
    #[setting(
        "Show conversion",
        help = "Share of the first stage, printed beside each value."
    )]
    pub show_conversion: bool,
    #[setting(
        "Animation length",
        default = 22,
        range(min = 1, max = 300),
        step = 1,
        unit = "frames",
        help = "Length of one band's wipe.",
        advanced
    )]
    pub duration_in_frames: u32,
    #[setting(
        "Stagger",
        default = 12,
        unit = "frames",
        help = "Frames between one band and the next.",
        advanced
    )]
    pub stagger_in_frames: u32,
    #[setting("Color", default = "#e8b86d", check = colour)]
    pub color: String,
    #[setting("Text color", default = "#fafafa", check = colour, advanced)]
    pub ink_color: String,
    #[setting("Label color", default = "rgba(250,250,250,0.55)", check = colour, advanced)]
    pub label_color: String,
    #[setting("Drop-off color", default = "#f472b6", check = colour, advanced)]
    pub dropoff_color: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Label width",
        range(min = 0, max = 2000),
        zero_label = "Auto",
        step = 10,
        unit = "px",
        advanced,
        "type" = "integer"
    )]
    pub label_width: Option<u32>,
    #[setting(
        "Number format",
        default = NumberFormat::Compact,
        advanced
    )]
    pub number_format: NumberFormat,
    #[setting("Drop-off heading", default = "Drop-off", advanced)]
    pub dropoff_heading: String,
    #[setting("Font", widget = "font", advanced)]
    pub font_family: String,
}

pub fn definition() -> ElementMeta {
    meta::<FunnelChart>(
        element(
            ElementKind::Component,
            "funnel-chart",
            "Funnel chart",
            "A bar for each step of a funnel, narrowing to show how many people drop off.",
            ElementCategory::Data,
            120,
        )
        .design(1920, 1080)
        .look(NodeLook::of("lucide:funnel")),
    )
}

fn n(value: f64) -> String {
    js_number_text(value)
}

fn label(x: f64, y: f64, fill: &str, size: f64, weight: u32, opacity: f64) -> SvgElement {
    SvgElement::new("text")
        .attr("x", x)
        .attr("y", y)
        .attr("fill", fill)
        .attr("font-size", size)
        .attr("font-weight", weight)
        .attr("text-anchor", "end")
        .attr("dominant-baseline", "central")
        .attr("opacity", opacity)
}

#[expect(
    clippy::suboptimal_flops,
    reason = "each step rounds on its own, and the recorded goldens hold what that draws"
)]
pub fn draw(props: &FunnelChart, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let frame = cx.frame();
    let stages = &props.stages;
    let count = float(stages.len().max(1));
    let highest = stages
        .iter()
        .map(|s| f64::from(s.value))
        .fold(0.0, f64::max);
    let peak = if highest > 0.0 { highest } else { 1.0 };
    let (width, height) = (props.width, props.height);
    let band_height = (height - props.gap * (count - 1.0)) / count;
    let label_size = (band_height * 0.34).min(width * 0.028).max(12.0);
    let gutter = props.label_width.map_or(width * 0.3, f64::from);
    let dropoff_gutter = if props.show_dropoff {
        width * 0.13
    } else {
        0.0
    };
    let funnel_width = width - gutter - dropoff_gutter;
    let centre = gutter + funnel_width / 2.0;
    let band_width = |value: f64| value.max(0.0) / peak * funnel_width;
    let duration = f64::from(props.duration_in_frames);
    let stagger = f64::from(props.stagger_in_frames);
    let first = stages.first().map_or(0.0, |s| f64::from(s.value));
    let mut root = SvgElement::new("svg")
        .attr("width", width)
        .attr("height", height)
        .attr("viewBox", format!("0 0 {} {}", n(width), n(height)))
        .attr(
            "font-family",
            font_or(&props.font_family, FontRole::Main).join(", "),
        );
    for (index, stage) in stages.iter().enumerate() {
        let value = f64::from(stage.value);
        let top = float(index) * (band_height + props.gap);
        let bottom = top + band_height;
        let top_width = band_width(value);
        let next = stages.get(index + 1);
        let bottom_width = next.map_or(top_width, |s| band_width(f64::from(s.value)));
        let start = float(index) * stagger;
        let shown = clamped_eased(frame, [start, start + duration], [0.0, 1.0], easing_enter())?;
        if shown <= 0.0 {
            continue;
        }
        let reveal_bottom = top + band_height * shown;
        let reveal_width = top_width + (bottom_width - top_width) * shown;
        let path = format!(
            "M {} {} L {} {} L {} {} L {} {} Z",
            n(centre - top_width / 2.0),
            n(top),
            n(centre + top_width / 2.0),
            n(top),
            n(centre + reveal_width / 2.0),
            n(reveal_bottom),
            n(centre - reveal_width / 2.0),
            n(reveal_bottom)
        );
        let conversion = js_round(value / (if first > 0.0 { first } else { 1.0 }) * 100.0);
        let dropoff = next.map_or(0.0, |s| {
            js_round((f64::from(s.value) - value) / value * 100.0)
        });
        let dropoff_start = start + stagger + duration * 0.6;
        let dropoff_progress = clamped_eased(
            frame,
            [dropoff_start, dropoff_start + 12.0],
            [0.0, 1.0],
            easing_enter(),
        )?;
        let fill = stage.color.as_deref().unwrap_or(&props.color);
        let fade = 1.0 - float(index) / (count - 1.0).max(1.0);
        let mut figures = format_number(value, props.number_format);
        if props.show_conversion {
            figures = format!("{figures} · {}%", js_number_text(conversion));
        }
        let mut group = SvgElement::new("g")
            .child(
                SvgElement::new("path")
                    .attr("d", path)
                    .attr("fill", fill)
                    .attr(
                        "opacity",
                        props.tail_opacity + (1.0 - props.tail_opacity) * fade,
                    ),
            )
            .child(
                label(
                    gutter - label_size,
                    top + band_height * 0.36,
                    &props.ink_color,
                    label_size,
                    600,
                    ((shown - 0.25) / 0.35).clamp(0.0, 1.0),
                )
                .text(stage.label.as_str()),
            )
            .child(
                label(
                    gutter - label_size,
                    top + band_height * 0.72,
                    &props.label_color,
                    label_size * 0.92,
                    600,
                    ((shown - 0.45) / 0.35).clamp(0.0, 1.0),
                )
                .text(figures),
            );
        if props.show_dropoff && next.is_some() && value > 0.0 {
            let sign = if dropoff >= 0.0 { "+" } else { "\u{2212}" };
            group = group.child(
                label(
                    width - label_size * 0.5,
                    bottom + props.gap / 2.0,
                    &props.dropoff_color,
                    label_size * 0.92,
                    700,
                    dropoff_progress,
                )
                .text(format!("{sign}{}%", js_number_text(dropoff.abs()))),
            );
        }
        if props.show_dropoff && index == 0 {
            group = group.child(
                label(
                    width - label_size * 0.5,
                    top + band_height * 0.32,
                    &props.label_color,
                    label_size * 0.82,
                    600,
                    ((shown - 0.4) / 0.3).clamp(0.0, 1.0),
                )
                .text(props.dropoff_heading.as_str()),
            );
        }
        root = root.child(group);
    }
    Ok(div(absolute_fill()
        .align_items(Align::Center)
        .justify_content(Align::Center)
        .text_align(TextAlign::Center))
    .child(svg(root, Style::default()))
    .into())
}
