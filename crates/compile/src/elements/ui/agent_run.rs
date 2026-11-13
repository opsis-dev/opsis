use std::f64::consts::{PI, SQRT_2};

use indexmap::IndexMap;
use opsis_macros::{choice, settings};
use opsis_scene::color::{Color, ColorSpace, Interpolation as ColorInterpolation};
use opsis_scene::css::shadows;
use opsis_scene::node::{Node, SvgElement, div, span, svg};
use opsis_scene::paint::Transform;
use opsis_scene::random::js_number_text;
use opsis_scene::style::{
    Align, BorderSide, BorderStyle, BoxSizing, Display, LineHeight, NumericVariant, Overflow,
    Position, Style, TextAlign, TextOverflow, VerticalAlign, WhiteSpace,
};
use opsis_scene::values::{Edges, Length, Num};
use opsis_types::looks::NodeLook;

use crate::elements::agent_model::{
    BODY_LINE, BODY_SIZE, BUBBLE_AT, BUBBLE_IN, BUBBLE_LINE, BUBBLE_PAD_X, BUBBLE_PAD_Y,
    BUBBLE_RISE, BUBBLE_SIZE, BlockKind, BlockState, CHIP_GAP, CHIP_SIZE, CHIPS_H, CODE_HEAD,
    CODE_PAD, CODE_SIZE, COLUMN, HEADER_H, ITEM_INDENT, LineState, PAD_X, PANEL_IN, PANEL_W,
    PLAN_INDENT, PLAN_LINE, PLAN_SIZE, PieceKind, Pop, ROW_H, RUN_START, RowState, Run, STATUS_H,
    STATUS_IN, SWAP, SWAP_RISE, Span as AgentSpan, Spinner, Step, block_state, ease_out,
    elapsed_at, expo, format_seconds, frame_of, heading_spec, layout_of, panel_lift, plan_state,
    reveal, round3, scroll_at, scroll_steps, shimmer_at, spinner_at, text_of, think_label_shown,
    timeline_of, tools_state, units,
};
use crate::elements::draw::{Cx, DrawFailure, css_color, tinted};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::theme::{Theme, ThemeMode, agent_run_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
pub enum AgentStepIcon {
    Chart,
    Database,
    Search,
    Book,
    Globe,
    Terminal,
    File,
}

#[settings(export)]
pub struct AgentRunRunStep {
    #[setting("Name", default = "Search logs")]
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Detail", "type" = "string")]
    pub detail: Option<String>,
    #[setting(
        "Length",
        default = 1.5,
        range(min = 0.2, max = 20),
        step = 0.1,
        unit = "s"
    )]
    pub duration: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Icon",
        "type" = "string",
        "enum" = ["chart", "database", "search", "book", "globe", "terminal", "file"]
    )]
    pub icon: Option<AgentStepIcon>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Sources", "type" = "array")]
    pub sources: Option<Vec<String>>,
}

