use opsis_scene::color::Color;
use opsis_scene::interpolate::{Interpolation, interpolate};
use opsis_scene::node::{Node, div};
use opsis_scene::paint::{ColorStop, Gradient};
use opsis_scene::style::{Align, BorderSide, BorderStyle, FlexDirection, Display, Style, TextAlign};
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::{Angle, Edges, Length};
use serde::Deserialize;

use crate::elements::draw::{Cx, DrawFailure, css_color};
use crate::elements::fonts::{FontRole, font_or};

const SWEEP_FRAMES: f64 = 90.0;
const SWEEP_FROM_PERCENT: f64 = -30.0;
const SWEEP_TO_PERCENT: f64 = 130.0;
const SWEEP_BAND_PERCENT: f64 = 20.0;
const SWEEP_SHINE: &str = "rgba(255,255,255,0.05)";
const BORDER_COLOR: &str = "rgba(255,255,255,0.18)";
const BORDER_WIDTH: f64 = 3.0;
const CARD_RADIUS: f64 = 28.0;
const CARD_GAP: f64 = 18.0;
const TITLE_SIZE: f64 = 56.0;
const TITLE_WEIGHT: f64 = 650.0;
const SUBTITLE_SIZE: f64 = 30.0;
const PLACEHOLDER_COLOR: &str = "#0c0c10";
const TITLE_COLOR: &str = "#e4e4e7";
const SUBTITLE_COLOR: &str = "#a1a1aa";
const DEFAULT_TITLE: &str = "Not captured yet";

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Placeholder {
    pub title: String,
    pub subtitle: String,
}

impl Default for Placeholder {
    fn default() -> Self {
        Self {
            title: DEFAULT_TITLE.to_owned(),
            subtitle: String::new(),
        }
    }
}

pub fn draw(props: &Placeholder, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let sweep = interpolate(
        cx.frame() % SWEEP_FRAMES,
        &[0.0, SWEEP_FRAMES],
        &[SWEEP_FROM_PERCENT, SWEEP_TO_PERCENT],
        &Interpolation::default(),
    )?;
    let band = Gradient::linear(
        Angle::deg(100.0),
        vec![
            ColorStop::at(Color::TRANSPARENT, Length::percent(sweep - SWEEP_BAND_PERCENT)),
            ColorStop::at(css_color(SWEEP_SHINE), Length::percent(sweep)),
            ColorStop::at(Color::TRANSPARENT, Length::percent(sweep + SWEEP_BAND_PERCENT)),
        ],
    );
    let mut card = div(Style::default()
        .width(Length::percent(72.0))
        .height(Length::percent(64.0))
        .border_radius_all(CARD_RADIUS)
        .border(Edges::all(BorderSide {
            width: Length::px(BORDER_WIDTH),
            style: BorderStyle::Dashed,
            color: Some(css_color(BORDER_COLOR)),
        }))
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .align_items(Align::Center)
        .justify_content(Align::Center)
        .gap(CARD_GAP)
        .background(vec![band.layer()]))
    .child(
        div(Style::default()
            .color(css_color(TITLE_COLOR))
            .font_size(TITLE_SIZE)
            .font_weight(TITLE_WEIGHT))
        .child(props.title.as_str()),
    );
    if !props.subtitle.is_empty() {
        card = card.child(
            div(Style::default()
                .color(css_color(SUBTITLE_COLOR))
                .font_size(SUBTITLE_SIZE)
                .max_width(Length::percent(80.0))
                .text_align(TextAlign::Center))
            .child(props.subtitle.as_str()),
        );
    }
    Ok(div(absolute_fill()
        .background_color(css_color(PLACEHOLDER_COLOR))
        .align_items(Align::Center)
        .justify_content(Align::Center)
        .font_family(font_or("", FontRole::Main)))
    .child(card)
    .into())
}
