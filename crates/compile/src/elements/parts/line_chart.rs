use opsis_scene::node::{Element, SvgElement, svg};
use opsis_scene::random::js_number_text;
use opsis_scene::style::{Overflow, Style};
use opsis_types::numbers::{float, whole};

use crate::elements::anim::{clamped, clamped_eased};
use crate::elements::chart_utils::{
    area_path, line_path, nice_domain, plot_area, plot_points, smooth_path,
};
use crate::elements::draw::DrawFailure;
use crate::elements::number_text::format_axis_value;
use crate::elements::path_utils::{path_draw, sample_path};
use crate::elements::text_split::easing_enter;

const SMOOTH_TENSION: f64 = 0.42;

#[derive(Debug, Clone)]
pub struct LineChart {
    pub points: Vec<(f64, f64, Option<String>)>,
    pub width: f64,
    pub height: f64,
    pub color: String,
    pub stroke_width: Option<f64>,
    pub smooth: bool,
    pub show_axis: bool,
    pub show_x_labels: bool,
    pub show_area: bool,
    pub show_dots: bool,
    pub show_head: bool,
    pub show_end_label: bool,
    pub include_zero: bool,
    pub label_color: String,
    pub grid_color: String,
    pub surface_color: String,
    pub duration: f64,
    pub delay: f64,
    pub frame: f64,
    pub id: String,
}

impl LineChart {
    pub fn new(
        points: Vec<(f64, f64, Option<String>)>,
        width: f64,
        height: f64,
        frame: f64,
    ) -> Self {
        Self {
            points,
            width,
            height,
            color: "#e8b86d".to_owned(),
            stroke_width: None,
            smooth: true,
            show_axis: true,
            show_x_labels: true,
            show_area: true,
            show_dots: true,
            show_head: true,
            show_end_label: false,
            include_zero: true,
            label_color: "rgba(250,250,250,0.52)".to_owned(),
            grid_color: "rgba(250,250,250,0.10)".to_owned(),
            surface_color: "#080810".to_owned(),
            duration: 70.0,
            delay: 0.0,
            frame,
            id: "line-chart".to_owned(),
        }
    }
}

fn text(x: f64, y: f64, fill: &str, size: f64, weight: u32, anchor: &str) -> SvgElement {
    SvgElement::new("text")
        .attr("x", x)
        .attr("y", y)
        .attr("fill", fill)
        .attr("font-size", size)
        .attr("font-weight", weight)
        .attr("text-anchor", anchor)
        .attr("dominant-baseline", "central")
}