#[settings(export)]
pub struct AgentRun {
    #[setting(
        "Theme",
        default = Theme::of(ThemeMode::Light),
        transform = agent_run_themes,
        widget = "theme"
    )]
    pub theme: Theme,
    #[setting("Accent", default = "#2563eb", check = colour)]
    pub accent_color: String,
    #[setting(
        "Compression",
        default = 3.0,
        range(min = 1, max = 8),
        step = 0.5,
        advanced
    )]
    pub compression: f64,
    #[setting("Seed", default = 1, range(min = 1, max = 99), step = 1, advanced)]
    pub seed: u32,
    #[setting("Title", default = "Checkout conversion drop")]
    pub run_title: String,
    #[setting("Prompt", default = "Why did checkout conversion drop last week?")]
    pub run_prompt: String,
    #[setting(
        "Plan",
        default = vec![
            "Compare the checkout funnel with the week before".into(),
            "Break the drop down by device and step".into(),
            "Check what shipped and what the error logs say".into(),
        ]
    )]
    pub run_plan: Vec<String>,
    #[setting(
        "Steps",
        default = vec![
            AgentRunRunStep {
                icon: Some(AgentStepIcon::Chart),
                name: "Query analytics".into(),
                detail: Some("checkout funnel, last 7 days vs prior 7".into()),
                duration: 1.4,
                ..Default::default()
            },
            AgentRunRunStep {
                icon: Some(AgentStepIcon::Database),
                name: "Run a query".into(),
                detail: Some("conversion by device and step".into()),
                duration: 2.3,
                ..Default::default()
            },
            AgentRunRunStep {
                icon: Some(AgentStepIcon::Search),
                detail: Some("card payment errors on tablets".into()),
                duration: 1.1,
                ..Default::default()
            },
            AgentRunRunStep {
                icon: Some(AgentStepIcon::Book),
                name: "Read 12 sources".into(),
                duration: 3.2,
                sources: Some(vec!["Checkout funnel".into(), "Device report".into(), "Release 4.12".into(), "Change #2186".into(), "Payments guide".into(), "Error log".into(), "Incident notes".into(), "Runbook".into(), "Support tickets".into(), "Status page".into(), "Team notes".into(), "Design spec".into()]),
                ..Default::default()
            },
        ]
    )]
    pub run_steps: Vec<AgentRunRunStep>,
    #[setting(
        "Answer",
        default = "## Card payments broke on tablets in release 4.12\n\nCheckout conversion fell from 3.4% to 2.7% last week, a 21% drop [1]. All of it comes from the payment step on tablets, 41% of checkouts [2].\n\n- Tablet conversion halved, from 3.5% to 1.8%\n- Phones and desktops held steady at 3.3%\n- 4.12 moved the card form into a frame without the payment permission [3]\n\n```tsx\n<iframe\n  src={cardFormUrl}\n  allow=\"payment\"\n/>\n```\n\n**Fix:** add the permission and ship 4.12.1 today.",
        widget = "textarea",
        help = "Use ## for a heading, - for a list, **bold**, `code` and [1] to cite a source."
    )]
    pub run_answer: String,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4), step = 0.25)]
    pub speed: f64,
    #[setting("Thinking label", default = "Thinking", advanced)]
    pub thinking_label: String,
    #[setting("Done thinking label", default = "Thought for", advanced)]
    pub thought_label: String,
    #[setting("Timer", default = true, advanced)]
    pub show_timer: bool,
    #[setting(
        "Corner radius",
        default = 18,
        range(min = 0, max = 40),
        step = 1,
        unit = "px",
        advanced
    )]
    pub corner_radius: u32,
    #[setting("Font", widget = "font", advanced)]
    pub font: String,
}

pub fn definition() -> ElementMeta {
    meta::<AgentRun>(
        element(
            ElementKind::Component,
            "agent-run",
            "Agent run",
            "An AI assistant working through a plan, ticking off each step.",
            ElementCategory::Ui,
            355,
        )
        .authored_clock()
        .look(NodeLook::of("lucide:bot")),
    )
}

const LIGHT: [(&str, &str); 11] = [
    ("panel", "#ffffff"),
    ("border", "#e4e4e0"),
    ("divider", "#ededea"),
    ("fg", "#1c1c1a"),
    ("muted", "#63635e"),
    ("subtle", "#9a9a94"),
    ("surface", "#f3f3f1"),
    ("surfaceBorder", "#e7e7e3"),
    ("code", "#f8f8f6"),
    ("track", "#e3e3df"),
    ("shadow", "0 1px 2px rgba(28, 28, 26, 0.06)"),
];
const DARK: [(&str, &str); 11] = [
    ("panel", "#161616"),
    ("border", "#2b2b2a"),
    ("divider", "#232322"),
    ("fg", "#ececea"),
    ("muted", "#a2a29d"),
    ("subtle", "#6d6d68"),
    ("surface", "#212120"),
    ("surfaceBorder", "#2e2e2d"),
    ("code", "#1c1c1b"),
    ("track", "#343433"),
    ("shadow", "0 1px 2px rgba(0, 0, 0, 0.5)"),
];
const CIRCLE: f64 = 2.0 * PI * 10.0;
const CHECK_LENGTH: f64 = 6.0 * SQRT_2;

struct Palette<'a> {
    base: &'static [(&'static str, &'static str); 11],
    overrides: &'a IndexMap<String, String>,
    accent: String,
    accent_tint: Color,
    accent_text: Color,
}

impl Palette<'_> {
    fn get(&self, key: &str) -> &str {
        self.overrides.get(key).map_or_else(
            || {
                self.base
                    .iter()
                    .find(|(k, _)| *k == key)
                    .map_or("", |(_, v)| *v)
            },
            String::as_str,
        )
    }

    fn color(&self, key: &str) -> Color {
        css_color(self.get(key))
    }
}

