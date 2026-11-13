use opsis_macros::settings;
use opsis_scene::css::with_background;
use opsis_scene::easing::Easing;
use opsis_scene::interpolate::{Interpolation, interpolate};
use opsis_scene::node::{Node, SvgElement, div, span, svg};
use opsis_scene::spring::SpringConfig;
use opsis_scene::style::{
    Align, BorderSide, BorderStyle, Display, NumericVariant, Position, Style, TextAlign,
    TextTransform,
};
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::{Edges, Length};
use opsis_types::looks::NodeLook;
use opsis_types::numbers::float;

use crate::elements::anim::sprung;
use crate::elements::code_theme::code_palette;
use crate::elements::code_window::{CodeWindow, code_window};
use crate::elements::draw::{Cx, DrawFailure, css_color, tinted};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::geometry::safe_padding;
use crate::elements::path_utils::path_length;
use crate::elements::text_split::easing_enter;
use crate::elements::theme::{Theme, ThemeMode, code_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export, eq)]
pub struct ComparisonTableRow {
    #[setting("Label", default = "New feature")]
    pub label: String,
    #[setting("Cells", default = vec!["yes".into(), "yes".into(), "yes".into()])]
    pub cells: Vec<String>,
}

#[settings(export)]
pub struct ComparisonTable {
    #[setting(
        "Columns",
        default = vec!["Free".into(), "Studio".into(), "Scale".into()],
        length(min = 1),
        help = "Column headings. The feature-label column is unnamed and sits ahead of these."
    )]
    pub columns: Vec<String>,
    #[setting(
        "Rows",
        default = vec![
            ComparisonTableRow {
                label: "Render minutes".into(),
                cells: vec!["60".into(), "2,000".into(), "Unlimited".into()],
            },
            ComparisonTableRow { label: "1080p exports".into(), ..Default::default() },
            ComparisonTableRow {
                label: "4K exports".into(),
                cells: vec!["no".into(), "yes".into(), "yes".into()],
            },
            ComparisonTableRow {
                label: "Team seats".into(),
                cells: vec!["1".into(), "9".into(), "Unlimited".into()],
            },
            ComparisonTableRow {
                label: "Render API".into(),
                cells: vec!["no".into(), "yes".into(), "yes".into()],
            },
            ComparisonTableRow {
                label: "Priority queue".into(),
                cells: vec!["no".into(), "no".into(), "yes".into()],
            },
        ],
        help = "Type yes or no in a cell for a tick or a dash."
    )]
    pub rows: Vec<ComparisonTableRow>,
    #[setting(
        "Title",
        default = "What you get",
        help = "Heading above the table. Omit to drop it."
    )]
    pub title: String,
    #[setting("Highlight a column", default = true)]
    pub highlight: bool,
    #[setting(
        "Highlighted column",
        default = 1,
        range(min = 0, max = 20),
        step = 1,
        help = "Column kept lit throughout, counting from 0.",
        show_if = { "highlight": ["true"] }
    )]
    pub highlight_column: u32,
    #[setting(
        "Highlight badge",
        default = "Most picked",
        help = "Chip pinned beside the title, above the highlighted column."
    )]
    pub highlight_label: String,
    #[setting(
        "Start at (seconds)",
        default = 0.5,
        step = 0.01,
        help = "Second the first row wipes in.",
        advanced
    )]
    pub start_at_seconds: f64,
    #[setting(
        "Row stagger (seconds)",
        default = 0.26,
        step = 0.01,
        help = "Seconds between rows. Cells inside a row trail their label by another 0.07s each.",
        advanced
    )]
    pub row_stagger_seconds: f64,
    #[setting(
        "Accent color",
        default = "#E8B86D",
        help = "Highlight band, its header, and the ticks inside it.",
        check = colour
    )]
    pub accent_color: String,
    #[setting(
        "Theme",
        default = Theme::of(ThemeMode::Dark),
        transform = code_themes,
        widget = "theme"
    )]
    pub theme: Theme,
    #[setting(
        "Speed",
        default = 1.0,
        range(min = 0.25, max = 4),
        step = 0.25,
        help = "Animation speed multiplier."
    )]
    pub speed: f64,
    #[setting(
        "Background",
        widget = "color",
        zero_label = "Theme",
        help = "Leave empty to follow the theme."
    )]
    pub background_color: String,
    #[setting("Tick colour", default = "#9BD4A0", widget = "color", check = colour)]
    pub tick_color: String,
    #[setting(
        "Label column width",
        default = 36.0,
        range(min = 15, max = 60),
        unit = "%",
        advanced
    )]
    pub label_width: f64,
    #[setting(
        "Table width",
        default = 880,
        range(min = 400, max = 1600),
        step = 1,
        unit = "px",
        advanced
    )]
    pub card_width: u32,
    #[setting("Font", widget = "font", advanced)]
    pub font_family: String,
    #[setting("Capital headings", default = true, advanced)]
    pub uppercase_headings: bool,
}

