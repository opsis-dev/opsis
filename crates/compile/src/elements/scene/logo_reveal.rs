use opsis_macros::settings;
use opsis_scene::color::{Color, Interpolation as ColorInterpolation};
use opsis_scene::interpolate::{Interpolation, interpolate};
use opsis_scene::node::{Node, SvgElement, div, svg};
use opsis_scene::paint::{ColorStop, Filter, Gradient, RadialShape, RadialSize};
use opsis_scene::random::js_number_text;
use opsis_scene::spring::{Spring, SpringConfig, spring};
use opsis_scene::style::{Align, Display, FlexDirection, LineHeight, Position, Style, TextAlign};
use opsis_scene::values::{Edges, Length, Num, Position2};
use opsis_types::looks::{NodeLook, NodeMotion};
use opsis_types::numbers::{exact_eq, float};

use crate::elements::draw::{Cx, DrawFailure, css_color, tinted};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::geometry::safe_padding;
use crate::elements::path_utils::{fit_view_box, path_length, sample_path};
use crate::elements::text_split::easing_enter;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct LogoReveal {
    #[setting(
        "Logo outline",
        default = "M 100 20 L 170 60 L 170 140 L 100 180 L 30 140 L 30 60 Z M 100 64 L 138 86 L 138 130 L 100 152 L 62 130 L 62 86 Z",
        widget = "code",
        help = "Logo path, or paths drawn in sequence."
    )]
    pub path_d: String,
    #[setting(
        "Name",
        default = "Acme",
        help = "Brand name rising after the mark completes."
    )]
    pub wordmark: String,
    #[setting(
        "Tagline",
        default = "Ship your next release faster",
        help = "Supporting line, entering last."
    )]
    pub tagline: String,
    #[setting("Line color", default = "#e8b86d", help = "Mark stroke color.", check = colour)]
    pub stroke: String,
    #[setting("Background color", default = "#080810", help = "Scene background.", check = colour)]
    pub background_color: String,
    #[setting("Text color", default = "#f4f4f5", check = colour)]
    pub text_color: String,
    #[setting(
        "Outline area",
        zero_label = "Fit",
        help = "Leave empty to fit the outline.",
        advanced
    )]
    pub view_box: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Logo size",
        range(min = 20, max = 2000),
        zero_label = "Auto",
        step = 10,
        unit = "px",
        "type" = "integer"
    )]
    pub size: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Line width",
        range(min = 1, max = 40),
        zero_label = "Auto",
        step = 0.5,
        unit = "px",
        advanced,
        "type" = "number"
    )]
    pub stroke_width: Option<f64>,
    #[setting("Fill colour", widget = "color", zero_label = "None")]
    pub fill: String,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project."
    )]
    pub font_family: String,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4), step = 0.25)]
    pub speed: f64,
    #[setting(
        "Draw time",
        default = 2.4,
        range(min = 0.2, max = 10),
        step = 0.1,
        unit = "s",
        advanced
    )]
    pub draw_seconds: f64,
    #[setting(
        "Time between lines",
        default = 0.27,
        range(min = 0, max = 2),
        step = 0.01,
        unit = "s",
        help = "When the logo has several lines, each starts this long after the one before.",
        advanced
    )]
    pub path_stagger: f64,
    #[setting(
        "Name at",
        default = 1.37,
        range(min = 0, max = 20),
        step = 0.01,
        unit = "s",
        help = "The tagline follows 0.4 s later.",
        advanced
    )]
    pub name_at: f64,
    #[setting("Glow", default = true, advanced)]
    pub glow: bool,
    #[setting(
        "Pen tip",
        default = true,
        help = "A dot at the end of the line while it draws.",
        advanced
    )]
    pub pen_tip: bool,
    #[setting(
        "Name size",
        default = 84,
        range(min = 16, max = 240),
        step = 1,
        unit = "px"
    )]
    pub name_size: u32,
    #[setting(
        "Name weight",
        default = 600,
        range(min = 100, max = 900),
        step = 100,
        advanced
    )]
    pub font_weight: u32,
    #[setting(
        "Tagline size",
        default = 40,
        range(min = 10, max = 120),
        step = 1,
        unit = "px",
        advanced
    )]
    pub tagline_size: u32,
    #[setting(
        "Tagline color",
        widget = "color",
        zero_label = "Softer text color",
        check = colour_or_empty,
        advanced
    )]
    pub tagline_color: String,
}

pub fn definition() -> ElementMeta {
    meta::<LogoReveal>(
        element(
            ElementKind::Component,
            "logo-reveal",
            "Logo reveal",
            "Draws your logo outline, then brings in the name and a tagline.",
            ElementCategory::Scene,
            120,
        )
        .look(NodeLook::of("lucide:pen-tool").moves(NodeMotion::Draw)),
    )
}

