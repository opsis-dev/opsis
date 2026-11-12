use opsis_macros::settings;
use opsis_scene::clock::js_round;
use opsis_scene::css::with_background;
use opsis_scene::node::{Node, div, span};
use opsis_scene::style::{
    Align, BoxSizing, Display, NumericVariant, Overflow, Position, Style, TextAlign, TextOverflow,
    WhiteSpace,
};
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::Length;
use opsis_types::looks::NodeLook;
use opsis_types::numbers::{exact_eq, float, truncate};

use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::number_text::format_number;
use crate::elements::shared::{Colour, Corner, NumberFormat};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct BarChartRaceSerie {
    #[setting("Label", default = "Partners")]
    pub label: String,
    #[setting("Values")]
    pub values: Vec<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Color", widget = "color", zero_label = "None", "type" = "string")]
    pub color: Option<String>,
}

#[settings(export)]
pub struct BarChartRace {
    #[setting(
        "Series",
        default = vec![
            BarChartRaceSerie {
                label: "Mobile".into(),
                values: vec![42.0, 58.0, 66.0, 72.0, 78.0, 84.0],
                ..Default::default()
            },
            BarChartRaceSerie {
                label: "Web".into(),
                values: vec![18.0, 34.0, 57.0, 76.0, 92.0, 108.0],
                ..Default::default()
            },
            BarChartRaceSerie {
                label: "Desktop".into(),
                values: vec![30.0, 40.0, 47.0, 52.0, 57.0, 61.0],
                ..Default::default()
            },
            BarChartRaceSerie {
                label: "API".into(),
                values: vec![12.0, 22.0, 36.0, 50.0, 63.0, 79.0],
                ..Default::default()
            },
            BarChartRaceSerie {
                label: "Email".into(),
                values: vec![24.0, 28.0, 31.0, 35.0, 38.0, 42.0],
                ..Default::default()
            },
            BarChartRaceSerie { values: vec![8.0, 12.0, 17.0, 21.0, 26.0, 30.0], ..Default::default() },
        ]
    )]
    pub series: Vec<BarChartRaceSerie>,
    #[setting(
        "Steps",
        default = vec!["Q1".into(), "Q2".into(), "Q3".into(), "Q4".into(), "Q5".into(), "Q6".into()],
        help = "Keyframe captions, e.g. years. Shown as the running clock."
    )]
    pub steps: Vec<String>,
    #[setting(
        "Time per step",
        default = 18,
        range(min = 1, max = 300),
        step = 1,
        unit = "frames",
        help = "Frames spent travelling between two keyframes."
    )]
    pub frames_per_step: u32,
    #[setting(
        "Visible rows",
        default = 6,
        range(min = 1, max = 20),
        step = 1,
        help = "How many rows stay on the board. Below that, series fade out.",
        advanced
    )]
    pub visible_rows: u32,
    #[setting(
        "Width",
        default = 1200.0,
        help = "Overall width, label column included."
    )]
    pub width: f64,
    #[setting(
        "Row height",
        default = 76.0,
        help = "Bar height. Type scales off it.",
        advanced
    )]
    pub row_height: f64,
    #[setting("Gap", default = 14.0, help = "Space between rows.", advanced)]
    pub gap: f64,
    #[setting(
        "Label width",
        default = 240.0,
        help = "Width reserved for the row labels.",
        advanced
    )]
    pub label_width: f64,
    #[setting(
        "Show step label",
        default = true,
        help = "Large step caption in the bottom-right corner."
    )]
    pub show_step_label: bool,
    #[setting(
        "Colors",
        default = vec![
            "#e8b86d".into(),
            "#2dd4bf".into(),
            "#f472b6".into(),
            "#8b8bf5".into(),
            "#f59e0b".into(),
            "#5eead4".into(),
        ],
        length(min = 1)
    )]
    pub colors: Vec<Colour>,
    #[setting("Text color", default = "#fafafa", check = colour, advanced)]
    pub ink_color: String,
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
        default = 22.0,
        range(min = 0, max = 50),
        unit = "%",
        advanced
    )]
    pub corner_radius: f64,
    #[setting(
        "Step label position",
        default = Corner::BottomRight,
        advanced
    )]
    pub step_label_position: Corner,
    #[setting(
        "Step label size",
        default = 90.0,
        range(min = 20, max = 300),
        unit = "%",
        advanced
    )]
    pub step_label_size: f64,
    #[setting(
        "Step label opacity",
        default = 22.0,
        range(min = 0, max = 100),
        unit = "%",
        advanced
    )]
    pub step_label_opacity: f64,
    #[setting("Font", widget = "font", advanced)]
    pub font_family: String,
}

pub fn definition() -> ElementMeta {
    meta::<BarChartRace>(
        element(
            ElementKind::Component,
            "bar-chart-race",
            "Bar chart race",
            "Ranked bars that grow and swap places as the values change over time.",
            ElementCategory::Data,
            120,
        )
        .design(1920, 1080)
        .look(NodeLook::of("lucide:chart-bar-decreasing")),
    )
}

fn value_at(values: &[f64], time: f64) -> f64 {
    let Some(last) = values.len().checked_sub(1) else {
        return 0.0;
    };
    let clamped = time.clamp(0.0, float(last));
    let lower: usize = truncate(clamped.floor());
    let upper = (lower + 1).min(last);
    let t = clamped - float(lower);
    (values[upper] - values[lower]).mul_add(t, values[lower])
}