pub fn definition() -> ElementMeta {
    meta::<ComparisonTable>(
        element(
            ElementKind::Component,
            "comparison-table",
            "Plan comparison table",
            "A table comparing plans feature by feature, with one plan highlighted.",
            ElementCategory::Scene,
            120,
        )
        .fonts(&["Inter"])
        .look(NodeLook::of("lucide:table-2")),
    )
}

const CARD_FOR: f64 = 0.46;
const HEAD_AT: f64 = 0.2;
const HEAD_FOR: f64 = 0.4;
const TICK: &str = "M5 12.5 L10 17.5 L19 7";
const CROSS: &str = "M7.5 7.5 L16.5 16.5 M16.5 7.5 L7.5 16.5";
const EDITORIAL: Easing = Easing::Bezier(0.45, 0.0, 0.55, 1.0);
const CARD_SPRING: SpringConfig = SpringConfig {
    damping: 18.0,
    stiffness: 130.0,
    mass: 0.85,
    overshoot_clamping: false,
};

enum Cell<'a> {
    Text(&'a str),
    Tick(bool),
}

fn cell_of(text: &str) -> Cell<'_> {
    match text.trim().to_lowercase().as_str() {
        "yes" => Cell::Tick(true),
        "no" => Cell::Tick(false),
        _ => Cell::Text(text),
    }
}

fn mark(d: &str, color: &str, width: f64, size: f64, progress: f64, join: bool) -> Node {
    let length = path_length(d);
    let mut path = SvgElement::new("path")
        .attr("d", d)
        .attr("stroke", color)
        .attr("stroke-width", width)
        .attr("stroke-linecap", "round")
        .attr("stroke-dasharray", length)
        .attr("stroke-dashoffset", (1.0 - progress) * length);
    if join {
        path = path.attr("stroke-linejoin", "round");
    }
    svg(
        SvgElement::new("svg")
            .attr("width", size)
            .attr("height", size)
            .attr("viewBox", "0 0 24 24")
            .attr("fill", "none")
            .child(path),
        Style::default(),
    )
    .into()
}

fn outline(color: &str, share: f64) -> Edges<BorderSide> {
    Edges::all(BorderSide {
        width: Length::px(1.0),
        style: BorderStyle::Solid,
        color: Some(tinted(color, share)),
    })
}