fn palette_of<'a>(theme: &'a Theme, accent: &str) -> Palette<'a> {
    let dark = theme.mode == ThemeMode::Dark;
    Palette {
        base: if dark { &DARK } else { &LIGHT },
        overrides: &theme.colors,
        accent: accent.to_owned(),
        accent_tint: tinted(accent, if dark { 22.0 } else { 12.0 }),
        accent_text: if dark {
            Color::mix(
                ColorInterpolation {
                    space: ColorSpace::Srgb,
                    ..ColorInterpolation::DEFAULT
                },
                (css_color(accent), Some(62.0)),
                (Color::css("#ffffff").unwrap_or_default(), None),
            )
        } else {
            css_color(accent)
        },
    }
}

fn icon_nodes(name: &str) -> Vec<SvgElement> {
    let path = |d: &str| SvgElement::new("path").attr("d", d);
    let circle = |cx: f64, cy: f64, r: f64| {
        SvgElement::new("circle")
            .attr("cx", cx)
            .attr("cy", cy)
            .attr("r", r)
    };
    match name {
        "chart" => vec![
            path("M3 3v16a2 2 0 0 0 2 2h16"),
            path("M18 17V9"),
            path("M13 17V5"),
            path("M8 17v-3"),
        ],
        "database" => vec![
            SvgElement::new("ellipse")
                .attr("cx", 12.0)
                .attr("cy", 5.0)
                .attr("rx", 9.0)
                .attr("ry", 3.0),
            path("M3 5V19A9 3 0 0 0 21 19V5"),
            path("M3 12A9 3 0 0 0 21 12"),
        ],
        "search" => vec![path("m21 21-4.34-4.34"), circle(11.0, 11.0, 8.0)],
        "book" => vec![
            path("M12 7v14"),
            path(
                "M3 18a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1h5a4 4 0 0 1 4 4 4 4 0 0 1 4-4h5a1 1 0 0 1 1 1v13a1 1 0 0 1-1 1h-6a3 3 0 0 0-3 3 3 3 0 0 0-3-3z",
            ),
        ],
        "globe" => vec![
            circle(12.0, 12.0, 10.0),
            path("M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20"),
            path("M2 12h20"),
        ],
        "terminal" => vec![path("M12 19h8"), path("m4 17 6-6-6-6")],
        "file" => vec![
            path(
                "M6 22a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h8a2.4 2.4 0 0 1 1.704.706l3.588 3.588A2.4 2.4 0 0 1 20 8v12a2 2 0 0 1-2 2z",
            ),
            path("M14 2v5a1 1 0 0 0 1 1h5"),
            path("M10 9H8"),
            path("M16 13H8"),
            path("M16 17H8"),
        ],
        _ => vec![
            SvgElement::new("rect")
                .attr("x", 8.0)
                .attr("y", 8.0)
                .attr("width", 14.0)
                .attr("height", 14.0)
                .attr("rx", 2.0),
            path("M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"),
        ],
    }
}

fn svg_box(size: f64, root: SvgElement) -> Node {
    svg(
        root.attr("width", size)
            .attr("height", size)
            .attr("viewBox", "0 0 24 24")
            .attr("fill", "none"),
        Style::default().display(Display::Block).flex_shrink(0.0),
    )
    .into()
}

fn icon(name: &str, size: f64, color: &str) -> Node {
    let root = icon_nodes(name).into_iter().fold(
        SvgElement::new("svg")
            .attr("stroke", color)
            .attr("stroke-width", 2.0)
            .attr("stroke-linecap", "round")
            .attr("stroke-linejoin", "round"),
        SvgElement::child,
    );
    svg_box(size, root)
}

