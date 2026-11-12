use opsis_macros::settings;
use opsis_scene::clock::js_round;
use opsis_scene::color::Color;
use opsis_scene::css::with_background;
use opsis_scene::node::{Node, div, span};
use opsis_scene::random::js_number_text;
use opsis_scene::style::{Align, BoxSizing, Display, NumericVariant, Style, TextAlign, WhiteSpace};
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::Length;
use opsis_types::looks::NodeLook;
use opsis_types::numbers::{exact_eq, float};

use crate::elements::anim::clamped_eased;
use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::number_text::{Direction, format_number, read_delta};
use crate::elements::shared::NumberFormat;
use crate::elements::text_split::easing_enter;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct ComparisonBarsRow {
    #[setting("Label", default = "Manual steps")]
    pub label: String,
    #[setting("Before", default = 62.0)]
    pub before: f64,
    #[setting("After", default = 18.0)]
    pub after: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Delta", "type" = "string")]
    pub delta: Option<String>,
}

#[settings(export)]
pub struct ComparisonBars {
    #[setting(
        "Rows",
        default = vec![
            ComparisonBarsRow {
                label: "Setup time (min)".into(),
                before: 240.0,
                after: 96.0,
                ..Default::default()
            },
            ComparisonBarsRow {
                label: "Tickets closed / week".into(),
                before: 120.0,
                after: 310.0,
                ..Default::default()
            },
            ComparisonBarsRow {
                label: "Active teams".into(),
                before: 34.0,
                after: 88.0,
                ..Default::default()
            },
            ComparisonBarsRow::default(),
        ],
        help = "The delta is computed when omitted."
    )]
    pub rows: Vec<ComparisonBarsRow>,
    #[setting(
        "Width",
        default = 1100.0,
        help = "Overall width, label column included."
    )]
    pub width: f64,
    #[setting(
        "Row height",
        default = 84.0,
        help = "Height of one pair, both bars and their gap included.",
        advanced
    )]
    pub row_height: f64,
    #[setting("Gap", default = 26.0, help = "Space between rows.", advanced)]
    pub gap: f64,
    #[setting(
        "Label width",
        default = 280.0,
        help = "Width reserved for the row labels.",
        advanced
    )]
    pub label_width: f64,
    #[setting(
        "Before color",
        default = "rgba(250,250,250,0.22)",
        help = "Baseline bar. Muted on purpose: it is the thing being beaten.",
        check = colour
    )]
    pub before_color: String,
    #[setting(
        "After color",
        default = "#e8b86d",
        help = "New-value bar, and the colour of a rising delta.",
        check = colour
    )]
    pub after_color: String,
    #[setting(
        "Decrease color",
        default = "#f472b6",
        help = "Delta colour when the change is a fall.",
        check = colour,
        advanced
    )]
    pub down_color: String,
    #[setting(
        "Series labels",
        default = vec!["Before".into(), "After".into()],
        length(max = 2),
        help = "Legend above the rows: the before bar, then the after bar."
    )]
    pub series_labels: Vec<String>,
    #[setting(
        "Show change",
        default = true,
        help = "Percentage-change chip at the end of the second bar."
    )]
    pub show_delta: bool,
    #[setting(
        "Animation length",
        default = 30,
        range(min = 1, max = 300),
        step = 1,
        unit = "frames",
        help = "Length of one bar's growth.",
        advanced
    )]
    pub duration_in_frames: u32,
    #[setting(
        "Stagger",
        default = 12,
        unit = "frames",
        help = "Frames between one row and the next.",
        advanced
    )]
    pub stagger_in_frames: u32,
    #[setting(
        "Pair offset",
        default = 6,
        unit = "frames",
        help = "Frames the second bar trails the first by.",
        advanced
    )]
    pub pair_offset_in_frames: u32,
    #[setting("Bar text color", default = "#0b0b10", check = colour, advanced)]
    pub bar_ink_color: String,
    #[setting("Label color", default = "rgba(250,250,250,0.55)", check = colour, advanced)]
    pub label_color: String,
    #[setting(
        "Number format",
        default = NumberFormat::Compact,
        advanced
    )]
    pub number_format: NumberFormat,
    #[setting(
        "Corner rounding",
        default = 28.0,
        range(min = 0, max = 50),
        unit = "%",
        advanced
    )]
    pub corner_radius: f64,
    #[setting("Show before value", advanced)]
    pub show_before_value: bool,
    #[setting("Font", widget = "font", advanced)]
    pub font_family: String,
}

pub fn definition() -> ElementMeta {
    meta::<ComparisonBars>(
        element(
            ElementKind::Component,
            "comparison-bars",
            "Before and after bars",
            "Pairs of bars comparing before and after, with the change called out.",
            ElementCategory::Data,
            120,
        )
        .design(1920, 1080)
        .look(NodeLook::of("lucide:chart-bar")),
    )
}

const DELTA_SHIFT: f64 = -12.0;

fn value_text(text: String, color: Color, size: f64, opacity: f64) -> Node {
    span(
        Style::default()
            .color(color)
            .font_size(size)
            .font_weight(700.0)
            .font_variant_numeric(NumericVariant {
                tabular: true,
                ..NumericVariant::default()
            })
            .opacity(opacity),
    )
    .child(text)
    .into()
}

fn bar(width: f64, height: f64, color: Color, radius: f64) -> Style {
    Style::default()
        .height(height)
        .width(width)
        .background_color(color)
        .border_radius_all(radius)
        .display(Display::Flex)
        .align_items(Align::Center)
        .justify_content(Align::FlexEnd)
        .padding_right(height * 0.32)
        .box_sizing(BoxSizing::BorderBox)
}

