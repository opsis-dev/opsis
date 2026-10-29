use opsis_macros::settings;
use opsis_types::cards::{CardRow, Icon, Tone};
use opsis_types::looks::NodeLook;
use opsis_types::numbers;
use opsis_types::recording::{BlurArea, PictureLook, RecordingProps};
use opsis_types::timeline::TimelineItem;
use opsis_types::values::ClipValue;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::clip::{IMAGE, VIDEO, map_items};
use crate::errors::SettingsError;
use crate::node::{
    CardCtx, Category, Ctx, InputTake, NodeDef, RECORDING_PORT, RowExt, missing, row, with_clip,
};
use crate::params::Params;
use crate::text::{format_duration, percent_text, plural};

const MAX_BLUR_OF_WIDTH: f64 = 0.04;
const MAX_SOFTNESS_OF_WIDTH: f64 = 0.02;

#[settings]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct Area {
    #[setting(range(min = 0, max = 1))]
    pub x: f64,
    #[setting(range(min = 0, max = 1))]
    pub y: f64,
    #[setting(range(min = 0, max = 1))]
    pub width: f64,
    #[setting(range(min = 0, max = 1))]
    pub height: f64,
}

pub fn default_areas() -> Vec<Area> {
    vec![Area {
        x: 0.62,
        y: 0.06,
        width: 0.3,
        height: 0.08,
    }]
}

#[settings]
#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct BlurAreas {
    #[setting(
        "Areas to blur",
        default = default_areas(),
        length(max = 8),
        widget = "boxes",
        time_key = "frameMs",
        help = "Drag on the picture to add an area. Move the slider to draw on a later moment of the take. The blur stays on that part of the window while the camera zooms and pans."
    )]
    pub areas: Vec<Area>,
    #[setting("Frame to draw on", range(min = 0, max = 3600000), unit = "ms", hidden)]
    pub frame_ms: f64,
    #[setting("Strength", default = 0.6, range(min = 0.05, max = 1), step = 0.05)]
    pub strength: f64,
    #[setting(
        "Soft edge",
        default = 0.3,
        range(min = 0, max = 1),
        step = 0.05,
        advanced
    )]
    pub softness: f64,
    #[setting(
        "Rounded corners",
        default = 0.3,
        range(min = 0, max = 1),
        step = 0.05,
        advanced
    )]
    pub roundness: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "From",
        range(min = 0, max = 3600000),
        unit = "ms",
        zero_label = "Start of the take",
        advanced
    )]
    pub from_ms: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Until",
        range(min = 0, max = 3600000),
        unit = "ms",
        zero_label = "End of the take",
        advanced
    )]
    pub until_ms: Option<u32>,
    #[setting(
        "Fade in and out",
        default = 200,
        range(min = 0, max = 2000),
        step = 1,
        unit = "ms",
        advanced
    )]
    pub fade_ms: u32,
}

impl Params for BlurAreas {
    fn check(&self) -> Result<(), SettingsError> {
        match (self.from_ms, self.until_ms) {
            (Some(from), Some(until)) if until <= from => {
                Err(SettingsError::rule("Until must come after From"))
            }
            _ => Ok(()),
        }
    }
}

pub fn blur_areas_of(p: &BlurAreas) -> Vec<BlurArea> {
    p.areas
        .iter()
        .map(|a| BlurArea {
            x: a.x,
            y: a.y,
            width: a.width,
            height: a.height,
            strength: p.strength * MAX_BLUR_OF_WIDTH,
            softness: p.softness * MAX_SOFTNESS_OF_WIDTH,
            roundness: p.roundness,
            fade_ms: f64::from(p.fade_ms),
            from_ms: p.from_ms.map(f64::from),
            until_ms: p.until_ms.map(f64::from),
        })
        .collect()
}

fn times_text(from_ms: Option<u32>, until_ms: Option<u32>) -> Option<String> {
    match (from_ms, until_ms) {
        (None, None) => None,
        (from, None) => Some(format!(
            "From {}",
            format_duration(f64::from(from.unwrap_or(0)))
        )),
        (None, Some(until)) => Some(format!("Until {}", format_duration(f64::from(until)))),
        (Some(from), Some(until)) => Some(format!(
            "{} → {}",
            format_duration(f64::from(from)),
            format_duration(f64::from(until))
        )),
    }
}