#[expect(
    clippy::too_many_lines,
    reason = "the chart's markup follows the film's one view function in order"
)]
pub fn line_chart(chart: &LineChart) -> Result<Element, DrawFailure> {
    let unit = chart.width / 960.0;
    let scale = |value: f64| f64::from(whole(value * unit)).max(1.0);
    let label_size = scale(22.0);
    let stroke = chart.stroke_width.unwrap_or_else(|| scale(5.0));
    let ys: Vec<f64> = chart.points.iter().map(|p| p.1).collect();
    let (domain, ticks) = nice_domain(&ys, chart.include_zero, 4.0);
    let labels: Vec<String> = ticks.iter().map(|t| format_axis_value(*t)).collect();
    let longest = labels
        .iter()
        .map(|l| l.encode_utf16().count())
        .max()
        .unwrap_or(0);
    let gutter = if chart.show_axis {
        f64::from(whole(float(longest) * label_size * 0.62)) + scale(18.0)
    } else {
        scale(6.0)
    };
    let plot = plot_area(
        chart.width,
        chart.height,
        (
            scale(if chart.show_end_label { 34.0 } else { 16.0 }),
            scale(if chart.show_end_label { 44.0 } else { 12.0 }),
            if chart.show_x_labels {
                label_size + scale(20.0)
            } else {
                scale(10.0)
            },
            gutter,
        ),
    );
    let plotted = plot_points(&chart.points, &plot, &domain);
    let xy: Vec<(f64, f64)> = plotted.iter().map(|p| (p.x, p.y)).collect();
    let line = if chart.smooth {
        smooth_path(&xy, SMOOTH_TENSION)
    } else {
        line_path(&xy)
    };
    let area = area_path(&line, &xy, plot.bottom);
    let progress = clamped_eased(
        chart.frame,
        [chart.delay, chart.delay + chart.duration],
        [0.0, 1.0],
        easing_enter(),
    )?;
    let tip = if line.is_empty() {
        (plot.left, plot.bottom)
    } else {
        let s = sample_path(&line, progress);
        (s.x, s.y)
    };
    let gradient = format!("{}-fill", chart.id);
    let clip = format!("{}-clip", chart.id);
    let n = js_number_text;
    let mut root = SvgElement::new("svg")
        .attr("width", chart.width)
        .attr("height", chart.height)
        .attr(
            "viewBox",
            format!("0 0 {} {}", n(chart.width), n(chart.height)),
        )
        .child(
            SvgElement::new("defs")
                .child(
                    SvgElement::new("linearGradient")
                        .attr("id", gradient.as_str())
                        .attr("x1", "0")
                        .attr("y1", "0")
                        .attr("x2", "0")
                        .attr("y2", "1")
                        .child(
                            SvgElement::new("stop")
                                .attr("offset", "0%")
                                .attr("stop-color", chart.color.as_str())
                                .attr("stop-opacity", 0.32),
                        )
                        .child(
                            SvgElement::new("stop")
                                .attr("offset", "100%")
                                .attr("stop-color", chart.color.as_str())
                                .attr("stop-opacity", 0),
                        ),
                )
                .child(
                    SvgElement::new("clipPath").attr("id", clip.as_str()).child(
                        SvgElement::new("rect")
                            .attr("x", plot.left)
                            .attr("y", plot.top - stroke)
                            .attr("width", plot.width * progress)
                            .attr("height", stroke.mul_add(2.0, plot.height)),
                    ),
                ),
        );
    if chart.show_axis {
        for (index, tick) in ticks.iter().enumerate() {
            let y = f64::mul_add((tick - domain.min) / domain.span, -plot.height, plot.bottom);
            root = root.child(
                SvgElement::new("g")
                    .child(
                        SvgElement::new("line")
                            .attr("x1", plot.left)
                            .attr("y1", y)
                            .attr("x2", plot.right)
                            .attr("y2", y)
                            .attr("stroke", chart.grid_color.as_str())
                            .attr("stroke-width", 1)
                            .attr("opacity", if index == 0 { 1.8 } else { 1.0 }),
                    )
                    .child(
                        text(
                            plot.left - scale(12.0),
                            y,
                            &chart.label_color,
                            label_size,
                            600,
                            "end",
                        )
                        .text(labels[index].as_str()),
                    ),
            );
        }
    }
    if chart.show_area && !area.is_empty() {
        root = root.child(
            SvgElement::new("path")
                .attr("d", area)
                .attr("fill", format!("url(#{gradient})"))
                .attr("clip-path", format!("url(#{clip})")),
        );
    }
    let (dash, offset) = path_draw(progress, &line);
    root = root.child(
        SvgElement::new("path")
            .attr("d", line.as_str())
            .attr("fill", "none")
            .attr("stroke", chart.color.as_str())
            .attr("stroke-width", stroke)
            .attr("stroke-linecap", "round")
            .attr("stroke-linejoin", "round")
            .attr("stroke-dasharray", format!("{} {}", n(dash), n(dash)))
            .attr("stroke-dashoffset", offset),
    );
    if chart.show_x_labels {
        let last = plotted.len().saturating_sub(1);
        for (index, point) in plotted.iter().enumerate() {
            let anchor = if index == 0 {
                "start"
            } else if index == last {
                "end"
            } else {
                "middle"
            };
            root = root.child(
                text(
                    point.x,
                    plot.bottom + scale(14.0) + label_size / 2.0,
                    &chart.label_color,
                    label_size,
                    600,
                    anchor,
                )
                .text(point.label.clone().unwrap_or_default()),
            );
        }
    }
    if chart.show_dots {
        let window = (plot.width * 0.04).max(1.0);
        for point in &plotted {
            let reveal = ((tip.0 - point.x + window) / window).clamp(0.0, 1.0);
            root = root.child(
                SvgElement::new("circle")
                    .attr("cx", point.x)
                    .attr("cy", point.y)
                    .attr("r", stroke * 0.95 * reveal)
                    .attr("fill", chart.color.as_str())
                    .attr("stroke", chart.surface_color.as_str())
                    .attr("stroke-width", stroke * 0.5 * reveal),
            );
        }
    }
    if chart.show_head && progress > 0.0 && progress < 1.0 {
        root = root.child(
            SvgElement::new("g")
                .child(
                    SvgElement::new("circle")
                        .attr("cx", tip.0)
                        .attr("cy", tip.1)
                        .attr("r", stroke * 2.6)
                        .attr("fill", chart.color.as_str())
                        .attr("opacity", 0.18),
                )
                .child(
                    SvgElement::new("circle")
                        .attr("cx", tip.0)
                        .attr("cy", tip.1)
                        .attr("r", stroke * 1.05)
                        .attr("fill", chart.color.as_str()),
                ),
        );
    }
    if chart.show_end_label
        && let Some(end) = plotted.last()
    {
        root = root.child(
            text(
                end.x,
                end.y - scale(22.0),
                &chart.color,
                scale(28.0),
                700,
                "end",
            )
            .attr("opacity", clamped(progress, [0.88, 1.0], [0.0, 1.0])?)
            .text(format_axis_value(end.value)),
        );
    }
    Ok(svg(root, Style::default().overflow(Overflow::Visible)))
}
