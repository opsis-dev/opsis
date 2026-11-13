use std::collections::HashMap;
use std::io::Cursor;
use std::sync::{LazyLock, Mutex};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use opsis_scene::clock::js_round;
use opsis_scene::node::SvgElement;
use opsis_types::recording::TerminalEffects;

use crate::elements::terminal::view::{CellBox, hex_colour};

const DESIGN_WIDTH: f64 = 1920.0;
const SHIFT_PER_FRINGE: f64 = 8.0;
const BLUR_PER_SOFTEN: f64 = 8.0;
const VIGNETTE_DEPTH: f64 = 0.9;
const GRAIN_OPACITY: f64 = 0.35;
const GRAIN_FREQUENCY: f64 = 0.85;
const BULGE_SHARE: f64 = 0.18;
const MAP_SIZE: u32 = 128;
const MAP_MIDDLE: f64 = 128.0;
const MAP_SPAN: f64 = 127.0;
const TRAIL_FRAMES: i32 = 3;
const TRAIL_DECAY_FROM: f64 = 0.5;
const TRAIL_DECAY_SPAN: f64 = 0.48;
const RED_ONLY: &str = "1 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 1 0";
const GREEN_ONLY: &str = "0 0 0 0 0 0 1 0 0 0 0 0 0 0 0 0 0 0 1 0";
const BLUE_ONLY: &str = "0 0 0 0 0 0 0 0 0 0 0 0 1 0 0 0 0 0 1 0";

static MAPS: LazyLock<Mutex<HashMap<String, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn clamped_byte(value: f64) -> u8 {
    u8::try_from(opsis_types::numbers::whole(
        value.round_ties_even().clamp(0.0, 255.0),
    ))
    .unwrap_or(u8::MAX)
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film page's JavaScript rounds every step, and the pictures have to match it"
)]
fn bulge_pixels(x: f64, y: f64) -> Vec<u8> {
    let size = f64::from(MAP_SIZE);
    let mut data = Vec::new();
    for j in 0..MAP_SIZE {
        for i in 0..MAP_SIZE {
            let dx = (f64::from(i) + 0.5) / size - x;
            let dy = (f64::from(j) + 0.5) / size - y;
            let pull = 0.0f64.max(1.0 - (dx * dx + dy * dy) * 2.0);
            data.extend([
                clamped_byte(MAP_MIDDLE - dx * pull * MAP_SPAN),
                clamped_byte(MAP_MIDDLE - dy * pull * MAP_SPAN),
                clamped_byte(MAP_MIDDLE),
                u8::MAX,
            ]);
        }
    }
    data
}

fn bulge_map(x: f64, y: f64) -> String {
    let key = format!("{x}|{y}");
    if let Some(known) = MAPS.lock().ok().and_then(|maps| maps.get(&key).cloned()) {
        return known;
    }
    let Some(image) = image::RgbaImage::from_raw(MAP_SIZE, MAP_SIZE, bulge_pixels(x, y)) else {
        return String::new();
    };
    let mut bytes = Cursor::new(Vec::new());
    if image.write_to(&mut bytes, image::ImageFormat::Png).is_err() {
        return String::new();
    }
    let url = format!(
        "data:image/png;base64,{}",
        STANDARD.encode(bytes.into_inner())
    );
    if let Ok(mut maps) = MAPS.lock() {
        maps.insert(key, url.clone());
    }
    url
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film page's JavaScript rounds every step, and the pictures have to match it"
)]
pub fn trail_weights(effects: Option<&TerminalEffects>) -> Vec<f64> {
    let Some(effects) = effects.filter(|e| e.trails > 0.0) else {
        return Vec::new();
    };
    let decay = effects.trails * TRAIL_DECAY_SPAN + TRAIL_DECAY_FROM;
    (1..=TRAIL_FRAMES)
        .map(|k| decay.powf(f64::from(k)))
        .collect()
}

pub fn soften_px(effects: Option<&TerminalEffects>, size: CellBox) -> f64 {
    effects.map_or(0.0, |e| {
        e.soften * BLUR_PER_SOFTEN * (size.width / DESIGN_WIDTH)
    })
}

