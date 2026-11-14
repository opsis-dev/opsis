use std::collections::HashMap;
use std::io::Cursor;
use std::sync::{LazyLock, Mutex};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use opsis_scene::clock::js_round;
use opsis_scene::node::{Node, SvgElement, img};
use opsis_scene::paint::BlendMode;
use opsis_scene::style::ObjectFit;
use opsis_scene::timeline::absolute_fill;
use opsis_types::recording::TerminalEffects;

use crate::elements::shader::{effect_over, generated_in, quads, value};
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
const LENS_PROGRAM: &str = "terminal-lens";
const GRAIN_PROGRAM: &str = "terminal-grain";
const EFFECT_SCALE: f64 = 2.0;

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
    reason = "each step rounds on its own, and the recorded goldens hold what that draws"
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

pub fn bulge_map(x: f64, y: f64) -> String {
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
    reason = "each step rounds on its own, and the recorded goldens hold what that draws"
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
    reason = "each step rounds on its own, and the recorded goldens hold what that draws"
)]
fn fringe_shift(effects: &TerminalEffects, size: CellBox) -> (f64, f64) {
    if effects.fringe <= 0.0 {
        return (0.0, 0.0);
    }
    let shift = 1.0f64.max(js_round(
        effects.fringe * SHIFT_PER_FRINGE * (size.width / DESIGN_WIDTH),
    ));
    let angle = effects.fringe_angle * std::f64::consts::PI / 180.0;
    (js_round(shift * angle.cos()), js_round(shift * angle.sin()))
}

pub fn lensed(effects: &TerminalEffects, size: CellBox, content: Node) -> Node {
    let (dx, dy) = fringe_shift(effects, size);
    let map = img(
        bulge_map(effects.bulge_x, effects.bulge_y),
        absolute_fill().object_fit(ObjectFit::Fill),
    );
    effect_over(
        LENS_PROGRAM,
        vec![
            value("shift", vec![dx, dy]),
            value(
                "bulge",
                vec![effects.bulge * BULGE_SHARE * size.width.max(size.height)],
            ),
        ],
        EFFECT_SCALE,
        vec![content, map.into()],
    )
}

const BLOCK: usize = 256;
const RAND_MAXIMUM: i32 = i32::MAX;
const HALF_MAX_16_BITS: f32 = 32767.5;

struct Seeded(i32);

impl Seeded {
    fn new(seed: f64) -> Self {
        let mut value = opsis_types::numbers::whole_signed(seed.trunc());
        if value <= 0 {
            value = -(value % (RAND_MAXIMUM - 1)) + 1;
        }
        Self(value.min(RAND_MAXIMUM - 1))
    }

    const fn next(&mut self) -> i32 {
        const AMPLITUDE: i32 = 16807;
        const Q: i32 = 127_773;
        const R: i32 = 2836;
        let mut result = AMPLITUDE * (self.0 % Q) - R * (self.0 / Q);
        if result <= 0 {
            result += RAND_MAXIMUM;
        }
        self.0 = result;
        result
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PerlinTables {
    pub lattice: Vec<f64>,
    pub gradients: Vec<f64>,
}

pub fn perlin_tables(seed: f64) -> PerlinTables {
    let mut random = Seeded::new(seed);
    let mut lattice: Vec<usize> = (0..BLOCK).collect();
    let mut noise = [[[0i32; 2]; BLOCK]; 4];
    for channel in &mut noise {
        for entry in channel.iter_mut() {
            entry[0] = random.next() % 512;
            entry[1] = random.next() % 512;
        }
    }
    for i in (1..BLOCK).rev() {
        let k = lattice[i];
        let j = usize::try_from(random.next() % 256).unwrap_or(0);
        lattice[i] = lattice[j];
        lattice[j] = k;
    }
    let shuffled: Vec<[[i32; 2]; BLOCK]> = noise
        .iter()
        .map(|channel| {
            let mut out = [[0i32; 2]; BLOCK];
            for (i, entry) in out.iter_mut().enumerate() {
                *entry = channel[lattice[i]];
            }
            out
        })
        .collect();
    let mut gradients = Vec::with_capacity(4 * BLOCK * 2);
    for channel in &shuffled {
        for entry in channel {
            let x = (opsis_types::numbers::narrow(f64::from(entry[0])) - 256.0) / 256.0;
            let y = (opsis_types::numbers::narrow(f64::from(entry[1])) - 256.0) / 256.0;
            let length = x.hypot(y);
            let (x, y) = if length > 0.0 {
                (x / length, y / length)
            } else {
                (x, y)
            };
            gradients.push(f64::from(((x + 1.0) * HALF_MAX_16_BITS).round()));
            gradients.push(f64::from(((y + 1.0) * HALF_MAX_16_BITS).round()));
        }
    }
    PerlinTables {
        lattice: lattice
            .iter()
            .map(|v| opsis_types::numbers::float(*v))
            .collect(),
        gradients,
    }
}

pub fn grain(effects: &TerminalEffects, frame: f64) -> Node {
    let tables = perlin_tables(frame);
    let style = absolute_fill()
        .opacity(effects.grain * GRAIN_OPACITY)
        .mix_blend_mode(BlendMode::Overlay);
    generated_in(
        GRAIN_PROGRAM,
        vec![
            value("frequency", vec![GRAIN_FREQUENCY, GRAIN_FREQUENCY]),
            value("colour", vec![if effects.colour_grain { 1.0 } else { 0.0 }]),
            quads("lattice", tables.lattice),
            quads("gradients", tables.gradients),
        ],
        EFFECT_SCALE,
        style,
    )
}

pub fn effect_defs(id: &str, effects: &TerminalEffects, size: CellBox) -> SvgElement {
    let black = hex_colour(0);
    let follows = effects.vignette_follows_frame;
    let radius = if follows {
        0.75
    } else {
        size.width.hypot(size.height) * 0.5
    };
    let defs = SvgElement::new("defs");
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
    defs.child(gradient)
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
    out
}