const AUTO_FIT_PADDING: f64 = 8.0;
const DURATION_SLOW: f64 = 36.0;
const DURATION_NORMAL: f64 = 24.0;
const DELAY_SHORT: f64 = 5.0;
const SMOOTH: SpringConfig = SpringConfig {
    damping: 200.0,
    stiffness: 100.0,
    mass: 1.0,
    overshoot_clamping: true,
};
const GLOW_ID: &str = "path-draw-glow";

fn scale_font(size: f64, width: f64) -> f64 {
    size.mul_add(width / 1080.0, 0.5).floor()
}

fn smooth(frame: f64, fps: f64, duration: f64) -> Result<f64, DrawFailure> {
    Ok(spring(&Spring {
        config: SMOOTH,
        duration_in_frames: Some(duration),
        ..Spring::at(frame, fps)
    })?)
}

fn eased(frame: f64, range: [f64; 2]) -> Result<f64, DrawFailure> {
    Ok(interpolate(
        frame,
        &range,
        &[0.0, 1.0],
        &Interpolation {
            easing: Some(easing_enter()),
            ..Interpolation::clamped()
        },
    )?)
}

struct Draw<'a> {
    path: &'a str,
    view_box: String,
    size: f64,
    line_width: f64,
    frames: f64,
    stagger: f64,
}

fn path_draw(props: &LogoReveal, draw: &Draw<'_>, frame: f64) -> Result<Node, DrawFailure> {
    let paths: Vec<&str> = std::iter::once(draw.path)
        .filter(|p| !p.is_empty())
        .collect();
    let mut root = SvgElement::new("svg")
        .attr("width", draw.size)
        .attr("height", draw.size)
        .attr("viewBox", draw.view_box.as_str());
    if props.glow {
        let parts: Vec<String> = draw
            .view_box
            .split(|c: char| c.is_whitespace() || c == ',')
            .filter(|p| !p.is_empty())
            .map(str::to_owned)
            .collect();
        let part = |i: usize| parts.get(i).cloned().unwrap_or_default();
        root = root.child(
            SvgElement::new("defs").child(
                SvgElement::new("filter")
                    .attr("id", GLOW_ID)
                    .attr("filterUnits", "userSpaceOnUse")
                    .attr("x", part(0))
                    .attr("y", part(1))
                    .attr("width", part(2))
                    .attr("height", part(3))
                    .child(
                        SvgElement::new("feGaussianBlur")
                            .attr("stdDeviation", "1.5")
                            .attr("result", "blur"),
                    )
                    .child(
                        SvgElement::new("feMerge")
                            .child(SvgElement::new("feMergeNode").attr("in", "blur"))
                            .child(SvgElement::new("feMergeNode").attr("in", "SourceGraphic")),
                    ),
            ),
        );
    }
    for (index, path) in paths.iter().enumerate() {
        let start = float(index).mul_add(draw.stagger, DELAY_SHORT);
        let progress = eased(frame, [start, start + draw.frames])?.clamp(0.0, 1.0);
        let (dash, offset) = dash_for(progress, path);
        let mut group = SvgElement::new("g");
        if !props.fill.is_empty() {
            let fill_opacity = eased(
                frame,
                [
                    draw.frames.mul_add(0.75, start),
                    draw.frames.mul_add(1.3, start),
                ],
            )?;
            group = group.child(
                SvgElement::new("path")
                    .attr("d", *path)
                    .attr("fill", props.fill.as_str())
                    .attr("opacity", fill_opacity)
                    .attr("stroke", "none"),
            );
        }
        let mut line = SvgElement::new("path")
            .attr("d", *path)
            .attr("fill", "none")
            .attr("stroke", props.stroke.as_str())
            .attr("stroke-width", draw.line_width)
            .attr("stroke-linecap", "round")
            .attr("stroke-linejoin", "round")
            .attr(
                "stroke-dasharray",
                format!("{} {}", js_number_text(dash), js_number_text(dash)),
            )
            .attr("stroke-dashoffset", offset)
            .attr("opacity", 0.95);
        if props.glow {
            line = line.attr("filter", format!("url(#{GLOW_ID})"));
        }
        group = group.child(line);
        if props.pen_tip && progress > 0.0 && progress < 1.0 {
            let tip = sample_path(path, progress);
            let mut dot = SvgElement::new("circle")
                .attr("cx", tip.x)
                .attr("cy", tip.y)
                .attr("r", draw.line_width * 0.9)
                .attr("fill", props.stroke.as_str());
            if props.glow {
                dot = dot.attr("filter", format!("url(#{GLOW_ID})"));
            }
            group = group.child(dot);
        }
        root = root.child(group);
    }
    Ok(svg(root, Style::default()).into())
}

fn dash_for(progress: f64, path: &str) -> (f64, f64) {
    let length = path_length(path);
    if exact_eq(progress, 0.0) {
        let dash = length * 1.5;
        return (dash, dash);
    }
    (length, progress.mul_add(-length, length))
}

