use opsis_macros::{choice, settings};
use opsis_types::cards::{CardRow, Icon};
use opsis_types::recording::{FrameTheme, RecordingDevice, WindowStyle};
use opsis_types::timeline::TimelineSpec;
use opsis_types::values::{CompileValue, ImageValue};
use opsis_types::workflow::{PortType, WorkflowSettings};
use serde::{Deserialize, Serialize};

use crate::asset_ref::base_name;
use crate::backdrops::Backdrop;
use crate::clip::{FRAMED_IMAGE, ItemExtra, clip_to_spec, item_clip};
use crate::color::is_color;
use crate::errors::SettingsError;
use crate::node::{
    CardCtx, Category, Ctx, NodeDef, Phase, Platform, RowExt, missing, outputs, port, row,
};
use crate::nodes::image::{FrameShape, IMAGE_PORT, PlaceAlign};
use crate::params::{Params, Values, to_values};
use crate::recording::{DEFAULT_FRAME_THEME, DEFAULT_WINDOW_STYLE, host_window_style};
use crate::text::humanize;

pub const FRAME_IMAGE_TYPE: &str = "image.frame";
pub const FRAMED_AT_FRAME: u32 = 60;
const FRAMED_FPS: u32 = 30;
const FRAMED_FRAMES: u32 = 90;
const TILT_ROOM: f64 = 0.06;

#[choice]
pub enum ImageFrameKind {
    #[default]
    #[label("Plain")]
    Plain,
    #[label("Window")]
    Window,
    #[label("Device")]
    Device,
}

#[choice]
pub enum ImageAngle {
    #[default]
    #[label("Flat")]
    Flat,
    #[label("Tilted")]
    Tilted,
}

#[choice]
pub enum ImageBackground {
    #[label("None")]
    None,
    #[default]
    #[label("Studio image")]
    Image,
    #[label("Colour")]
    Solid,
    #[label("Gradient")]
    Gradient,
    #[label("Your image")]
    Asset,
}