fn spinner(state: Spinner, size: f64, palette: &Palette<'_>) -> Node {
    let active = 1.0 - state.done;
    let ring = |stroke: &str, opacity: f64| {
        SvgElement::new("circle")
            .attr("cx", 12.0)
            .attr("cy", 12.0)
            .attr("r", 10.0)
            .attr("stroke", stroke)
            .attr("opacity", opacity)
    };
    let n = js_number_text;
    let accent = palette.accent.clone();
    let root = SvgElement::new("svg")
        .attr("stroke-width", 2.0)
        .attr("stroke-linecap", "round")
        .attr("stroke-linejoin", "round")
        .child(ring(palette.get("track"), active))
        .child(
            ring(&accent, active)
                .attr(
                    "stroke-dasharray",
                    format!("{} {}", n(round3(state.arc * CIRCLE)), n(round3(CIRCLE))),
                )
                .attr("transform", format!("rotate({} 12 12)", n(state.angle))),
        )
        .child(ring(palette.get("muted"), state.done))
        .child(
            SvgElement::new("path")
                .attr("d", "m9 12 2 2 4-4")
                .attr("stroke", palette.get("muted"))
                .attr("stroke-dasharray", round3(CHECK_LENGTH))
                .attr(
                    "stroke-dashoffset",
                    round3(CHECK_LENGTH * (1.0 - state.check)),
                )
                .attr("opacity", if state.check > 0.0 { 1.0 } else { 0.0 }),
        );
    svg_box(size, root)
}

fn span_style(span: &AgentSpan, palette: &Palette<'_>) -> Style {
    match span.kind {
        PieceKind::Bold => Style::default().font_weight(600.0),
        PieceKind::Code => Style::default()
            .font_family(font_or("", FontRole::Code))
            .font_size(Length::em(0.875))
            .padding(Edges::axes(Length::px(1.0), Length::px(5.0)))
            .border_radius_all(5.0)
            .background_color(palette.color("surface"))
            .border(line(palette.color("surfaceBorder"))),
        PieceKind::Cite => Style::default()
            .display(Display::InlineFlex)
            .align_items(Align::Center)
            .justify_content(Align::Center)
            .min_width(Length::px(18.0))
            .height(Length::px(18.0))
            .padding(Edges::axes(Length::ZERO, Length::px(5.0)))
            .margin_left(Length::px(2.0))
            .border_radius_all(9.0)
            .box_sizing(BoxSizing::BorderBox)
            .background_color(palette.accent_tint.clone())
            .color(palette.accent_text.clone())
            .font_size(11.5)
            .font_weight(600.0)
            .line_height(LineHeight::Length(Length::px(18.0)))
            .font_variant_numeric(tabular())
            .vertical_align(VerticalAlign::Middle)
            .opacity(span.opacity)
            .transform(vec![Transform::Scale(
                Num(span.scale),
                Num(span.scale),
                Num(1.0),
            )]),
        PieceKind::Punct | PieceKind::Comment => Style::default().color(palette.color("subtle")),
        PieceKind::String => Style::default().color(palette.color("muted")),
        PieceKind::Text => Style::default(),
    }
}

fn spans(list: &[AgentSpan], palette: &Palette<'_>) -> Vec<Node> {
    list.iter()
        .map(|s| {
            span(span_style(s, palette))
                .children(
                    s.fragments
                        .iter()
                        .map(|f| {
                            let style = if f.opacity < 1.0 {
                                Style::default().opacity(f.opacity)
                            } else {
                                Style::default()
                            };
                            span(style).child(f.text.clone()).into()
                        })
                        .collect::<Vec<Node>>(),
                )
                .into()
        })
        .collect()
}

fn tabular() -> NumericVariant {
    NumericVariant {
        tabular: true,
        ..NumericVariant::default()
    }
}

fn line(color: Color) -> Edges<BorderSide> {
    Edges::all(BorderSide {
        width: Length::px(1.0),
        style: BorderStyle::Solid,
        color: Some(color),
    })
}

fn side(color: Color, top: bool) -> Edges<BorderSide> {
    let edge = BorderSide {
        width: Length::px(1.0),
        style: BorderStyle::Solid,
        color: Some(color),
    };
    Edges {
        top: if top {
            edge.clone()
        } else {
            BorderSide::default()
        },
        right: BorderSide::default(),
        bottom: if top { BorderSide::default() } else { edge },
        left: BorderSide::default(),
    }
}

fn placed(top: f64, left: f64) -> Style {
    Style::default()
        .position(Position::Absolute)
        .top(Length::px(top))
        .left(Length::px(left))
}

fn text_line(state: &LineState, top: f64, left: f64, palette: &Palette<'_>) -> Node {
    div(placed(top, left)
        .height(Length::px(state.height))
        .line_height(LineHeight::Length(Length::px(state.height)))
        .white_space(WhiteSpace::Pre))
    .children(spans(&state.spans, palette))
    .into()
}

fn rise(lift: f64) -> Vec<Transform> {
    vec![Transform::Translate(
        Length::ZERO,
        Length::px(lift),
        Length::ZERO,
    )]
}