fn after_take_row(take: Option<InputTake>, from_ms: Option<u32>) -> Option<CardRow> {
    let (take, from) = (take?, from_ms?);
    (f64::from(from) >= take.duration_ms).then(|| {
        row(
            Icon::TriangleAlert,
            format!(
                "After the take ends at {}",
                format_duration(take.duration_ms)
            ),
        )
        .tone(Tone::Warn)
    })
}

fn blur_areas() -> NodeDef {
    NodeDef::recording_modifier::<BlurAreas>("effect.blurAreas", "Blur areas", Category::Effects)
        .describe("Blurs parts of the browser window to hide private details. The blur stays on the page as the camera moves.")
        .card(|ctx: &CardCtx<'_, BlurAreas>| {
            let p = &ctx.params;
            let take = ctx.inputs.takes.get(RECORDING_PORT).copied().flatten();
            vec![
                Some(if p.areas.is_empty() {
                    missing(Icon::EyeOff, "Draw an area to blur")
                } else {
                    row(Icon::EyeOff, format!("{} · {} strength", plural(numbers::float(p.areas.len()), "area"), percent_text(p.strength, f64::INFINITY)))
                }),
                times_text(p.from_ms, p.until_ms).map(|t| row(Icon::Clock, t)),
                after_take_row(take, p.from_ms),
            ]
        })
        .restyle(|props: RecordingProps, p: &BlurAreas, _| {
            let mut areas = props.blur_areas.clone().unwrap_or_default();
            areas.extend(blur_areas_of(p));
            RecordingProps { blur_areas: Some(areas), ..props }
        })
}

#[settings]
#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct Look {
    #[setting(
        "Brightness",
        range(min = -1, max = 1),
        step = 0.05,
        presets_label = "Look",
        presets = [
            { "label": "Natural", "values": { "brightness": 0, "contrast": 1, "saturation": 1, "warmth": 0, "vignette": 0, "grain": 0 } },
            { "label": "Warm", "values": { "brightness": 0, "contrast": 1, "saturation": 1.1, "warmth": 0.35, "vignette": 0.2, "grain": 0 } },
            { "label": "Cool", "values": { "brightness": 0, "contrast": 1.05, "saturation": 1, "warmth": -0.3, "vignette": 0, "grain": 0 } },
            { "label": "Punchy", "values": { "brightness": 0, "contrast": 1.2, "saturation": 1.25, "warmth": 0, "vignette": 0, "grain": 0 } },
            { "label": "Soft", "values": { "brightness": 0.1, "contrast": 0.88, "saturation": 0.9, "warmth": 0, "vignette": 0, "grain": 0 } },
            { "label": "Black and white", "values": { "brightness": 0, "contrast": 1.15, "saturation": 0, "warmth": 0, "vignette": 0, "grain": 0 } },
            { "label": "Old film", "values": { "brightness": 0, "contrast": 1, "saturation": 0.75, "warmth": 0.25, "vignette": 0.45, "grain": 0.35 } }
        ]
    )]
    pub brightness: f64,
    #[setting(
        "Contrast",
        default = 1.0,
        range(min = 0.5, max = 1.5),
        step = 0.05,
        unit = "×"
    )]
    pub contrast: f64,
    #[setting(
        "Colour",
        default = 1.0,
        range(min = 0, max = 2),
        step = 0.05,
        unit = "×"
    )]
    pub saturation: f64,
    #[setting("Warmth", range(min = -1, max = 1), step = 0.05)]
    pub warmth: f64,
    #[setting("Dark edges", range(min = 0, max = 1), step = 0.05)]
    pub vignette: f64,
    #[setting("Film grain", range(min = 0, max = 1), step = 0.05)]
    pub grain: f64,
    #[setting("Dark edges size", default = 0.65, range(min = 0, max = 1), advanced)]
    pub vignette_size: f64,
    #[setting(
        "Dark edges softness",
        default = 0.35,
        range(min = 0, max = 1),
        advanced
    )]
    pub vignette_softness: f64,
    #[setting("Dark edges colour", default = "#000000", widget = "color", check = colour, advanced)]
    pub vignette_color: String,
    #[setting("Tint", default = 0.0, range(min = -1, max = 1), advanced)]
    pub tint: f64,
}