#[settings(check)]
pub struct FrameImage {
    #[setting("Kind")]
    pub kind: ImageFrameKind,
    #[setting("Device", default = RecordingDevice::Laptop, show_if = { "kind": ["device"] })]
    pub device: RecordingDevice,
    #[setting(
        "Window style",
        default = DEFAULT_WINDOW_STYLE,
        show_if = { "kind": ["window"] }
    )]
    pub window_style: WindowStyle,
    #[setting("Frame colour", default = DEFAULT_FRAME_THEME, show_if = { "kind": ["window"] })]
    pub frame_theme: FrameTheme,
    #[setting("Angle", show_if = { "kind": ["plain", "window"] })]
    pub angle: ImageAngle,
    #[setting(
        "Turn",
        default = 18.0,
        range(min = -60, max = 60),
        step = 1,
        unit = "°",
        help = "Turns the picture away on its right side, or its left with a negative angle.",
        show_if = { "kind": ["plain", "window"], "angle": ["tilted"] }
    )]
    pub turn: f64,
    #[setting(
        "Tilt",
        default = 8.0,
        range(min = -60, max = 60),
        step = 1,
        unit = "°",
        help = "Leans the top back, or the bottom with a negative angle.",
        show_if = { "kind": ["plain", "window"], "angle": ["tilted"] }
    )]
    pub tilt: f64,
    #[setting(
        "Lean",
        range(min = -45, max = 45),
        step = 1,
        unit = "°",
        show_if = { "kind": ["plain", "window"], "angle": ["tilted"] },
        advanced
    )]
    pub lean: f64,
    #[setting(
        "Viewing distance",
        default = 2.0,
        range(min = 0.5, max = 10),
        step = 0.1,
        help = "How far away the picture is seen from, in its long side. Closer looks more three-dimensional.",
        show_if = { "kind": ["plain", "window"], "angle": ["tilted"] },
        advanced
    )]
    pub perspective: f64,
    #[setting("Background")]
    pub background: ImageBackground,
    #[setting("Studio image", default = Backdrop::Glaze, show_if = { "background": ["image"] })]
    pub backdrop: Backdrop,
    #[setting("Colour", default = "#0b0b10", widget = "color", show_if = { "background": ["solid"] })]
    pub color: String,
    #[setting(
        "Gradient",
        default = "linear-gradient(135deg, #1e1b4b 0%, #0f172a 100%)",
        widget = "gradient",
        show_if = { "background": ["gradient"] }
    )]
    pub gradient: String,
    #[setting("Image", widget = "asset", accept = "image/*", show_if = { "background": ["asset"] })]
    pub asset: String,
    #[setting(
        "Padding",
        default = 8.0,
        range(min = 0, max = 50),
        step = 1,
        unit = "%",
        help = "Space around the picture, as a share of its shorter side."
    )]
    pub padding: f64,
    #[setting(
        "Shape",
        help = "Grows the background around the picture to this shape; the picture is never cut."
    )]
    pub shape: FrameShape,
    #[setting(
        "Place on the shape",
        labels = { "start": "Top or left", "center": "Centre", "end": "Bottom or right" },
        help = "Where the picture sits when the shape grows the background.",
        advanced
    )]
    pub align: PlaceAlign,
    #[setting(
        "Corner radius",
        default = 16.0,
        range(min = 0, max = 200),
        step = 1,
        unit = "px",
        show_if = { "kind": ["plain"] }
    )]
    pub radius: f64,
    #[setting(
        "Shadow drop",
        default = 16.0,
        range(min = 0, max = 200),
        step = 1,
        unit = "px",
        show_if = { "kind": ["plain", "window"] },
        advanced
    )]
    pub shadow_drop: f64,
    #[setting(
        "Shadow softness",
        default = 40.0,
        range(min = 0, max = 200),
        step = 1,
        unit = "px",
        show_if = { "kind": ["plain", "window"] },
        advanced
    )]
    pub shadow_softness: f64,
    #[setting(
        "Shadow opacity",
        default = 0.35,
        range(min = 0, max = 1),
        step = 0.05,
        show_if = { "kind": ["plain", "window"] },
        advanced
    )]
    pub shadow_opacity: f64,
    #[setting(
        "Shadow colour",
        default = "#000000",
        widget = "color",
        check = colour,
        show_if = { "kind": ["plain", "window"] },
        advanced
    )]
    pub shadow_color: String,
}