fn ranks_at(series: &[BarChartRaceSerie], step: usize) -> Vec<f64> {
    let mut live: Vec<(usize, f64)> = series
        .iter()
        .enumerate()
        .filter_map(|(index, entry)| entry.values.get(step).map(|v| (index, *v)))
        .collect();
    live.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    (0..series.len())
        .map(|index| float(live.iter().position(|e| e.0 == index).unwrap_or(live.len())))
        .collect()
}

pub fn draw(props: &BarChartRace, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let frame = cx.frame();
    let series = &props.series;
    let step_count = series.iter().map(|s| s.values.len()).max().unwrap_or(0);
    let last_step = step_count.saturating_sub(1);
    let time = (frame / f64::from(props.frames_per_step.max(1))).clamp(0.0, float(last_step));
    let presence: Vec<(f64, f64, String)> = series
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            (
                value_at(&entry.values, time),
                (float(entry.values.len()) - time).clamp(0.0, 1.0),
                entry.color.clone().unwrap_or_else(|| {
                    props
                        .colors
                        .get(index % props.colors.len().max(1))
                        .map_or_default(|c| c.0.clone())
                }),
            )
        })
        .collect();
    let highest = presence
        .iter()
        .filter(|p| p.1 > 0.0)
        .map(|p| p.0)
        .fold(0.0, f64::max);
    let leader = if exact_eq(highest, 0.0) || highest.is_nan() {
        1.0
    } else {
        highest
    };
    let lower_step: usize = truncate(time.floor());
    let lower = ranks_at(series, lower_step);
    let upper = ranks_at(series, (lower_step + 1).min(last_step));
    let blend = time - float(lower_step);
    let eased = blend * blend * 2.0_f64.mul_add(-blend, 3.0);
    let row = props.row_height;
    let pitch = row + props.gap;
    let bar_width = props.width - props.label_width;
    let rows_shown = f64::from(props.visible_rows);
    let mut rows = Vec::new();
    for (index, entry) in series.iter().enumerate() {
        let rank = (upper[index] - lower[index]).mul_add(eased, lower[index]);
        let (current, present, color) = &presence[index];
        let visibility = (rows_shown - rank).clamp(0.0, 1.0) * present;
        if visibility <= 0.0 {
            continue;
        }
        rows.push(
            div(Style::default()
                .position(Position::Absolute)
                .top(Length::ZERO)
                .left(Length::ZERO)
                .width(props.width)
                .height(row)
                .display(Display::Flex)
                .align_items(Align::Center)
                .gap(props.gap)
                .translate(Length::ZERO, Length::px(rank * pitch))
                .opacity(visibility))
            .child(
                span(
                    Style::default()
                        .width(props.label_width - props.gap)
                        .text_align(TextAlign::Right)
                        .color(cx.color(&props.label_color))
                        .font_size(row * 0.32)
                        .font_weight(600.0)
                        .white_space(WhiteSpace::Nowrap)
                        .overflow(Overflow::Hidden)
                        .text_overflow(TextOverflow::Ellipsis),
                )
                .child(entry.label.as_str()),
            )
            .child(
                div(with_background(
                    Style::default()
                        .height(row)
                        .width((row * 0.6).max(current / leader * bar_width))
                        .border_radius_all(row * props.corner_radius / 100.0)
                        .display(Display::Flex)
                        .align_items(Align::Center)
                        .justify_content(Align::FlexEnd)
                        .padding_right(row * 0.28)
                        .box_sizing(BoxSizing::BorderBox),
                    color,
                ))
                .child(
                    span(
                        Style::default()
                            .color(cx.color(&props.bar_ink_color))
                            .font_size(row * 0.3)
                            .font_weight(700.0)
                            .font_variant_numeric(NumericVariant {
                                tabular: true,
                                ..NumericVariant::default()
                            }),
                    )
                    .child(format_number(*current, props.number_format)),
                ),
            )
            .into(),
        );
    }
    let step_index: usize = truncate(js_round(time).max(0.0));
    if let Some(label) = props
        .steps
        .get(step_index)
        .filter(|_| props.show_step_label)
        .filter(|l| !l.is_empty())
    {
        let mut corner = Style::default().position(Position::Absolute);
        corner = match props.step_label_position {
            Corner::TopLeft => corner.left(Length::ZERO).top(Length::ZERO),
            Corner::TopRight => corner.right(Length::ZERO).top(Length::ZERO),
            Corner::BottomLeft => corner.left(Length::ZERO).bottom(Length::ZERO),
            Corner::BottomRight => corner.right(Length::ZERO).bottom(Length::ZERO),
        };
        rows.push(
            div(corner
                .color(cx.color(&props.ink_color))
                .font_size(row * props.step_label_size / 100.0)
                .font_weight(700.0)
                .opacity(props.step_label_opacity / 100.0)
                .font_variant_numeric(NumericVariant {
                    tabular: true,
                    ..NumericVariant::default()
                }))
            .child(label.as_str())
            .into(),
        );
    }
    Ok(div(absolute_fill()
        .align_items(Align::Center)
        .justify_content(Align::Center)
        .text_align(TextAlign::Center))
    .child(
        div(Style::default()
            .position(Position::Relative)
            .width(props.width)
            .height(pitch * rows_shown)
            .opacity(1.0)
            .font_family(font_or(&props.font_family, FontRole::Main)))
        .children(rows),
    )
    .into())
}
