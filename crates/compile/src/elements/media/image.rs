use std::collections::HashMap;

use opsis_macros::settings;
use opsis_scene::measure::Measure;
use opsis_scene::node::{Node, div, img};
use opsis_scene::paint::Transform;
use opsis_scene::style::{Display, ObjectFit, Style};
use opsis_scene::values::{Length, Num};
use opsis_types::looks::NodeLook;
use serde::Deserialize;

use crate::elements::anim::zoom_over;
use crate::elements::backgrounds::backgrounded;
use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::shared::Fit;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

const US_PER_S: f64 = 1_000_000.0;

#[settings(export)]
pub struct Image {
    #[setting("Image", widget = "asset", accept = "image/*")]
    pub src: String,
    #[setting("Fit", default = Fit::Contain)]
    pub fit: Fit,
    #[setting(
        "Background",
        widget = "color",
        zero_label = "None",
        check = colour_or_empty
    )]
    pub background: String,
    #[setting("Slow push-in")]
    pub ken_burns: bool,
    #[setting(
        "Push-in zoom",
        default = 1.06,
        range(min = 1, max = 1.5),
        step = 0.01,
        unit = "×",
        show_if = { "kenBurns": ["true"] },
        advanced
    )]
    pub ken_burns_zoom: f64,
}

pub fn definition() -> ElementMeta {
    meta::<Image>(
        element(
            ElementKind::Component,
            "image",
            "Image",
            "Shows an image from your project.",
            ElementCategory::Media,
            90,
        )
        .look(NodeLook::of("lucide:image")),
    )
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Shown {
    #[serde(flatten)]
    pub settings: Image,
    #[serde(default)]
    pub footage_urls: HashMap<String, String>,
}

pub const fn object_fit(fit: Fit) -> ObjectFit {
    match fit {
        Fit::Contain => ObjectFit::Contain,
        Fit::Cover => ObjectFit::Cover,
        Fit::Fill => ObjectFit::Fill,
    }
}

pub fn footage_frame_url(base: &str, time_s: f64, measure: &dyn Measure) -> String {
    let micros = opsis_scene::clock::js_round(time_s * US_PER_S).max(0.0);
    let shown = measure.footage_time(base, micros);
    format!("{base}{}", opsis_scene::random::js_number_text(shown))
}

pub fn filled(source: String, fit: Fit, transform: Option<Vec<Transform>>) -> Node {
    let mut style = Style::default()
        .width(Length::percent(100.0))
        .height(Length::percent(100.0))
        .object_fit(object_fit(fit))
        .display(Display::Block);
    if let Some(transform) = transform {
        style = style.transform(transform);
    }
    img(source, style).into()
}

pub fn draw(props: &Shown, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let image = &props.settings;
    if image.src.is_empty() {
        return Ok(div(Style::default()).into());
    }
    let scale = if image.ken_burns {
        zoom_over(cx.frame(), cx.duration(), image.ken_burns_zoom)?
    } else {
        1.0
    };
    let transform = vec![Transform::Scale(Num(scale), Num(scale), Num(1.0))];
    let source = props.footage_urls.get("src").map_or_else(
        || image.src.clone(),
        |url| footage_frame_url(url, 0.0, cx.measure),
    );
    Ok(backgrounded(
        &image.background,
        vec![filled(source, image.fit, Some(transform))],
    ))
}