pub fn draw(props: &LogoReveal, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let (width, height, fps) = (cx.at.stage.width, cx.at.stage.height, cx.fps());
    let frame = cx.frame() * props.speed;
    let (side, edge) = safe_padding(width, height);
    let has_copy = !props.wordmark.is_empty() || !props.tagline.is_empty();
    let mark_size = props.size.map_or_else(
        || f64::mul_add(width.min(height), if has_copy { 0.4 } else { 0.56 }, 0.5).floor(),
        f64::from,
    );
    let line_width = props
        .stroke_width
        .unwrap_or_else(|| 2.0f64.max(mark_size.mul_add(0.022, 0.5).floor()));
    let draw_frames = props.draw_seconds * fps;
    let name_frame = props.name_at * fps;
    let tagline_frame = 0.4f64.mul_add(fps, name_frame);
    let bloom = smooth(frame, fps, DURATION_SLOW)?;
    let wordmark_lift = smooth(frame - name_frame, fps, DURATION_NORMAL)?;
    let tagline_in = eased(frame, [tagline_frame, tagline_frame + DURATION_NORMAL])?;
    let view_box = if props.view_box.is_empty() {
        fit_view_box(&[props.path_d.as_str()], AUTO_FIT_PADDING)
    } else {
        props.view_box.clone()
    };
    let grow = bloom.mul_add(0.08, 0.92);
    let mut mark = div(Style::default()
        .position(Position::Relative)
        .display(Display::Flex)
        .align_items(Align::Center)
        .justify_content(Align::Center)
        .scale(grow, grow));
    if props.glow {
        let halo = bloom.mul_add(0.35, 0.8);
        mark = mark.child(div(Style::default()
            .position(Position::Absolute)
            .width(Length::px(mark_size * 1.8))
            .height(Length::px(mark_size * 1.8))
            .border_radius_all(Length::percent(50.0))
            .background(vec![
                Gradient::Radial {
                    shape: RadialShape::Circle,
                    size: RadialSize::FarthestCorner,
                    center: Position2 {
                        x: Length::percent(50.0),
                        y: Length::percent(50.0),
                    },
                    stops: vec![
                        ColorStop::new(tinted(&props.stroke, 22.0)),
                        ColorStop::at(Color::TRANSPARENT, Length::percent(70.0)),
                    ],
                    repeating: false,
                    interpolation: ColorInterpolation::DEFAULT,
                }
                .layer(),
            ])
            .filter(vec![Filter::Blur(Length::px(scale_font(36.0, width)))])
            .opacity(bloom * 0.9)
            .scale(halo, halo)));
    }
    let mark = mark.child(path_draw(
        props,
        &Draw {
            path: &props.path_d,
            view_box,
            size: mark_size,
            line_width,
            frames: draw_frames,
            stagger: props.path_stagger * fps,
        },
        frame,
    )?);
    let weight = f64::from(props.font_weight);
    let mut column = div(Style::default()
        .width(Length::px(width))
        .height(Length::px(height))
        .background_color(css_color(&props.background_color))
        .padding(Edges::axes(Length::px(edge), Length::px(side)))
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .align_items(Align::Center)
        .justify_content(Align::Center)
        .gap(scale_font(28.0, width))
        .position(Position::Relative)
        .font_family(font_or(&props.font_family, FontRole::Main)))
    .child(mark);
    if !props.wordmark.is_empty() {
        column = column.child(
            div(Style::default()
                .color(css_color(&props.text_color))
                .font_size(scale_font(f64::from(props.name_size), width))
                .font_weight(weight)
                .line_height(LineHeight::Number(Num(1.0)))
                .letter_spacing(Length::px(-0.5))
                .text_align(TextAlign::Center)
                .opacity(wordmark_lift)
                .translate(
                    Length::ZERO,
                    Length::px((1.0 - wordmark_lift) * scale_font(28.0, width)),
                ))
            .child(props.wordmark.clone()),
        );
    }
    if !props.tagline.is_empty() {
        let color = if props.tagline_color.is_empty() {
            tinted(&props.text_color, 60.0)
        } else {
            css_color(&props.tagline_color)
        };
        column = column.child(
            div(Style::default()
                .color(color)
                .font_size(scale_font(f64::from(props.tagline_size), width))
                .font_weight((weight - 100.0).max(100.0))
                .line_height(LineHeight::Number(Num(1.2)))
                .text_align(TextAlign::Center)
                .max_width(Length::px(width * 0.7))
                .opacity(tagline_in)
                .translate(
                    Length::ZERO,
                    Length::px((1.0 - tagline_in) * scale_font(16.0, width)),
                ))
            .child(props.tagline.clone()),
        );
    }
    Ok(column.into())
}
