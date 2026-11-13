use std::collections::HashMap;

use opsis_macros::settings;
use opsis_scene::node::{Node, div};
use opsis_scene::style::Style;
use opsis_scene::timeline::absolute_fill;
use opsis_types::looks::NodeLook;
use serde::Deserialize;

use crate::elements::backgrounds::backgrounded;
use crate::elements::draw::{Cx, DrawFailure, css_color};
use crate::elements::entry::{element, meta};
use crate::elements::media::image::{filled, footage_frame_url};
use crate::elements::shared::Fit;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct Video {
    #[setting("Video", widget = "asset", accept = "video/*")]
    pub src: String,
    #[setting("Fit", default = Fit::Cover)]
    pub fit: Fit,
    #[setting("Loop")]
    pub r#loop: bool,
    #[setting("Muted", default = true)]
    pub muted: bool,
    #[setting("Volume", default = 1.0, range(min = 0, max = 2), step = 0.05)]
    pub volume: f64,
    #[setting(
        "Speed",
        default = 1.0,
        range(min = 0.25, max = 4),
        step = 0.05,
        advanced
    )]
    pub playback_rate: f64,
    #[setting("Start at", range(min = 0, max = 3_600_000), step = 100, unit = "ms")]
    pub trim_before_ms: u32,
    #[setting(
        "Background",
        widget = "color",
        zero_label = "None",
        check = colour_or_empty,
        help = "Shows around the video when Fit leaves room.",
        advanced
    )]
    pub background: String,
    #[setting(
        "Dim",
        range(min = 0, max = 1),
        step = 0.05,
        help = "Darkens the video, so text laid over it reads."
    )]
    pub dim: f64,
    #[setting("Dim colour", default = "#000000", widget = "color", check = colour, advanced)]
    pub dim_color: String,
}

pub fn definition() -> ElementMeta {
    meta::<Video>(
        element(
            ElementKind::Component,
            "video",
            "Video",
            "Plays a video from your project.",
            ElementCategory::Media,
            150,
        )
        .look(NodeLook::of("lucide:video")),
    )
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Shown {
    #[serde(flatten)]
    pub settings: Video,
    #[serde(default)]
    pub footage_urls: HashMap<String, String>,
}

pub fn draw(props: &Shown, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let video = &props.settings;
    let Some(url) = props
        .footage_urls
        .get("src")
        .filter(|_| !video.src.is_empty())
    else {
        return Ok(div(Style::default()).into());
    };
    let time_s = cx.frame() / cx.fps() * video.playback_rate;
    let mut children = vec![filled(footage_frame_url(url, time_s), video.fit, None)];
    if video.dim > 0.0 {
        children.push(
            div(absolute_fill()
                .background_color(css_color(&video.dim_color))
                .opacity(video.dim))
            .into(),
        );
    }
    Ok(backgrounded(&video.background, children))
}