fn scaled(scale: f64) -> Vec<Transform> {
    vec![Transform::Scale(Num(scale), Num(scale), Num(1.0))]
}

fn answer_block(block: &BlockState, palette: &Palette<'_>) -> Node {
    match block.kind {
        BlockKind::Code => {
            let mut head = div(Style::default()
                .height(Length::px(CODE_HEAD))
                .box_sizing(BoxSizing::BorderBox)
                .display(Display::Flex)
                .align_items(Align::Center)
                .justify_content(Align::SpaceBetween)
                .padding(Edges {
                    top: Length::ZERO,
                    right: Length::px(12.0),
                    bottom: Length::ZERO,
                    left: Length::px(16.0),
                })
                .border(side(palette.color("divider"), false))
                .font_family(font_or("", FontRole::Code))
                .font_size(12.5)
                .color(palette.color("muted")))
            .child(span(Style::default()).child(block.lang.clone()));
            head = head.child(icon("copy", 14.0, palette.get("subtle")));
            let mut outer = div(placed(block.y, 0.0)
                .width(Length::px(COLUMN))
                .height(Length::px(block.height))
                .opacity(block.opacity)
                .overflow(Overflow::Hidden)
                .box_sizing(BoxSizing::BorderBox)
                .border(line(palette.color("surfaceBorder")))
                .border_radius_all(10.0)
                .background_color(palette.color("code")))
            .child(head);
            for l in &block.lines {
                outer = outer.child(
                    div(placed(CODE_HEAD + CODE_PAD - 1.0 + l.y, 16.0)
                        .height(Length::px(l.height))
                        .line_height(LineHeight::Length(Length::px(l.height)))
                        .font_family(font_or("", FontRole::Code))
                        .font_size(CODE_SIZE)
                        .white_space(WhiteSpace::Pre)
                        .color(palette.color("fg")))
                    .children(spans(&l.spans, palette)),
                );
            }
            outer.into()
        }
        BlockKind::Heading(level) => {
            let (size, _) = heading_spec(level);
            let from = BODY_SIZE / size;
            let scale = if block.styled {
                (1.0 - from).mul_add(block.morph, from)
            } else {
                1.0
            };
            let lines: Vec<Node> = block
                .lines
                .iter()
                .enumerate()
                .map(|(index, l)| {
                    let mut row = div(placed(block.y + l.y, 0.0)
                        .height(Length::px(l.height))
                        .display(Display::Flex)
                        .align_items(Align::Center)
                        .white_space(WhiteSpace::Pre));
                    if index == 0 && !block.mark.is_empty() && block.morph < 1.0 {
                        row = row.child(
                            span(
                                Style::default()
                                    .max_width(Length::px(block.mark_width * (1.0 - block.morph)))
                                    .overflow(Overflow::Hidden)
                                    .flex_shrink(0.0)
                                    .opacity(1.0 - block.morph)
                                    .font_size(BODY_SIZE)
                                    .color(palette.color("subtle")),
                            )
                            .child(block.mark.clone()),
                        );
                    }
                    row.child(
                        span(
                            Style::default()
                                .font_size(if block.styled { size } else { BODY_SIZE })
                                .font_weight(200.0f64.mul_add(block.morph, 400.0).round())
                                .line_height(LineHeight::Number(Num(1.25)))
                                .transform(scaled(round3(scale)))
                                .transform_origin((
                                    Length::ZERO,
                                    Length::percent(50.0),
                                    Length::ZERO,
                                )),
                        )
                        .children(spans(&l.spans, palette)),
                    )
                    .into()
                })
                .collect();
            div(Style::default()).children(lines).into()
        }
        BlockKind::Paragraph | BlockKind::Item => {
            let item = block.kind == BlockKind::Item;
            let indent = if item { ITEM_INDENT } else { 0.0 };
            let mut out = div(Style::default());
            if item {
                out = out.child(div(placed(block.y + BODY_LINE / 2.0 - 3.0, 5.0)
                    .width(Length::px(6.0))
                    .height(Length::px(6.0))
                    .border_radius_all(3.0)
                    .background_color(palette.color("fg"))
                    .opacity(block.dot_opacity)
                    .transform(scaled(block.dot))));
            }
            out.children(
                block
                    .lines
                    .iter()
                    .map(|l| text_line(l, block.y + l.y, indent, palette)),
            )
            .into()
        }
    }
}