pub fn has_lens(effects: Option<&TerminalEffects>) -> bool {
    effects.is_some_and(|e| e.fringe > 0.0 || e.bulge != 0.0)
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film page's JavaScript rounds every step, and the pictures have to match it"
)]
fn lens_filter(id: &str, effects: &TerminalEffects, size: CellBox) -> SvgElement {
    let shift = 1.0f64.max(js_round(
        effects.fringe * SHIFT_PER_FRINGE * (size.width / DESIGN_WIDTH),
    ));
    let angle = effects.fringe_angle * std::f64::consts::PI / 180.0;
    let fringe = effects.fringe > 0.0;
    let dx = if fringe {
        js_round(shift * angle.cos())
    } else {
        0.0
    };
    let dy = if fringe {
        js_round(shift * angle.sin())
    } else {
        0.0
    };
    let mut filter = SvgElement::new("filter")
        .attr("id", id)
        .attr("x", 0.0)
        .attr("y", 0.0)
        .attr("width", size.width)
        .attr("height", size.height)
        .attr("filterUnits", "userSpaceOnUse")
        .attr("color-interpolation-filters", "sRGB");
    if effects.bulge == 0.0 {
        filter = filter.child(
            SvgElement::new("feOffset")
                .attr("in", "SourceGraphic")
                .attr("dx", 0.0)
                .attr("dy", 0.0)
                .attr("result", "bent"),
        );
    } else {
        filter = filter
            .child(
                SvgElement::new("feImage")
                    .attr("href", bulge_map(effects.bulge_x, effects.bulge_y))
                    .attr("x", 0.0)
                    .attr("y", 0.0)
                    .attr("width", size.width)
                    .attr("height", size.height)
                    .attr("preserveAspectRatio", "none")
                    .attr("result", "map"),
            )
            .child(
                SvgElement::new("feDisplacementMap")
                    .attr("in", "SourceGraphic")
                    .attr("in2", "map")
                    .attr(
                        "scale",
                        effects.bulge * BULGE_SHARE * size.width.max(size.height),
                    )
                    .attr("xChannelSelector", "R")
                    .attr("yChannelSelector", "G")
                    .attr("result", "bent"),
            );
    }
    let channel = |values: &str, result: &str| {
        SvgElement::new("feColorMatrix")
            .attr("in", "bent")
            .attr("values", values)
            .attr("result", result)
    };
    let offset = |input: &str, x: f64, y: f64, result: &str| {
        SvgElement::new("feOffset")
            .attr("in", input)
            .attr("dx", x)
            .attr("dy", y)
            .attr("result", result)
    };
    filter
        .child(channel(RED_ONLY, "red"))
        .child(offset("red", dx, dy, "redShifted"))
        .child(channel(GREEN_ONLY, "green"))
        .child(channel(BLUE_ONLY, "blue"))
        .child(offset("blue", -dx, -dy, "blueShifted"))
        .child(
            SvgElement::new("feBlend")
                .attr("in", "redShifted")
                .attr("in2", "green")
                .attr("mode", "screen")
                .attr("result", "redGreen"),
        )
        .child(
            SvgElement::new("feBlend")
                .attr("in", "redGreen")
                .attr("in2", "blueShifted")
                .attr("mode", "screen"),
        )
}

pub fn effect_defs(id: &str, effects: &TerminalEffects, size: CellBox, frame: f64) -> SvgElement {
    let black = hex_colour(0);
    let follows = effects.vignette_follows_frame;
    let radius = if follows {
        0.75
    } else {
        size.width.hypot(size.height) * 0.5
    };
    let mut defs = SvgElement::new("defs");
    if has_lens(Some(effects)) {
        defs = defs.child(lens_filter(&format!("{id}-lens"), effects, size));
    }
    let gradient = SvgElement::new("radialGradient")
        .attr("id", format!("{id}-vignette"))
        .attr(
            "cx",
            if follows {
                effects.vignette_x
            } else {
                effects.vignette_x * size.width
            },
        )
        .attr(
            "cy",
            if follows {
                effects.vignette_y
            } else {
                effects.vignette_y * size.height
            },
        )
        .attr("r", radius)
        .attr(
            "gradientUnits",
            if follows {
                "objectBoundingBox"
            } else {
                "userSpaceOnUse"
            },
        )
        .child(
            SvgElement::new("stop")
                .attr("offset", "0.45")
                .attr("stop-color", black.as_str())
                .attr("stop-opacity", 0.0),
        )
        .child(
            SvgElement::new("stop")
                .attr("offset", "1")
                .attr("stop-color", black.as_str())
                .attr("stop-opacity", effects.vignette * VIGNETTE_DEPTH),
        );
    let mut grain = SvgElement::new("filter")
        .attr("id", format!("{id}-grain"))
        .attr("x", 0.0)
        .attr("y", 0.0)
        .attr("width", "100%")
        .attr("height", "100%")
        .child(
            SvgElement::new("feTurbulence")
                .attr("type", "fractalNoise")
                .attr("baseFrequency", GRAIN_FREQUENCY)
                .attr("numOctaves", 1.0)
                .attr("seed", frame)
                .attr("stitchTiles", "stitch"),
        );
    if !effects.colour_grain {
        grain = grain.child(
            SvgElement::new("feColorMatrix")
                .attr("type", "saturate")
                .attr("values", "0"),
        );
    }
    defs.child(gradient).child(grain)
}

fn full_rect(size: CellBox) -> SvgElement {
    SvgElement::new("rect")
        .attr("x", 0.0)
        .attr("y", 0.0)
        .attr("width", size.width)
        .attr("height", size.height)
}

pub fn effect_layers(id: &str, effects: &TerminalEffects, size: CellBox) -> Vec<SvgElement> {
    let mut out = Vec::new();
    if effects.vignette > 0.0 {
        out.push(full_rect(size).attr("fill", format!("url(#{id}-vignette)")));
    }
    if effects.grain > 0.0 {
        out.push(
            full_rect(size)
                .attr("filter", format!("url(#{id}-grain)"))
                .attr("opacity", effects.grain * GRAIN_OPACITY)
                .attr("style", "mix-blend-mode: overlay"),
        );
    }
    out
}