impl Params for FrameImage {
    fn check(&self) -> Result<(), SettingsError> {
        match self.background {
            ImageBackground::Solid if !is_color(&self.color) => {
                Err(SettingsError::rule("Pick a colour"))
            }
            ImageBackground::Gradient if self.gradient.trim().is_empty() => {
                Err(SettingsError::rule("Pick a gradient"))
            }
            ImageBackground::Asset if self.asset.trim().is_empty() => {
                Err(SettingsError::rule("Choose an image"))
            }
            _ => Ok(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct ImageTilt {
    pub turn: f64,
    pub tilt: f64,
    pub lean: f64,
    pub perspective: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct FramedImageProps {
    pub src: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window: Option<WindowStyle>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme: Option<FrameTheme>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device: Option<RecordingDevice>,
    pub radius: f64,
    pub shadow_drop: f64,
    pub shadow_softness: f64,
    pub shadow_opacity: f64,
    pub shadow_color: String,
    pub background: String,
    pub background_image: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tilt: Option<ImageTilt>,
}

impl FrameImage {
    fn tilted(&self) -> Option<ImageTilt> {
        (self.kind != ImageFrameKind::Device && self.angle == ImageAngle::Tilted).then_some(
            ImageTilt {
                turn: self.turn,
                tilt: self.tilt,
                lean: self.lean,
                perspective: self.perspective,
            },
        )
    }

    fn background_css(&self) -> (String, String) {
        match self.background {
            ImageBackground::None => (String::new(), String::new()),
            ImageBackground::Image => (
                String::new(),
                format!("app:backdrops/{}.jpg", self.backdrop.id()),
            ),
            ImageBackground::Solid => (self.color.clone(), String::new()),
            ImageBackground::Gradient => (self.gradient.clone(), String::new()),
            ImageBackground::Asset => (String::new(), crate::asset_ref::project_ref(&self.asset)),
        }
    }

    fn background_row(&self) -> CardRow {
        match self.background {
            ImageBackground::None => row(Icon::Image, "No background"),
            ImageBackground::Image => row(Icon::Image, humanize(self.backdrop.id())),
            ImageBackground::Solid => row(Icon::Palette, self.color.clone()),
            ImageBackground::Gradient => row(Icon::Blend, "Gradient").titled(self.gradient.clone()),
            ImageBackground::Asset if self.asset.is_empty() => {
                missing(Icon::Image, "Choose an image")
            }
            ImageBackground::Asset => row(Icon::Image, base_name(&self.asset)),
        }
    }

    fn kind_row(&self) -> CardRow {
        let angle = if self.tilted().is_some() {
            " · tilted"
        } else {
            ""
        };
        match self.kind {
            ImageFrameKind::Plain => row(Icon::Image, format!("Plain{angle}")),
            ImageFrameKind::Window => row(
                Icon::AppWindow,
                format!("{} window{angle}", self.window_style.label()),
            ),
            ImageFrameKind::Device => row(
                match self.device {
                    RecordingDevice::Laptop => Icon::Laptop,
                    RecordingDevice::Phone => Icon::Smartphone,
                },
                self.device.label(),
            ),
        }
    }
}

#[expect(
    clippy::many_single_char_names,
    reason = "w, h, x, y, z are the picture's size and a corner's position in 3D"
)]
fn tilted_box(size: (f64, f64), tilt: &ImageTilt) -> (f64, f64) {
    let (w, h) = size;
    let distance = tilt.perspective * w.max(h);
    let (lean, lift, turn) = (
        tilt.lean.to_radians(),
        tilt.tilt.to_radians(),
        tilt.turn.to_radians(),
    );
    let corners = [(0.0, 0.0), (w, 0.0), (w, h), (0.0, h)].map(|(x, y)| {
        let (x, y) = (x - w / 2.0, y - h / 2.0);
        let (x, y) = (
            f64::mul_add(y, -lean.sin(), x * lean.cos()),
            f64::mul_add(y, lean.cos(), x * lean.sin()),
        );
        let (y, z) = (y * lift.cos(), -y * lift.sin());
        let (x, z) = (
            f64::mul_add(z, -turn.sin(), x * turn.cos()),
            f64::mul_add(z, turn.cos(), x * turn.sin()),
        );
        let scale = distance / (distance + z);
        (x * scale, y * scale)
    });
    let spread = |pick: fn(&(f64, f64)) -> f64| {
        let low = corners.iter().map(pick).fold(f64::INFINITY, f64::min);
        let high = corners.iter().map(pick).fold(f64::NEG_INFINITY, f64::max);
        high - low
    };
    let room = TILT_ROOM * w.max(h) * 2.0;
    (spread(|c| c.0) + room, spread(|c| c.1) + room)
}

pub fn framed_placement(source: (f64, f64), p: &FrameImage) -> ((f64, f64), (f64, f64)) {
    let held = p.tilted().map_or(source, |t| tilted_box(source, &t));
    let pad = (held.0.min(held.1) * p.padding.clamp(0.0, 50.0) / 100.0).round();
    let (mut w, mut h) = (2.0f64.mul_add(pad, held.0), 2.0f64.mul_add(pad, held.1));
    if let Some(ratio) = p.shape.ratio() {
        if w / h > ratio {
            h = (w / ratio).round();
        } else {
            w = (h * ratio).round();
        }
    }
    let free = (
        2.0f64.mul_add(-pad, w - held.0),
        2.0f64.mul_add(-pad, h - held.1),
    );
    let at = (
        free.0.mul_add(p.align.share(), pad) + (held.0 - source.0) / 2.0,
        free.1.mul_add(p.align.share(), pad) + (held.1 - source.1) / 2.0,
    );
    ((w.round(), h.round()), (at.0.round(), at.1.round()))
}

pub fn framed_image_props(p: &FrameImage, src: &str, source: (f64, f64)) -> FramedImageProps {
    let (_, (x, y)) = framed_placement(source, p);
    let (background, background_image) = p.background_css();
    FramedImageProps {
        src: src.to_owned(),
        x,
        y,
        width: source.0,
        height: source.1,
        window: (p.kind == ImageFrameKind::Window).then_some(p.window_style),
        theme: (p.kind == ImageFrameKind::Window).then_some(p.frame_theme),
        device: (p.kind == ImageFrameKind::Device).then_some(p.device),
        radius: if p.kind == ImageFrameKind::Plain {
            p.radius
        } else {
            0.0
        },
        shadow_drop: p.shadow_drop,
        shadow_softness: p.shadow_softness,
        shadow_opacity: p.shadow_opacity,
        shadow_color: p.shadow_color.clone(),
        background,
        background_image,
        tilt: p.tilted(),
    }
}

pub fn framed_image_spec(p: &FrameImage, src: &str, source: (f64, f64)) -> TimelineSpec {
    let ((width, height), _) = framed_placement(source, p);
    let props: Values = to_values(&framed_image_props(p, src, source));
    let clip = item_clip(
        FRAME_IMAGE_TYPE,
        FRAMED_IMAGE,
        props,
        FRAMED_FRAMES,
        ItemExtra::default(),
    );
    let settings = WorkflowSettings {
        width: opsis_types::numbers::whole(width).max(2),
        height: opsis_types::numbers::whole(height).max(2),
        fps: FRAMED_FPS,
        ..WorkflowSettings::default()
    };
    clip_to_spec(&clip, &settings)
}

fn result_size(ctx: &CardCtx<'_, FrameImage>) -> Option<(f64, f64)> {
    let image = ctx.result?.output(IMAGE_PORT)?.as_image()?;
    Some((
        f64::from(image.width.filter(|w| *w != 0)?),
        f64::from(image.height.filter(|h| *h != 0)?),
    ))
}

fn card(ctx: &CardCtx<'_, FrameImage>) -> Vec<Option<CardRow>> {
    let p = &ctx.params;
    vec![
        Some(p.kind_row()),
        Some(p.background_row()),
        result_size(ctx).map(|(w, h)| row(Icon::Maximize, format!("{w} × {h} px"))),
    ]
}

pub fn frame_image() -> NodeDef {
    NodeDef::of::<FrameImage>(FRAME_IMAGE_TYPE, "Frame image", Category::Image, Phase::Render)
        .describe("Frames an image for a post: plain with rounded corners and a soft shadow, in a window with its title bar, or on a laptop or phone, flat or tilted, on a padded background.")
        .executes()
        .inputs(vec![(IMAGE_PORT, port(PortType::Media, "Image"))])
        .outputs(vec![(IMAGE_PORT, port(PortType::Image, "Image"))])
        .card(card)
        .compiles(|ctx: &Ctx<'_, FrameImage>| {
            let image = ctx
                .result
                .and_then(|r| r.output(IMAGE_PORT))
                .and_then(CompileValue::as_image)
                .cloned()
                .unwrap_or(ImageValue {
                    src: None,
                    width: None,
                    height: None,
                    scale: None,
                    body: None,
                });
            outputs([(IMAGE_PORT, CompileValue::Image(image))])
        })
        .host_defaults(|platform: Platform| {
            let mut values = Values::new();
            values.insert(
                "windowStyle".into(),
                serde_json::to_value(host_window_style(platform)).expect("style serializes"),
            );
            values
        })
}