fn chip(palette: &Palette<'_>, pop: Pop, padding: (f64, f64)) -> Style {
    Style::default()
        .display(Display::InlineFlex)
        .align_items(Align::Center)
        .gap(5.0)
        .height(Length::px(24.0))
        .box_sizing(BoxSizing::BorderBox)
        .padding(Edges {
            top: Length::ZERO,
            right: Length::px(padding.0),
            bottom: Length::ZERO,
            left: Length::px(padding.1),
        })
        .border_radius_all(7.0)
        .border(line(palette.color("surfaceBorder")))
        .background_color(palette.color("surface"))
        .font_size(CHIP_SIZE)
        .color(palette.color("muted"))
        .font_variant_numeric(tabular())
        .white_space(WhiteSpace::Nowrap)
        .opacity(pop.opacity)
        .transform(scaled(pop.scale))
}

fn tool_row(row: &RowState, palette: &Palette<'_>) -> Node {
    let r = &row.layout;
    let mut style = Style::default()
        .position(Position::Absolute)
        .top(Length::px(r.y))
        .left(Length::ZERO)
        .right(Length::ZERO)
        .height(Length::px(row.height))
        .overflow(Overflow::Hidden)
        .box_sizing(BoxSizing::BorderBox);
    if r.divider {
        style = style.border(side(palette.color("divider"), true));
    }
    let main = div(Style::default()
        .height(Length::px(ROW_H))
        .display(Display::Flex)
        .align_items(Align::Center)
        .gap(10.0)
        .padding(Edges::axes(Length::ZERO, Length::px(14.0)))
        .opacity(row.content))
    .child(icon(r.icon, 16.0, palette.get("muted")))
    .child(
        span(
            Style::default()
                .flex_shrink(0.0)
                .font_size(15.0)
                .font_weight(500.0)
                .white_space(WhiteSpace::Nowrap),
        )
        .child(r.name.clone()),
    )
    .child(
        span(
            Style::default()
                .flex_grow(1.0)
                .flex_shrink(1.0)
                .flex_basis(Length::percent(0.0))
                .min_width(Length::ZERO)
                .font_size(15.0)
                .color(palette.color("muted"))
                .white_space(WhiteSpace::Nowrap)
                .overflow(Overflow::Hidden)
                .text_overflow(TextOverflow::Ellipsis),
        )
        .child(r.detail.clone()),
    )
    .child(
        span(
            Style::default()
                .font_size(13.0)
                .color(palette.color("muted"))
                .font_variant_numeric(tabular())
                .opacity(row.label),
        )
        .child(r.duration.clone()),
    )
    .child(spinner(row.spinner, 16.0, palette));
    let mut out = div(style).child(main);
    if row.sources {
        let mut chips: Vec<Node> = row
            .chips
            .iter()
            .filter(|(_, pop)| pop.visible)
            .map(|(c, pop)| {
                span(chip(palette, *pop, (9.0, 7.0)))
                    .child(icon(c.icon, 12.0, palette.get("subtle")))
                    .child(c.label.clone())
                    .into()
            })
            .collect();
        if let Some((count, pop)) = row.more.filter(|(_, p)| p.visible) {
            chips.push(
                span(chip(palette, pop, (9.0, 9.0)))
                    .child(format!("+{count}"))
                    .into(),
            );
        }
        out = out.child(
            div(Style::default()
                .display(Display::Flex)
                .gap(CHIP_GAP)
                .height(Length::px(CHIPS_H))
                .padding(Edges {
                    top: Length::ZERO,
                    right: Length::px(14.0),
                    bottom: Length::ZERO,
                    left: Length::px(40.0),
                })
                .opacity(row.content))
            .children(chips),
        );
    }
    out.into()
}

const fn icon_name(icon: AgentStepIcon) -> &'static str {
    match icon {
        AgentStepIcon::Chart => "chart",
        AgentStepIcon::Database => "database",
        AgentStepIcon::Search => "search",
        AgentStepIcon::Book => "book",
        AgentStepIcon::Globe => "globe",
        AgentStepIcon::Terminal => "terminal",
        AgentStepIcon::File => "file",
    }
}