#[expect(
    clippy::too_many_lines,
    reason = "a row's markup follows the film's one view function in order"
)]
pub fn draw(props: &ComparisonBars, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let frame = cx.frame();
    let radius = |h: f64| h * props.corner_radius / 100.0;
    let highest = props
        .rows
        .iter()
        .map(|r| r.before.max(r.after))
        .fold(0.0, f64::max);
    let peak = if highest > 0.0 { highest } else { 1.0 };
    let bar_width = props.width - props.label_width;
    let bar_height = (props.row_height - 8.0) / 2.0;
    let chip = bar_height * 0.5;
    let duration = f64::from(props.duration_in_frames);
    let grow =
        |start: f64| clamped_eased(frame, [start, start + duration], [0.0, 1.0], easing_enter());
    let label_color = cx.color(&props.label_color);
    let ink = cx.color(&props.bar_ink_color);
    let mut grid = div(Style::default()
        .width(props.width)
        .display(Display::Grid)
        .gap(props.gap)
        .font_family(font_or(&props.font_family, FontRole::Main)));
    let legend: Vec<Node> = [&props.before_color, &props.after_color]
        .iter()
        .enumerate()
        .filter_map(|(i, swatch)| {
            let text = props.series_labels.get(i).filter(|t| !t.is_empty())?;
            Some(
                span(
                    Style::default()
                        .display(Display::Flex)
                        .align_items(Align::Center)
                        .gap(8.0),
                )
                .child(span(with_background(
                    Style::default()
                        .width(bar_height * 0.42)
                        .height(bar_height * 0.42)
                        .border_radius_all(4.0),
                    swatch,
                )))
                .child(text.as_str())
                .into(),
            )
        })
        .collect();
    if !legend.is_empty() {
        grid = grid.child(
            div(Style::default()
                .display(Display::Flex)
                .gap(props.gap)
                .margin_left(props.label_width)
                .color(label_color.clone())
                .font_size(bar_height * 0.46)
                .font_weight(600.0)
                .opacity(1.0))
            .children(legend),
        );
    }
    for (index, row) in props.rows.iter().enumerate() {
        let start = float(index) * f64::from(props.stagger_in_frames);
        let before = grow(start)?;
        let after = grow(start + f64::from(props.pair_offset_in_frames))?;
        let change = if exact_eq(row.before, 0.0) {
            0.0
        } else {
            (row.after - row.before) / row.before * 100.0
        };
        let given = row.delta.as_deref().filter(|d| !d.is_empty());
        let delta = given.map_or_else(
            || {
                if exact_eq(row.before, 0.0) {
                    String::new()
                } else {
                    let sign = if change >= 0.0 { "+" } else { "\u{2212}" };
                    format!("{sign}{}%", js_number_text(js_round(change).abs()))
                }
            },
            str::to_owned,
        );
        let falls = given.map_or(change < 0.0, |d| read_delta(d).0 == Direction::Down);
        let mut before_bar = div(bar(
            row.before / peak * bar_width * before,
            bar_height,
            cx.color(&props.before_color),
            radius(bar_height),
        ));
        if props.show_before_value {
            before_bar = before_bar.child(value_text(
                format_number(row.before, props.number_format),
                ink.clone(),
                bar_height * 0.46,
                ((before - 0.55) / 0.25).clamp(0.0, 1.0),
            ));
        }
        let after_bar = div(bar(
            row.after / peak * bar_width * after,
            bar_height,
            cx.color(&props.after_color),
            radius(bar_height),
        ))
        .child(value_text(
            format_number(row.after, props.number_format),
            ink.clone(),
            bar_height * 0.46,
            ((after - 0.55) / 0.25).clamp(0.0, 1.0),
        ));
        let mut after_row = div(Style::default()
            .display(Display::Flex)
            .align_items(Align::Center)
            .gap(14.0))
        .child(after_bar);
        if props.show_delta && !delta.is_empty() {
            let shown = ((after - 0.86) / 0.14).clamp(0.0, 1.0);
            after_row = after_row.child(
                span(
                    Style::default()
                        .color(cx.color(if falls {
                            &props.down_color
                        } else {
                            &props.after_color
                        }))
                        .font_size(chip)
                        .font_weight(700.0)
                        .font_variant_numeric(NumericVariant {
                            tabular: true,
                            ..NumericVariant::default()
                        })
                        .white_space(WhiteSpace::Nowrap)
                        .opacity(shown)
                        .translate(Length::px((1.0 - shown) * DELTA_SHIFT), Length::ZERO),
                )
                .child(delta),
            );
        }
        grid = grid.child(
            div(Style::default()
                .display(Display::Flex)
                .align_items(Align::Center)
                .gap(Length::ZERO)
                .opacity(1.0)
                .translate(Length::ZERO, Length::ZERO))
            .child(
                span(
                    Style::default()
                        .width(props.label_width)
                        .padding_right(20.0)
                        .box_sizing(BoxSizing::BorderBox)
                        .color(label_color.clone())
                        .font_size(bar_height * 0.5)
                        .font_weight(600.0)
                        .text_align(TextAlign::Right),
                )
                .child(row.label.as_str()),
            )
            .child(
                div(Style::default()
                    .display(Display::Grid)
                    .gap(8.0)
                    .flex_grow(1.0)
                    .flex_shrink(1.0)
                    .flex_basis(Length::percent(0.0)))
                .child(before_bar)
                .child(after_row),
            ),
        );
    }
    Ok(div(absolute_fill()
        .align_items(Align::Center)
        .justify_content(Align::Center)
        .text_align(TextAlign::Center))
    .child(grid)
    .into())
}