#[expect(
    clippy::too_many_lines,
    reason = "the table's parts in the film's order, kept together so the port reads against it"
)]
pub fn draw(props: &ComparisonTable, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let (width, height, frame, fps) = (cx.at.stage.width, cx.at.stage.height, cx.frame(), cx.fps());
    let palette = code_palette(props.theme.mode, &[], &props.theme.colors);
    let (side, edge) = safe_padding(width, height);
    let at = |seconds: f64| seconds * fps / props.speed;
    let ease = |from: f64, to: f64, easing: Easing| -> Result<f64, DrawFailure> {
        Ok(interpolate(
            frame,
            &[at(from), at(to)],
            &[0.0, 1.0],
            &Interpolation {
                easing: Some(easing),
                ..Interpolation::clamped()
            },
        )?)
    };
    let highlight = props
        .highlight
        .then(|| usize::try_from(props.highlight_column).ok())
        .flatten();
    let rows = float(props.rows.len());
    let design_u = if height > width {
        (width / 496.0).min(height / 896.0)
    } else {
        (width / 1024.0).min(height / 576.0)
    };
    let title_room = if props.title.is_empty() { 0.0 } else { 44.0 };
    let u = design_u
        .min(f64::mul_add(edge, -2.0, height) / rows.mul_add(46.0, 48.0 + title_room + 44.0));
    let card_in = sprung(frame - at(0.0), fps, CARD_SPRING)?;
    let open = ease(0.0, CARD_FOR, EDITORIAL)?;
    let head_in = ease(HEAD_AT, HEAD_AT + HEAD_FOR, easing_enter())?;
    let card_w = f64::mul_add(side, -2.0, width).min(f64::from(props.card_width) * u);
    let pad = 24.0 * u;
    let label_w = (pad.mul_add(-2.0, card_w)) * props.label_width / 100.0;
    let col_w = (pad.mul_add(-2.0, card_w) - label_w) / float(props.columns.len().max(1));
    let row_h = 46.0 * u;
    let head_h = 44.0 * u;
    let accent = props.accent_color.as_str();
    let mut card = div(code_window(
        &palette,
        CodeWindow {
            width: card_w,
            radius: 20.0 * u,
            open,
            rise: (1.0 - card_in) * 22.0 * u,
            clip_round: 20.0 * u,
            shadow: (26.0 * u, 64.0 * u),
        },
    )
    .padding(Edges::all(Length::px(pad)))
    .position(Position::Relative));
    if let Some(column) = highlight.filter(|c| *c < props.columns.len()) {
        card = card.child(div(Style::default()
            .position(Position::Absolute)
            .left(Length::px(float(column).mul_add(col_w, pad + label_w)))
            .top(Length::px(f64::mul_add(title_room, u, pad)))
            .width(Length::px(col_w))
            .height(Length::px(rows.mul_add(row_h, head_h)))
            .border_radius_all(12.0 * u)
            .background_color(tinted(accent, 7.0))
            .border(outline(accent, 23.0))
            .opacity(head_in)));
    }
    if !props.title.is_empty() {
        let mut title = div(Style::default()
            .height(Length::px(44.0 * u))
            .display(Display::Flex)
            .align_items(Align::Center)
            .justify_content(Align::SpaceBetween)
            .color(css_color(palette.get("fg")))
            .font_size(26.0 * u)
            .font_weight(700.0)
            .letter_spacing(Length::em(-0.02))
            .opacity(head_in))
        .child(span(Style::default()).child(props.title.clone()));
        if !props.highlight_label.is_empty() && highlight.is_some() {
            title = title.child(
                span(
                    Style::default()
                        .padding(Edges::axes(Length::px(4.0 * u), Length::px(10.0 * u)))
                        .border_radius_all(999.0)
                        .background_color(tinted(accent, 13.0))
                        .border(outline(accent, 40.0))
                        .color(css_color(accent))
                        .font_size(14.5 * u)
                        .font_weight(600.0)
                        .letter_spacing(Length::em(0.02)),
                )
                .child(props.highlight_label.clone()),
            );
        }
        card = card.child(title);
    }
    let mut heads = div(Style::default()
        .display(Display::Flex)
        .height(Length::px(head_h))
        .align_items(Align::FlexEnd))
    .child(div(Style::default().width(Length::px(label_w))));
    for (index, name) in props.columns.iter().enumerate() {
        let mut style = Style::default()
            .width(Length::px(col_w))
            .text_align(TextAlign::Center)
            .padding_bottom(Length::px(10.0 * u))
            .color(css_color(if highlight == Some(index) {
                accent
            } else {
                palette.get("dim")
            }))
            .font_size(15.0 * u)
            .font_weight(600.0)
            .letter_spacing(Length::em(0.04))
            .opacity(head_in)
            .translate(Length::ZERO, Length::px((1.0 - head_in) * 8.0 * u));
        if props.uppercase_headings {
            style = style.text_transform(TextTransform::Uppercase);
        }
        heads = heads.child(div(style).child(name.clone()));
    }
    let mut table = div(Style::default().position(Position::Relative)).child(heads);
    for (row_index, row) in props.rows.iter().enumerate() {
        let row_at = float(row_index).mul_add(props.row_stagger_seconds, props.start_at_seconds);
        let row_in = ease(row_at, row_at + 0.4, easing_enter())?;
        let mut line = div(Style::default()
            .display(Display::Flex)
            .align_items(Align::Center)
            .height(Length::px(row_h))
            .border(Edges {
                top: BorderSide {
                    width: Length::px(1.0),
                    style: BorderStyle::Solid,
                    color: Some(css_color(palette.get("border"))),
                },
                right: BorderSide::default(),
                bottom: BorderSide::default(),
                left: BorderSide::default(),
            })
            .opacity(row_in))
        .child(
            div(Style::default()
                .width(Length::px(label_w))
                .color(css_color(palette.get("fg")))
                .font_size(16.5 * u)
                .font_weight(500.0)
                .translate(Length::px((1.0 - row_in) * -12.0 * u), Length::ZERO))
            .child(row.label.clone()),
        );
        for cell_index in 0..props.columns.len() {
            let text = row.cells.get(cell_index).map_or("", String::as_str);
            let cell_at = float(cell_index).mul_add(0.07, row_at + 0.09);
            let cell_in = ease(cell_at, cell_at + 0.3, EDITORIAL)?;
            let lit = highlight == Some(cell_index);
            let content: Node = match cell_of(text) {
                Cell::Text(value) => {
                    let grow = cell_in.mul_add(0.08, 0.92);
                    span(
                        Style::default()
                            .color(css_color(if lit { accent } else { palette.get("fg") }))
                            .font_size(16.0 * u)
                            .font_weight(if lit { 700.0 } else { 600.0 })
                            .font_variant_numeric(NumericVariant {
                                tabular: true,
                                ..NumericVariant::default()
                            })
                            .scale(grow, grow),
                    )
                    .child(value.to_owned())
                    .into()
                }
                Cell::Tick(true) => mark(
                    TICK,
                    if lit { accent } else { &props.tick_color },
                    2.6,
                    22.0 * u,
                    cell_in,
                    true,
                ),
                Cell::Tick(false) => {
                    mark(CROSS, palette.get("faint"), 2.4, 22.0 * u, cell_in, false)
                }
            };
            line = line.child(
                div(Style::default()
                    .width(Length::px(col_w))
                    .display(Display::Flex)
                    .align_items(Align::Center)
                    .justify_content(Align::Center)
                    .opacity(cell_in))
                .child(content),
            );
        }
        table = table.child(line);
    }
    let page = if props.background_color.is_empty() {
        palette.get("page")
    } else {
        &props.background_color
    };
    Ok(div(with_background(
        absolute_fill()
            .font_family(font_or(&props.font_family, FontRole::Main))
            .display(Display::Flex)
            .align_items(Align::Center)
            .justify_content(Align::Center)
            .padding(Edges::axes(Length::px(edge), Length::px(side))),
        page,
    ))
    .child(card.child(table))
    .into())
}