fn run_of(props: &AgentRun) -> Run {
    Run {
        title: props.run_title.clone(),
        prompt: units(&props.run_prompt),
        plan: props.run_plan.iter().map(|p| units(p)).collect(),
        steps: props
            .run_steps
            .iter()
            .map(|s| Step {
                name: s.name.clone(),
                detail: s.detail.clone().unwrap_or_default(),
                duration: s.duration,
                icon: s.icon.map(icon_name),
                sources: s.sources.clone().unwrap_or_default(),
            })
            .collect(),
        answer: units(&props.run_answer),
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "the panel's sections in the film's order, kept together so the port reads against it"
)]
pub fn draw(props: &AgentRun, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let run = run_of(props);
    let frame = frame_of(cx.at.stage.width, cx.at.stage.height);
    let plan = timeline_of(&run, props.seed, props.compression);
    let layout = layout_of(&run);
    let steps = scroll_steps(&layout, &plan, frame.viewport);
    let time = cx.authored_frame().max(0.0) * props.speed.max(0.0);
    let palette = palette_of(&props.theme, &props.accent_color);
    let elapsed = elapsed_at(time, &plan);
    let running = time - RUN_START;
    let swap = if time >= plan.think_end {
        expo((time - plan.think_end) / SWAP)
    } else {
        0.0
    };
    let scroll = round3(scroll_at(time, &steps));
    let answer_local = time - plan.answer_start;
    let answer_revealed = reveal(&plan.answer_chunks, answer_local);
    let status_opacity = round3(ease_out((running + 1.0) / STATUS_IN));
    let header_spinner = spinner_at(time, RUN_START, plan.run_end);
    let header = div(Style::default()
        .height(Length::px(HEADER_H))
        .box_sizing(BoxSizing::BorderBox)
        .display(Display::Flex)
        .align_items(Align::Center)
        .justify_content(Align::SpaceBetween)
        .gap(16.0)
        .padding(Edges::axes(Length::ZERO, Length::px(PAD_X)))
        .border(side(palette.color("divider"), false)))
    .child(
        span(
            Style::default()
                .min_width(Length::ZERO)
                .font_size(14.0)
                .font_weight(500.0)
                .white_space(WhiteSpace::Nowrap)
                .overflow(Overflow::Hidden)
                .text_overflow(TextOverflow::Ellipsis),
        )
        .child(run.title.clone()),
    )
    .child(
        div(Style::default()
            .display(Display::Flex)
            .align_items(Align::Center)
            .gap(7.0)
            .flex_shrink(0.0)
            .opacity(if props.show_timer {
                status_opacity
            } else {
                0.0
            }))
        .child(spinner(header_spinner, 14.0, &palette))
        .child(
            span(
                Style::default()
                    .min_width(Length::px(38.0))
                    .font_size(13.0)
                    .color(palette.color("muted"))
                    .font_variant_numeric(tabular())
                    .text_align(TextAlign::Right),
            )
            .child(format_seconds(elapsed)),
        ),
    );
    let mut column: Vec<Node> = Vec::new();
    column.push(
        div(Style::default()
            .position(Position::Absolute)
            .top(Length::px(layout.bubble_y))
            .right(Length::ZERO)
            .padding(Edges::axes(
                Length::px(BUBBLE_PAD_Y),
                Length::px(BUBBLE_PAD_X),
            ))
            .border_radius_all(18.0)
            .background_color(palette.color("surface"))
            .font_size(BUBBLE_SIZE)
            .line_height(LineHeight::Length(Length::px(BUBBLE_LINE)))
            .white_space(WhiteSpace::Pre)
            .opacity(round3(ease_out((time - BUBBLE_AT + 1.0) / BUBBLE_IN)))
            .transform(rise(round3(
                BUBBLE_RISE * (1.0 - expo((time - BUBBLE_AT) / BUBBLE_IN)),
            ))))
        .children(
            layout
                .bubble_lines
                .iter()
                .map(|l| div(Style::default()).child(l.clone()).into())
                .collect::<Vec<Node>>(),
        )
        .into(),
    );
    if running >= 0.0 {
        let thinking = units(&props.thinking_label);
        let mut status = div(placed(layout.status_y, 0.0)
            .width(Length::px(COLUMN))
            .height(Length::px(STATUS_H))
            .font_size(15.0)
            .font_weight(500.0)
            .line_height(LineHeight::Length(Length::px(STATUS_H)))
            .white_space(WhiteSpace::Pre)
            .opacity(status_opacity));
        if swap < 1.0 {
            let shimmer = shimmer_at(time, thinking.len());
            status = status.child(
                span(placed(0.0, 0.0).opacity(1.0 - swap)).children(
                    shimmer
                        .iter()
                        .enumerate()
                        .map(|(i, level)| {
                            span(Style::default().opacity(*level))
                                .child(text_of(&thinking[i..=i]))
                                .into()
                        })
                        .collect::<Vec<Node>>(),
                ),
            );
        }
        if swap > 0.0 {
            let thought = format!(
                "{} {}s",
                props.thought_label,
                js_number_text(think_label_shown(plan.think_seconds))
            );
            status = status.child(
                span(
                    placed(0.0, 0.0)
                        .color(palette.color("muted"))
                        .opacity(round3(swap))
                        .transform(rise(round3(SWAP_RISE * (1.0 - swap)))),
                )
                .child(thought),
            );
        }
        column.push(status.into());
    }
    if let Some(section) = &layout.plan {
        let state = plan_state(section, &plan.plan_chunks, time - plan.plan_start);
        let mut out = div(placed(state.y, 0.0)
            .width(Length::px(COLUMN))
            .font_size(PLAN_SIZE)
            .color(palette.color("muted")))
        .child(div(placed(0.0, 0.0)
            .width(Length::px(2.0))
            .height(Length::px(state.rule))
            .border_radius_all(1.0)
            .background_color(palette.color("track"))));
        for item in &state.items {
            let mut group = div(Style::default()).child(
                span(
                    placed(item.y, 14.0)
                        .line_height(LineHeight::Length(Length::px(PLAN_LINE)))
                        .color(palette.color("subtle"))
                        .font_variant_numeric(tabular())
                        .opacity(item.number_opacity),
                )
                .child(format!("{}.", item.index + 1)),
            );
            for l in &item.lines {
                group = group.child(text_line(l, item.y + l.y, PLAN_INDENT, &palette));
            }
            out = out.child(group);
        }
        column.push(out.into());
    }
    if let Some(section) = &layout.tools {
        let state = tools_state(section, &plan, time);
        if !state.rows.is_empty() {
            column.push(
                div(placed(state.y, 0.0)
                    .width(Length::px(COLUMN))
                    .height(Length::px(state.height))
                    .box_sizing(BoxSizing::BorderBox)
                    .overflow(Overflow::Hidden)
                    .border(line(palette.color("surfaceBorder")))
                    .border_radius_all(12.0))
                .children(state.rows.iter().map(|r| tool_row(r, &palette)))
                .into(),
            );
        }
    }
    for block in &layout.blocks {
        if let Some(state) = block_state(
            block,
            run.answer.len(),
            &plan.answer_chunks,
            answer_local,
            answer_revealed,
        ) {
            column.push(answer_block(&state, &palette));
        }
    }
    let mut inner = Style::default()
        .position(Position::Relative)
        .width(Length::percent(100.0))
        .height(Length::percent(100.0))
        .box_sizing(BoxSizing::BorderBox)
        .overflow(Overflow::Hidden)
        .opacity(round3(ease_out((time + 1.0) / PANEL_IN)))
        .transform(rise(panel_lift(time)))
        .background_color(palette.color("panel"))
        .border(line(palette.color("border")))
        .border_radius_all(f64::from(props.corner_radius))
        .color(palette.color("fg"))
        .font_family(font_or(&props.font, FontRole::Main))
        .font_size(BODY_SIZE);
    if let Some(shadow) = shadows(palette.get("shadow")) {
        inner = inner.box_shadow(shadow);
    }
    let viewport = div(Style::default()
        .position(Position::Relative)
        .height(Length::px(frame.viewport))
        .overflow(Overflow::Hidden))
    .child(
        div(placed(0.0, PAD_X)
            .width(Length::px(COLUMN))
            .transform(rise(-scroll)))
        .children(column),
    );
    Ok(div(Style::default()
        .position(Position::Absolute)
        .inset(Length::ZERO))
    .child(
        div(Style::default()
            .position(Position::Absolute)
            .left(Length::percent(50.0))
            .top(Length::percent(50.0))
            .width(Length::px(PANEL_W))
            .height(Length::px(frame.panel_height))
            .translate(Length::percent(-50.0), Length::percent(-50.0))
            .scale(frame.unit, frame.unit))
        .child(div(inner).child(header).child(viewport)),
    )
    .into())
}