impl Params for Look {}

impl Look {
    fn values(&self) -> [f64; 6] {
        [
            self.brightness,
            self.contrast,
            self.saturation,
            self.warmth,
            self.vignette,
            self.grain,
        ]
    }
}

const NEUTRAL: [f64; 6] = [0.0, 1.0, 1.0, 0.0, 0.0, 0.0];

const LOOK_PRESETS: [(&str, [f64; 6]); 7] = [
    ("Natural", NEUTRAL),
    ("Warm", [0.0, 1.0, 1.1, 0.35, 0.2, 0.0]),
    ("Cool", [0.0, 1.05, 1.0, -0.3, 0.0, 0.0]),
    ("Punchy", [0.0, 1.2, 1.25, 0.0, 0.0, 0.0]),
    ("Soft", [0.1, 0.88, 0.9, 0.0, 0.0, 0.0]),
    ("Black and white", [0.0, 1.15, 0.0, 0.0, 0.0, 0.0]),
    ("Old film", [0.0, 1.0, 0.75, 0.25, 0.45, 0.35]),
];

fn changed_look(p: &Look) -> PictureLook {
    let keep = |i: usize, v: f64| (v != NEUTRAL[i]).then_some(v);
    let v = p.values();
    let plain = Look::default();
    let changed = |v: f64, default: f64| (v != default).then_some(v);
    PictureLook {
        vignette_size: changed(p.vignette_size, plain.vignette_size),
        vignette_softness: changed(p.vignette_softness, plain.vignette_softness),
        vignette_color: (p.vignette_color != plain.vignette_color)
            .then(|| p.vignette_color.clone()),
        tint: changed(p.tint, plain.tint),
        brightness: keep(0, v[0]),
        contrast: keep(1, v[1]),
        saturation: keep(2, v[2]),
        warmth: keep(3, v[3]),
        vignette: keep(4, v[4]),
        grain: keep(5, v[5]),
    }
}

const LOOK_COMPONENTS: [&str; 2] = [VIDEO, IMAGE];

fn with_look(clip: &ClipValue, look: &PictureLook) -> ClipValue {
    let shown = serde_json::to_value(look).expect("look serializes");
    ClipValue {
        tracks: map_items(&clip.tracks, &|item| match item {
            TimelineItem::Recording { base, props } => TimelineItem::Recording {
                base,
                props: Box::new(RecordingProps {
                    look: Some(look.clone()),
                    ..*props
                }),
            },
            TimelineItem::Element {
                base,
                component,
                mut props,
            } if LOOK_COMPONENTS.contains(&component.as_str()) => {
                props.insert("look".into(), shown.clone());
                TimelineItem::Element {
                    base,
                    component,
                    props,
                }
            }
            other => other,
        }),
        ..clip.clone()
    }
}

fn picture_look() -> NodeDef {
    NodeDef::clip_modifier::<Look>("effect.look", "Picture look", Category::Effects)
        .describe(
            "Gives recordings, videos and images a look: colour, brightness, a dark vignette and film grain. To grade a recording's whole picture, window and backdrop included, use Apply grade on an exported file.",
        )
        .card(|ctx: &CardCtx<'_, Look>| {
            let values = ctx.params.values();
            let preset = LOOK_PRESETS.iter().find(|(_, v)| *v == values).map(|(label, _)| *label);
            let changed = values.iter().zip(NEUTRAL).filter(|(v, n)| **v != *n).count();
            vec![Some(match preset {
                Some(label) => row(Icon::Sparkles, label).muted_if(label == "Natural"),
                None => row(Icon::Sparkles, plural(numbers::float(changed), "change")),
            })]
        })
        .compiles(|ctx: &Ctx<'_, Look>| with_clip(ctx, |clip| with_look(clip, &changed_look(&ctx.params))))
}

pub fn nodes() -> Vec<NodeDef> {
    vec![
        blur_areas().look(NodeLook::of("mynaui:incognito")),
        picture_look().look(NodeLook::of("lucide:contrast")),
    ]
}
