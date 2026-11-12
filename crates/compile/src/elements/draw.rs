use opsis_scene::clock::AUTHORED_FPS;
use opsis_scene::color::Color;
use opsis_scene::interpolate::InterpolateError;
use opsis_scene::measure::Measure;
use opsis_scene::node::Node;
use opsis_scene::spring::SpringError;
use opsis_scene::timeline::{At, DrawError, Drawn};
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

use crate::elements::{background, data, overlay, placeholder, text, transition};

#[derive(Clone, Copy)]
pub struct Cx<'a> {
    pub at: &'a At,
    pub measure: &'a dyn Measure,
}

impl std::fmt::Debug for Cx<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cx")
            .field("at", &self.at)
            .finish_non_exhaustive()
    }
}

impl Cx<'_> {
    pub fn color(&self, css: &str) -> Color {
        css_color(css)
    }

    pub const fn frame(&self) -> f64 {
        self.at.clock.frame
    }

    pub fn fps(&self) -> f64 {
        f64::from(self.at.clock.fps)
    }

    pub const fn duration(&self) -> f64 {
        self.at.clock.duration_in_frames
    }

    pub fn authored_fps() -> f64 {
        f64::from(AUTHORED_FPS)
    }

    pub fn authored_frame(&self) -> f64 {
        self.frame() * Self::authored_fps() / self.fps()
    }

    pub fn authored_duration(&self) -> f64 {
        self.duration() * Self::authored_fps() / self.fps()
    }

    pub fn real_frames(&self, authored: f64) -> f64 {
        authored * self.fps() / Self::authored_fps()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DrawFailure {
    #[error(transparent)]
    Interpolate(#[from] InterpolateError),
    #[error(transparent)]
    Spring(#[from] SpringError),
    #[error(transparent)]
    Element(DrawError),
    #[error("Couldn't read the animation {0}.")]
    Animation(String),
}

pub fn css_color(css: &str) -> Color {
    Color::css(css).unwrap_or_default()
}

type Draw<T> = fn(&T, &Cx<'_>) -> Result<Node, DrawFailure>;

fn drawn<T: DeserializeOwned>(
    component: &str,
    props: &Map<String, Value>,
    cx: &Cx<'_>,
    draw: Draw<T>,
) -> Drawn {
    let settings = serde_json::from_value::<T>(Value::Object(props.clone()))
        .map_err(|e| DrawError::new(component, e))?;
    draw(&settings, cx)
        .map(Some)
        .map_err(|e| DrawError::new(component, e))
}

#[derive(Debug, Clone)]
pub struct Transition {
    pub entering: bool,
    pub progress: f64,
    pub duration: f64,
    pub children: Vec<Node>,
}

type Present<T> = fn(&T, Transition, &Cx<'_>) -> Result<Node, DrawFailure>;

fn presented<T: DeserializeOwned>(
    kind: &str,
    params: Option<&Map<String, Value>>,
    transition: Transition,
    cx: &Cx<'_>,
    present: Present<T>,
) -> Drawn {
    let settings = serde_json::from_value::<T>(Value::Object(params.cloned().unwrap_or_default()))
        .map_err(|e| DrawError::new(kind, e))?;
    present(&settings, transition, cx)
        .map(Some)
        .map_err(|e| DrawError::new(kind, e))
}

pub fn presentation(
    kind: &str,
    params: Option<&Map<String, Value>>,
    transition: Transition,
    cx: &Cx<'_>,
) -> Drawn {
    match kind {
        "fade" => presented(kind, params, transition, cx, transition::fade::present),
        "slide" => presented(kind, params, transition, cx, transition::slide::present),
        "wipe" => presented(kind, params, transition, cx, transition::wipe::present),
        "flip" => presented(kind, params, transition, cx, transition::flip::present),
        "clock-wipe" => presented(kind, params, transition, cx, transition::clock_wipe::present),
        "iris" => presented(kind, params, transition, cx, transition::iris::present),
        "whip-pan" => presented(kind, params, transition, cx, transition::whip_pan::present),
        "push-through" => {
            presented(kind, params, transition, cx, transition::push_through::present)
        }
        "zoom-blur" => presented(kind, params, transition, cx, transition::zoom_blur::present),
        "focus-pull" => presented(kind, params, transition, cx, transition::focus_pull::present),
        "page-turn" => presented(kind, params, transition, cx, transition::page_turn::present),
        "grid-pixelate-wipe" => {
            presented(kind, params, transition, cx, transition::grid_pixelate_wipe::present)
        }
        "transition-blinds" => {
            presented(kind, params, transition, cx, transition::transition_blinds::present)
        }
        "frosted-glass-wipe" => {
            presented(kind, params, transition, cx, transition::frosted_glass_wipe::present)
        }
        "chromatic-aberration-wipe" => presented(
            kind,
            params,
            transition,
            cx,
            transition::chromatic_aberration_wipe::present,
        ),
        "lens-zoom" => presented(kind, params, transition, cx, transition::lens_zoom::present),
        "transition-morph-shape" => {
            presented(kind, params, transition, cx, transition::transition_morph_shape::present)
        }
        _ => Ok(None),
    }
}

pub fn element(component: &str, props: &Map<String, Value>, cx: &Cx<'_>) -> Drawn {
    match component {
        "placeholder" => drawn(component, props, cx, placeholder::draw),
        "solid" => drawn(component, props, cx, background::solid::draw),
        "starburst" => drawn(component, props, cx, background::starburst::draw),
        "gradient" => drawn(component, props, cx, background::gradient::draw),
        "paper" => drawn(component, props, cx, background::paper::draw),
        "noise" => drawn(component, props, cx, background::noise::draw),
        "aurora-bg" => drawn(component, props, cx, background::aurora_bg::draw),
        "tracking-in" => drawn(component, props, cx, text::tracking_in::draw),
        "staggered-fade-up" => drawn(component, props, cx, text::staggered_fade_up::draw),
        "soft-blur-in" => drawn(component, props, cx, text::soft_blur_in::draw),
        "typewriter" => drawn(component, props, cx, text::typewriter::draw),
        "matrix-decode" => drawn(component, props, cx, text::matrix_decode::draw),
        "word-push" => drawn(component, props, cx, text::word_push::draw),
        "mask-reveal-up" => drawn(component, props, cx, text::mask_reveal_up::draw),
        "line-by-line-slide" => drawn(component, props, cx, text::line_by_line_slide::draw),
        "slot-machine-roll" => drawn(component, props, cx, text::slot_machine_roll::draw),
        "neon-flicker-text" => drawn(component, props, cx, text::neon_flicker_text::draw),
        "rgb-glitch-text" => drawn(component, props, cx, text::rgb_glitch_text::draw),
        "handwriting-text" => drawn(component, props, cx, text::handwriting_text::draw),
        "kinetic-headline" => drawn(component, props, cx, text::kinetic_headline::draw),
        "title" => drawn(component, props, cx, text::title::draw),
        "status-cycle" => drawn(component, props, cx, text::status_cycle::draw),
        "text-highlight" => drawn(component, props, cx, text::text_highlight::draw),
        "punch-lines" => drawn(component, props, cx, text::punch_lines::draw),
        "announce-title" => drawn(component, props, cx, text::announce_title::draw),
        "callout" => drawn(component, props, cx, overlay::callout::draw),
        "animated-bar-chart" => drawn(component, props, cx, data::animated_bar_chart::draw),
        "rolling-number" => drawn(component, props, cx, data::rolling_number::draw),
        "animated-line-chart" => drawn(component, props, cx, data::animated_line_chart::draw),
        "funnel-chart" => drawn(component, props, cx, data::funnel_chart::draw),
        "comparison-bars" => drawn(component, props, cx, data::comparison_bars::draw),
        "donut-chart" => drawn(component, props, cx, data::donut_chart::draw),
        "gauge-dial" => drawn(component, props, cx, data::gauge_dial::draw),
        "stat-card" => drawn(component, props, cx, data::stat_card::draw),
        "metric-ticker" => drawn(component, props, cx, data::metric_ticker::draw),
        "bar-chart-race" => drawn(component, props, cx, data::bar_chart_race::draw),
        "lower-third" => drawn(component, props, cx, overlay::lower_third::draw),
        "ink-underline" => drawn(component, props, cx, overlay::ink_underline::draw),
        "scribble-circle" => drawn(component, props, cx, overlay::scribble_circle::draw),
        "ink-arrow" => drawn(component, props, cx, overlay::ink_arrow::draw),
        "confetti" => drawn(component, props, cx, overlay::confetti::draw),
        "radial-burst" => drawn(component, props, cx, overlay::radial_burst::draw),
        "lottie" => drawn(component, props, cx, overlay::lottie::draw),
        "keystroke" => drawn(component, props, cx, overlay::keystroke::draw),
        "captions" => drawn(component, props, cx, overlay::captions::draw),
        "notification-stack" => {
            drawn(component, props, cx, overlay::notification_stack::draw)
        }
        "gradient-scale-cut-text" => {
            drawn(component, props, cx, text::gradient_scale_cut_text::draw)
        }
        _ => Ok(None),
    }
}
