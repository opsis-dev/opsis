use indexmap::IndexMap;
use opsis_macros::{choice, settings};
use opsis_types::actions::{Action, ActionKind, ActionLog, Point, Rect};
use opsis_types::cards::{CardRow, Icon, Tone};
use opsis_types::looks::{NodeLook, NodeMotion};
use opsis_types::numbers;
use opsis_types::numbers::exact_eq;
use opsis_types::recording::{
    AddressMode, CameraEasing, CameraProps, CameraStyle, CameraTuning, FootageChain, FrameTheme,
    RecordingProps, ScreenProps, ShotProps, WindowProps, WindowStyle,
};
use opsis_types::timeline::{ElementRef, GroupProps};
use opsis_types::values::{
    ClipValue, CompileValue, RecordingValue, StyleEntry, TaggedRecording, TakeKind, TakeSource,
};
use opsis_types::workflow::{DEFAULT_TAKE_MS, PortType};
use serde_json::{Map, Value};

use crate::asset_ref::base_name;
use crate::backdrops::Backdrop;
use crate::card_rows::toggle_row;
use crate::clip::{
    ItemExtra, clip_frame_of_recording, patch_recordings, placeholder_clip, recording_clip,
    wrap_clip,
};
use crate::color::is_color;
use crate::elements::device::laptop_frame::LaptopFinale;
use crate::elements::device::phone_frame::{PhoneEntrance, PhoneVariant};
use crate::elements::element_meta;
use crate::elements::theme::{Theme, ThemeMode, warm_themes};
use crate::errors::SettingsError;
use crate::node::{
    CLIP_PORT, CardCtx, Category, Ctx, InputTake, NodeDef, Phase, Platform, RECORDING_PORT, RowExt,
    STYLE_PORT, missing, opt, outputs, port, row, with_clip,
};
use crate::nodes::elements::element_props;
use crate::nodes::terminal::with_look;
use crate::params::{Params, Values};
use crate::recording::{
    DEFAULT_ADDRESS_BAR, DEFAULT_FRAME_THEME, DEFAULT_TAKE_SPEED, DEFAULT_WINDOW_STYLE,
    default_recording_props, follow_cursor_beats, host_window_style, idle_spans, point_rect,
    recording_speed,
};
use crate::registry::node_def;
use crate::remap::{RemapSegment, eased_segments, remap_duration, remap_from_segments};
use crate::terminal::default_look;

const NO_PAN_LIMIT_PX: f64 = 1.0e9;
use crate::style::{restyle_with, retime_with};
use crate::text::{display_address, format_duration, humanize, percent_text, plural};
use crate::time::{EPSILON, ms_to_frames, recording_ms_to_frames, whole_frames};

pub const FOLLOW_CURSOR_TYPE: &str = "camera.followCursor";
pub const CLIP_FROM_TAKE_TYPE: &str = "camera.take";
pub const CAMERA_TYPE: &str = "camera.camera";
pub const SHOT_TYPE: &str = "camera.shot";
pub const WINDOW_FRAME_TYPE: &str = "camera.windowFrame";
pub const SPEED_RAMP_TYPE: &str = "camera.speedRamp";
const CURSOR_STYLE_TYPE: &str = crate::nodes::cursor::CURSOR_STYLE_TYPE;

#[choice]
pub enum FrameChoice {
    #[default]
    #[label("None")]
    None,
    #[label("macOS")]
    Macos,
    #[label("Windows")]
    Windows,
    #[label("Linux")]
    Linux,
}

impl FrameChoice {
    pub(crate) const fn style(self) -> Option<WindowStyle> {
        match self {
            Self::None => None,
            Self::Macos => Some(WindowStyle::Macos),
            Self::Windows => Some(WindowStyle::Windows),
            Self::Linux => Some(WindowStyle::Linux),
        }
    }
}

#[settings]
pub struct ClipFromTake {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Playback speed",
        range(min = 0.25, max = 4),
        step = 0.05,
        unit = "×",
        zero_label = "The take's usual",
        help = "Browser takes usually play at 1.25×, terminal takes at 1×."
    )]
    pub speed: Option<f64>,
}

pub const fn usual_speed(kind: TakeKind) -> f64 {
    match kind {
        TakeKind::Browser => DEFAULT_TAKE_SPEED,
        TakeKind::Terminal => 1.0,
    }
}

impl ClipFromTake {
    pub fn speed_for(&self, kind: TakeKind) -> f64 {
        self.speed.unwrap_or_else(|| usual_speed(kind))
    }
}

#[settings]
pub struct Camera {
    #[setting("Zoom to the actions", default = true)]
    pub zoom: bool,
    #[setting(
        "Maximum zoom",
        default = 1.74,
        range(min = 1.1, max = 2.5),
        step = 0.02,
        unit = "×",
        help = "Higher zooms closer but can look soft.",
        show_if = { "zoom": ["true"] }
    )]
    pub max_scale: f64,
    #[setting(
        "Hold each zoom at least",
        default = 1300,
        range(min = 0, max = 10000),
        step = 50,
        unit = "ms",
        show_if = { "zoom": ["true"] }
    )]
    pub hold: u32,
    #[setting(
        "Least zoom",
        default = 1.18,
        range(min = 1, max = 2),
        step = 0.02,
        unit = "×",
        show_if = { "zoom": ["true"] },
        advanced
    )]
    pub min_scale: f64,
    #[setting(
        "First zoom at most",
        default = 1.54,
        range(min = 1, max = 2.5),
        step = 0.02,
        unit = "×",
        help = "The first shot stays wider so the page is seen first.",
        show_if = { "zoom": ["true"] },
        advanced
    )]
    pub first_max_scale: f64,
    #[setting(
        "Room around the target",
        default = 80,
        range(min = 0, max = 400),
        step = 5,
        unit = "px",
        show_if = { "zoom": ["true"] },
        advanced
    )]
    pub room: u32,
    #[setting(
        "Target size",
        default = 55,
        range(min = 10, max = 100),
        step = 1,
        unit = "%",
        help = "How much of the frame the target and its room fill.",
        show_if = { "zoom": ["true"] },
        advanced
    )]
    pub target_share: u32,
    #[setting(
        "Off centre",
        default = 0.26,
        range(min = 0, max = 0.5),
        step = 0.01,
        help = "How far a target sits from the middle, leaving room on the side it faces.",
        show_if = { "zoom": ["true"] },
        advanced
    )]
    pub off_centre: f64,
    #[setting(
        "Join actions within",
        default = 2200,
        range(min = 0, max = 20000),
        step = 100,
        unit = "ms",
        help = "Actions this close share one zoom.",
        show_if = { "zoom": ["true"] },
        advanced
    )]
    pub join_within_ms: u32,
    #[setting(
        "Stay zoomed within",
        default = 2800,
        range(min = 0, max = 20000),
        step = 100,
        unit = "ms",
        help = "Shots this close stay zoomed in between.",
        show_if = { "zoom": ["true"] },
        advanced
    )]
    pub stay_zoomed_within_ms: u32,
    #[setting(
        "Hold after typing",
        default = 400,
        range(min = 0, max = 5000),
        step = 50,
        unit = "ms",
        show_if = { "zoom": ["true"] },
        advanced
    )]
    pub hold_after_typing_ms: u32,
    #[setting(
        "Rest at the end",
        default = 600,
        range(min = 0, max = 10000),
        step = 50,
        unit = "ms",
        help = "Time zoomed out before the take ends.",
        show_if = { "zoom": ["true"] },
        advanced
    )]
    pub rest_at_end_ms: u32,
    #[setting(
        "Longest pan",
        default = 500,
        range(min = 50, max = 5000),
        step = 10,
        unit = "px",
        help = "Farther moves zoom out and back in instead.",
        advanced
    )]
    pub longest_pan: u32,
    #[setting("Easing", advanced)]
    pub camera_easing: CameraEasing,
    #[setting(
        "Move time",
        default = 1.0,
        range(min = 0.25, max = 4),
        step = 0.05,
        unit = "×",
        help = "Longer than 1 makes every camera move slower.",
        advanced
    )]
    pub camera_pace: f64,
}

impl Camera {
    fn tuning(&self) -> CameraTuning {
        let seconds = |ms: u32| Some(f64::from(ms) / 1000.0);
        CameraTuning {
            s_max: Some(self.max_scale),
            hold_min_s: seconds(self.hold),
            min_scale: Some(self.min_scale),
            first_max_scale: Some(self.first_max_scale),
            room_px: Some(f64::from(self.room)),
            target_share: Some(f64::from(self.target_share) / 100.0),
            off_centre: Some(self.off_centre),
            join_within_s: seconds(self.join_within_ms),
            stay_zoomed_within_s: seconds(self.stay_zoomed_within_ms),
            hold_after_typing_s: seconds(self.hold_after_typing_ms),
            rest_at_end_s: seconds(self.rest_at_end_ms),
            longest_pan_px: Some(f64::from(self.longest_pan)),
            pace: Some(self.camera_pace),
            easing: Some(self.camera_easing),
        }
    }
}

#[settings]
pub struct Shot {
    #[setting("Camera style", default = CameraStyle::Classic)]
    pub style: CameraStyle,
    #[setting("Ground colour", default = "#faf6f1", widget = "color", check = colour, show_if = { "style": ["stage"] })]
    pub ground: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Window size",
        range(min = 40, max = 100),
        unit = "%",
        zero_label = "The style's own",
        help = "How much of the frame the window fills.",
        show_if = { "style": ["classic", "stage"] },
        "type" = "integer",
        advanced
    )]
    pub window_size: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Window corners",
        range(min = 0, max = 60),
        unit = "px",
        zero_label = "The frame's own",
        show_if = { "style": ["classic", "stage"] },
        "type" = "integer",
        advanced
    )]
    pub corner_radius: Option<u32>,
    #[setting(
        "Window shadow",
        default = 1.0,
        range(min = 0, max = 1),
        step = 0.05,
        show_if = { "style": ["classic", "stage"] },
        advanced
    )]
    pub shadow: f64,
    #[setting(
        "Lift the colours",
        default = true,
        help = "A touch more saturation, contrast and brightness on the screen.",
        advanced
    )]
    pub lift_colours: bool,
    #[setting(
        "Background blur",
        default = 48,
        range(min = 0, max = 200),
        step = 1,
        unit = "px",
        help = "Behind a full-page style in another shape.",
        show_if = { "style": ["proof", "ledger"] },
        advanced
    )]
    pub bleed_blur: u32,
    #[setting(
        "Background brightness",
        default = 0.55,
        range(min = 0, max = 1.5),
        step = 0.05,
        show_if = { "style": ["proof", "ledger"] },
        advanced
    )]
    pub bleed_dim: f64,
}

impl Shot {
    fn apply(&self, shot: ShotProps) -> ShotProps {
        ShotProps {
            style: self.style,
            window_fit: self.window_size.map(|percent| f64::from(percent) / 100.0),
            ground: Some(self.ground.clone()),
            radius: self.corner_radius.map(f64::from),
            rim_glow: Some(self.shadow),
            lift_colours: Some(self.lift_colours),
            bleed_blur: Some(f64::from(self.bleed_blur)),
            bleed_dim: Some(self.bleed_dim),
            ..shot
        }
    }
}

#[settings]
pub struct ManualZoom {
    #[setting(
        "At",
        default = 1000,
        range(min = 0, max = 600_000),
        step = 50,
        unit = "ms"
    )]
    pub at_ms: u32,
    #[setting(
        "Hold",
        default = 1500,
        range(min = 100, max = 600_000),
        step = 50,
        unit = "ms"
    )]
    pub hold_ms: u32,
    #[setting(
        "Spot",
        default = 0.5,
        range(min = 0, max = 1),
        step = 0.01,
        widget = "spot",
        time_key = "atMs",
        help = "Click the frame to choose where to zoom."
    )]
    pub x: f64,
    #[setting(
        "Spot vertical",
        default = 0.5,
        range(min = 0, max = 1),
        step = 0.01,
        hidden
    )]
    pub y: f64,
    #[setting(
        "Zoom",
        default = 1.5,
        range(min = 1.1, max = 2.5),
        step = 0.05,
        unit = "×"
    )]
    pub scale: f64,
    #[setting(
        "Start moving before",
        default = 400,
        range(min = 0, max = 5000),
        step = 50,
        unit = "ms",
        help = "How long before the moment the camera starts to move.",
        advanced
    )]
    pub lead_ms: u32,
}

#[settings]
pub struct Pan {
    #[setting(
        "Start at",
        default = 1000,
        range(min = 0, max = 600_000),
        step = 50,
        unit = "ms"
    )]
    pub at_ms: u32,
    #[setting(
        "Pan for",
        default = 1500,
        range(min = 100, max = 600_000),
        step = 50,
        unit = "ms"
    )]
    pub duration_ms: u32,
    #[setting(
        "From",
        default = 0.3,
        range(min = 0, max = 1),
        step = 0.01,
        widget = "spot",
        time_key = "atMs",
        help = "Click the frame to choose where to start."
    )]
    pub from_x: f64,
    #[setting(
        "From vertical",
        default = 0.4,
        range(min = 0, max = 1),
        step = 0.01,
        hidden
    )]
    pub from_y: f64,
    #[setting(
        "To",
        default = 0.7,
        range(min = 0, max = 1),
        step = 0.01,
        widget = "spot",
        time_key = "atMs",
        help = "Click the frame to choose where to end."
    )]
    pub to_x: f64,
    #[setting(
        "To vertical",
        default = 0.6,
        range(min = 0, max = 1),
        step = 0.01,
        hidden
    )]
    pub to_y: f64,
    #[setting(
        "Zoom",
        default = 1.4,
        range(min = 1.1, max = 2.5),
        step = 0.05,
        unit = "×"
    )]
    pub scale: f64,
    #[setting(
        "Zoom out for far moves",
        default = true,
        help = "When the spots are far apart, the camera zooms out and back in instead of sliding.",
        advanced
    )]
    pub zoom_out_when_far: bool,
}

#[settings]
pub struct FollowCursor {
    #[setting(
        "Zoom",
        default = 1.35,
        range(min = 1.1, max = 2.5),
        step = 0.05,
        unit = "×"
    )]
    pub scale: f64,
    #[setting(
        "Re-aim every",
        default = 700,
        range(min = 200, max = 5000),
        step = 50,
        unit = "ms",
        advanced
    )]
    pub every_ms: u32,
    #[setting(
        "Zoom out on big jumps",
        default = true,
        help = "When the cursor jumps far, the camera zooms out and back in instead of sliding.",
        advanced
    )]
    pub zoom_out_on_jumps: bool,
}

#[settings]
pub struct IdleSpeedup {
    #[setting(
        "Quiet for longer than",
        default = 1500,
        range(min = 0, max = 20000),
        step = 50,
        unit = "ms"
    )]
    pub threshold_ms: u32,
    #[setting(
        "Keep around each action",
        default = 400,
        range(min = 0, max = 5000),
        step = 50,
        unit = "ms",
        advanced
    )]
    pub pad_ms: u32,
    #[setting(
        "Speed while quiet",
        default = 4.0,
        range(min = 1.5, max = 64),
        step = 0.5,
        unit = "×"
    )]
    pub rate: f64,
    #[setting(
        "Ease into the speed",
        range(min = 0, max = 2000),
        step = 50,
        unit = "ms",
        help = "Changes speed gradually over this long instead of at once.",
        advanced
    )]
    pub ease_ms: u32,
}

#[settings]
pub struct SpeedSegment {
    #[setting("From", step = 50, unit = "ms")]
    pub from_ms: u32,
    #[setting("To", default = 1000, step = 50, unit = "ms")]
    pub to_ms: u32,
    #[setting(
        "Speed",
        default = 2.0,
        range(min = 0.1, max = 64),
        step = 0.1,
        unit = "×"
    )]
    pub rate: f64,
}

#[settings]
pub struct SpeedRamp {
    #[setting("Parts", default = vec![SpeedSegment::default()])]
    pub segments: Vec<SpeedSegment>,
    #[setting(
        "Ease into the speed",
        range(min = 0, max = 2000),
        step = 50,
        unit = "ms",
        help = "Changes speed gradually over this long instead of at once.",
        advanced
    )]
    pub ease_ms: u32,
}

#[settings(check)]
pub struct Crop {
    #[setting("Left", default = 0.1, range(min = 0, max = 1), step = 0.01)]
    pub x: f64,
    #[setting("Top", default = 0.1, range(min = 0, max = 1), step = 0.01)]
    pub y: f64,
    #[setting("Width", default = 0.8, range(min = 0.05, max = 1), step = 0.01)]
    pub w: f64,
    #[setting("Height", default = 0.8, range(min = 0.05, max = 1), step = 0.01)]
    pub h: f64,
    #[setting(
        "Fill the frame",
        help = "Crops further so the area fills the frame instead of showing bars.",
        advanced
    )]
    pub fill: bool,
}

impl Params for Crop {
    fn check(&self) -> Result<(), SettingsError> {
        if self.x + self.w > 1.0 + EPSILON {
            return Err(SettingsError::rule("The area runs past the right edge"));
        }
        if self.y + self.h > 1.0 + EPSILON {
            return Err(SettingsError::rule("The area runs past the bottom edge"));
        }
        Ok(())
    }
}

#[choice]
pub enum Device {
    #[label("Laptop")]
    Laptop,
    #[label("Phone")]
    Phone,
}

impl Device {
    const fn icon(self) -> Icon {
        match self {
            Self::Laptop => Icon::Laptop,
            Self::Phone => Icon::Smartphone,
        }
    }

    const fn component(self) -> &'static str {
        match self {
            Self::Laptop => "laptop-frame",
            Self::Phone => "phone-frame",
        }
    }
}

#[choice]
pub enum DeviceEntrance {
    #[label("Slide in")]
    Rise,
    #[label("Open up")]
    Open,
    #[label("None")]
    None,
}

#[settings]
pub struct DeviceFrame {
    #[setting(
        "Device",
        default = Device::Laptop
    )]
    pub device: Device,
    #[setting(
        "Entrance",
        default = DeviceEntrance::Open,
        show_if = { "device": ["laptop"] }
    )]
    pub entrance: DeviceEntrance,
    #[setting(
        "Entrance",
        default = PhoneEntrance::Rise,
        show_if = { "device": ["phone"] }
    )]
    pub phone_entrance: PhoneEntrance,
    #[setting(
        "Angle",
        default = PhoneVariant::Flat,
        show_if = { "device": ["phone"] }
    )]
    pub variant: PhoneVariant,
    #[setting(
        "Theme",
        default = Theme::of(ThemeMode::Dark),
        transform = warm_themes,
        widget = "theme"
    )]
    pub theme: Theme,
    #[setting("Body colour", widget = "color", zero_label = "Theme", check = colour_or_empty, advanced)]
    pub bezel_color: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Size",
        range(min = 0.3, max = 1.5),
        step = 0.05,
        zero_label = "The device's usual"
    )]
    pub scale: Option<f64>,
    #[setting("Notch", default = true, show_if = { "device": ["laptop"] }, advanced)]
    pub show_notch: bool,
    #[setting("Notch label", default = "Recording started", show_if = { "device": ["laptop"], "showNotch": ["true"] }, advanced)]
    pub notch_label: String,
    #[setting("Battery level", default = 85, range(min = 0, max = 100), unit = "%", show_if = { "device": ["laptop"], "showNotch": ["true"] }, advanced)]
    pub battery_level: u32,
    #[setting("Recording light", default = "#30D158", widget = "color", check = colour, show_if = { "device": ["laptop"], "showNotch": ["true"] }, advanced)]
    pub indicator_color: String,
    #[setting("Float", default = true, advanced)]
    pub float_loop: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Float amplitude", range(min = 0, max = 30), step = 1, zero_label = "The device's usual", show_if = { "floatLoop": ["true"] }, "type" = "integer", advanced)]
    pub float_amplitude: Option<u32>,
    #[setting("Lid tilt", default = 18, range(min = 0, max = 45), unit = "°", show_if = { "device": ["laptop"] }, advanced)]
    pub rest_tilt: u32,
    #[setting(
        "Finale",
        default = LaptopFinale::None,
        show_if = { "device": ["laptop"] },
        advanced
    )]
    pub finale: LaptopFinale,
    #[setting("Shadow", zero_label = "The device's usual", advanced)]
    pub shadow: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Corner radius", range(min = 0, max = 90), unit = "px", zero_label = "The device's usual", "type" = "integer", advanced)]
    pub radius: Option<u32>,
    #[setting("Turn", default = -12, range(min = -30, max = 30), unit = "°", show_if = { "device": ["phone"], "variant": ["tilt"] }, advanced)]
    pub tilt_angle: i32,
    #[setting("Dynamic island", default = true, show_if = { "device": ["phone"] }, advanced)]
    pub show_dynamic_island: bool,
}

impl DeviceFrame {
    fn element_values(&self) -> Values {
        let laptop = self.device == Device::Laptop;
        let mut values = Values::new();
        let mut set = |key: &str, value: Value| {
            values.insert(key.to_owned(), value);
        };
        set(
            "theme",
            serde_json::to_value(&self.theme).unwrap_or(Value::Null),
        );
        set("bezelColor", Value::from(self.bezel_color.clone()));
        set(
            "scale",
            Value::from(self.scale.unwrap_or(if laptop { 0.95 } else { 1.0 })),
        );
        set("floatLoop", Value::from(self.float_loop));
        set(
            "floatAmplitude",
            Value::from(self.float_amplitude.unwrap_or(if laptop { 4 } else { 6 })),
        );
        if !self.shadow.trim().is_empty() {
            set("shadow", Value::from(self.shadow.clone()));
        }
        if let Some(radius) = self.radius {
            set("radius", Value::from(radius));
        } else if laptop {
            set("radius", Value::from(22));
        }
        if laptop {
            set(
                "entrance",
                serde_json::to_value(self.entrance).unwrap_or(Value::Null),
            );
            set("showNotch", Value::from(self.show_notch));
            set("notchLabel", Value::from(self.notch_label.clone()));
            set("batteryLevel", Value::from(self.battery_level));
            set("indicatorColor", Value::from(self.indicator_color.clone()));
            set("restTilt", Value::from(self.rest_tilt));
            set(
                "finale",
                serde_json::to_value(self.finale).unwrap_or(Value::Null),
            );
        } else {
            set(
                "entrance",
                serde_json::to_value(self.phone_entrance).unwrap_or(Value::Null),
            );
            set(
                "variant",
                serde_json::to_value(self.variant).unwrap_or(Value::Null),
            );
            set("tiltAngle", Value::from(self.tilt_angle));
            set("showDynamicIsland", Value::from(self.show_dynamic_island));
        }
        values
    }
}

#[choice]
pub enum BackdropKind {
    #[label("Studio image")]
    Image,
    #[label("Colour")]
    Solid,
    #[label("Gradient")]
    Gradient,
    #[label("Your image")]
    Asset,
}

#[settings(check, eq)]
pub struct BackdropStyle {
    #[setting(
        "Kind",
        default = BackdropKind::Image
    )]
    pub kind: BackdropKind,
    #[setting("Studio image", default = Backdrop::Glaze, show_if = { "kind": ["image"] })]
    pub name: Backdrop,
    #[setting("Colour", default = "#0b0b10", widget = "color", show_if = { "kind": ["solid"] })]
    pub color: String,
    #[setting(
        "Gradient",
        default = "linear-gradient(135deg, #1e1b4b 0%, #0f172a 100%)",
        widget = "gradient",
        show_if = { "kind": ["gradient"] }
    )]
    pub gradient: String,
    #[setting("Image", widget = "asset", accept = "image/*", show_if = { "kind": ["asset"] })]
    pub asset: String,
}

impl Params for BackdropStyle {
    fn check(&self) -> Result<(), SettingsError> {
        match self.kind {
            BackdropKind::Solid if !is_color(&self.color) => {
                Err(SettingsError::rule("Pick a colour"))
            }
            BackdropKind::Gradient if self.gradient.trim().is_empty() => {
                Err(SettingsError::rule("Pick a gradient"))
            }
            BackdropKind::Asset if self.asset.trim().is_empty() => {
                Err(SettingsError::rule("Choose an image"))
            }
            _ => Ok(()),
        }
    }
}

impl BackdropStyle {
    pub fn card_row(&self) -> CardRow {
        match self.kind {
            BackdropKind::Image => row(Icon::Image, humanize(self.name.id())),
            BackdropKind::Solid => row(Icon::Palette, self.color.clone()),
            BackdropKind::Gradient => row(Icon::Blend, "Gradient").titled(self.gradient.clone()),
            BackdropKind::Asset if self.asset.is_empty() => missing(Icon::Image, "Choose an image"),
            BackdropKind::Asset => row(Icon::Image, base_name(&self.asset)),
        }
    }

    fn chosen(&self) -> Option<String> {
        match self.kind {
            BackdropKind::Image => Some(self.name.id().to_owned()),
            BackdropKind::Solid => Some(self.color.clone()),
            BackdropKind::Gradient => Some(self.gradient.clone()),
            BackdropKind::Asset => (!self.asset.is_empty()).then(|| self.asset.clone()),
        }
    }
}

#[settings(eq)]
pub struct MotionBlur {
    #[setting("Motion blur", default = true)]
    pub enabled: bool,
    #[setting(
        "Amount",
        default = 180,
        range(min = 0, max = 360),
        step = 10,
        unit = "°",
        help = "How long each frame's shutter stays open, as an angle.",
        show_if = { "enabled": ["true"] }
    )]
    pub shutter_angle: u32,
}

#[settings(eq)]
pub struct WindowFrame {
    #[setting(
        "Window style",
        default = DEFAULT_WINDOW_STYLE
    )]
    pub window_style: WindowStyle,
    #[setting("Frame colour", default = DEFAULT_FRAME_THEME)]
    pub frame_theme: FrameTheme,
    #[setting(
        "Title bar",
        default = DEFAULT_ADDRESS_BAR,
        help = "Shows the page's address or the program's title in the title bar."
    )]
    pub title_bar: bool,
    #[setting(
        "Title bar text",
        zero_label = "The take's own",
        help = "Shown instead of the page's address or the program's title, such as app.example.com.",
        show_if = { "titleBar": ["true"] }
    )]
    pub title: String,
    #[setting(
        "Show",
        default = AddressMode::Fixed,
        show_if = { "titleBar": ["true"], "title": ["*"] },
        show_if_take = ["browser"],
        advanced
    )]
    pub address_mode: AddressMode,
    #[setting(
        "Title bar font",
        widget = "font",
        zero_label = "The computer's",
        advanced
    )]
    pub font: String,
}

impl WindowFrame {
    fn bar(&self) -> Bar<'_> {
        Bar {
            shown: self.title_bar,
            text: &self.title,
            mode: self.address_mode,
        }
    }
}

#[settings(eq)]
pub struct ApplyStyle {}

struct Bar<'a> {
    shown: bool,
    text: &'a str,
    mode: AddressMode,
}

impl Bar<'_> {
    fn typed(&self) -> Option<&str> {
        Some(self.text.trim()).filter(|a| self.shown && !a.is_empty())
    }

    fn row(&self) -> CardRow {
        if !self.shown {
            return row(Icon::Link, "No title bar text").muted_if(true);
        }
        match self.typed() {
            None => row(Icon::Link, "The take's own title"),
            Some(address) if self.mode == AddressMode::Host => row(
                Icon::Link,
                format!("{} · keeps the path", display_address(address, false)),
            ),
            Some(address) => row(Icon::Link, display_address(address, false)),
        }
    }
}

fn with_window_frame(props: RecordingProps, p: &WindowFrame) -> RecordingProps {
    let bar = p.bar();
    let text = bar.typed().map(str::to_owned);
    RecordingProps {
        window: WindowProps {
            chrome: true,
            style: Some(p.window_style),
            theme: Some(p.frame_theme),
            font: crate::fonts::font_prop(&p.font),
            bar: Some(bar.shown),
            mode: text.as_ref().map(|_| bar.mode),
            text,
        },
        ..props
    }
}

const fn frame_theme_name(theme: FrameTheme) -> &'static str {
    match theme {
        FrameTheme::Light => "light",
        FrameTheme::Dark => "dark",
    }
}

fn take_row(take: Option<InputTake>, at_ms: u32) -> Option<CardRow> {
    let take = take?;
    (f64::from(at_ms) >= take.duration_ms).then(|| {
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

struct FocusSpot {
    id: &'static str,
    t: f64,
    x: f64,
    y: f64,
    t_depart: Option<f64>,
}

fn focus_beats(
    mut props: RecordingProps,
    node_id: &str,
    cluster: &str,
    scale: f64,
    spots: &[FocusSpot],
) -> RecordingProps {
    let vp = props.log.viewport;
    for spot in spots {
        let point = Point {
            x: spot.x * f64::from(vp.width),
            y: spot.y * f64::from(vp.height),
        };
        let mut beat = Action::new(format!("{node_id}:{}", spot.id), ActionKind::Focus, spot.t);
        beat.t_depart = spot.t_depart;
        beat.point = Some(point);
        beat.rect = Some(point_rect(point, 1.0));
        beat.cluster = Some(format!("{node_id}:{cluster}"));
        beat.zoom_scale = Some(scale);
        beat.node_id = Some(node_id.to_owned());
        props.extra_beats.push(beat);
    }
    props
}

fn remap_clip(
    clip: &ClipValue,
    node_id: &str,
    fps: u32,
    segments: &[(f64, f64, f64)],
) -> ClipValue {
    let to_frames = |recording_ms: f64| {
        clip_frame_of_recording(&clip.tracks, &|rec| {
            recording_ms_to_frames(recording_ms, recording_speed(rec), fps)
        })
        .unwrap_or_else(|| recording_ms_to_frames(recording_ms, 1.0, fps))
    };
    let parts: Vec<RemapSegment> = segments
        .iter()
        .map(|(from, to, rate)| RemapSegment {
            from: to_frames(*from),
            to: to_frames(*to),
            rate: *rate,
        })
        .collect();
    let remap = remap_from_segments(f64::from(clip.duration_in_frames), &parts);
    let duration = whole_frames(remap_duration(&remap), 1);
    wrap_clip(
        clip,
        node_id,
        GroupProps {
            remap: Some(remap),
            ..GroupProps::default()
        },
        Some(duration),
    )
}

fn entries_of(value: Option<&CompileValue>) -> Vec<StyleEntry> {
    value
        .and_then(CompileValue::as_style)
        .map_or_default(|s| s.entries.clone())
}

fn base_props(rec: &RecordingValue, log: ActionLog, speed: f64) -> Option<RecordingProps> {
    let mut props = match &rec.source {
        TakeSource::Browser(browser) => {
            default_recording_props(browser.video.clone()?, log, rec.node_id.clone())
        }
        TakeSource::Terminal(terminal) => {
            let look = default_look(terminal.theme.as_deref().cloned());
            let mut props = default_recording_props(String::new(), log, rec.node_id.clone());
            props.pointer.shown = props.log.cursor_track.is_some();
            props.screen = ScreenProps::Terminal {
                cast: terminal.cast.clone()?,
                cols: terminal.cols,
                rows: terminal.rows,
                look: Box::new(look.clone()),
                chain: FootageChain::default(),
                frames_url: None,
            };
            with_look(props, |_| look)
        }
    };
    props.speed = speed;
    props.camera = CameraProps {
        zoom: Camera::default().zoom,
        tuning: Camera::default().tuning(),
    };
    props.shot = Shot::default().apply(props.shot);
    let blur = MotionBlur::default();
    props.shot.motion_blur = blur.enabled;
    props.shot.shutter_angle = Some(f64::from(blur.shutter_angle));
    Some(props)
}

pub fn clip_from_take_props(
    rec: &RecordingValue,
    speed: f64,
    entries: &[StyleEntry],
) -> Result<Option<RecordingProps>, SettingsError> {
    let Some(props) = rec.log.clone().and_then(|log| base_props(rec, log, speed)) else {
        return Ok(None);
    };
    let mut all = entries.to_vec();
    all.extend(rec.edits.iter().cloned());
    restyle_with(props, &all).map(Some)
}

fn style_warnings(ctx: &CardCtx<'_, ClipFromTake>) -> [Option<CardRow>; 2] {
    let no_track = ctx
        .inputs
        .takes
        .get(RECORDING_PORT)
        .is_some_and(|t| t.is_some_and(|t| t.kind == TakeKind::Browser && !t.cursor_track));
    let styled = |kind: &str| {
        ctx.inputs
            .style
            .get(STYLE_PORT)
            .is_some_and(|s| s.iter().any(|t| t == kind))
    };
    [
        (no_track && styled(FOLLOW_CURSOR_TYPE)).then(|| {
            row(Icon::TriangleAlert, "The take has no cursor to follow")
                .tone(Tone::Warn)
                .titled("Set Cursor to Smooth on the Record take node.")
        }),
        (no_track && styled(CURSOR_STYLE_TYPE)).then(|| {
            row(Icon::TriangleAlert, "The take films its own cursor").tone(Tone::Warn).titled(
                "The Cursor node's look, size and colours apply only to a drawn cursor. Set Cursor to Smooth on the Record take node.",
            )
        }),
    ]
}

const fn take_kind_label(kind: TakeKind) -> &'static str {
    match kind {
        TakeKind::Browser => "Browser take",
        TakeKind::Terminal => "Terminal take",
    }
}

fn unused_styles(ctx: &CardCtx<'_, ClipFromTake>, kind: TakeKind) -> Vec<Option<CardRow>> {
    let mut seen: Vec<&str> = Vec::new();
    ctx.inputs
        .style
        .get(STYLE_PORT)
        .into_iter()
        .flatten()
        .filter_map(|t| {
            let def = node_def(t)?;
            let only = def.only_for?;
            (only != kind && !seen.contains(&t.as_str())).then(|| {
                seen.push(t);
                Some(
                    row(
                        Icon::Info,
                        format!(
                            "{} does nothing on a {}",
                            def.title,
                            take_kind_label(kind).to_lowercase()
                        ),
                    )
                    .tone(Tone::Muted),
                )
            })
        })
        .collect()
}

fn clip_from_take() -> NodeDef {
    NodeDef::of::<ClipFromTake>(CLIP_FROM_TAKE_TYPE, "Clip from take", Category::Camera, Phase::Compile)
        .describe("Turns a take into a clip, shot the way its style says: the window, the backdrop and the camera moves.")
        .inputs(vec![(RECORDING_PORT, port(PortType::Recording, "Recording")), (STYLE_PORT, opt(PortType::RecordingStyle, "Style"))])
        .outputs(vec![(CLIP_PORT, port(PortType::Clip, "Clip"))])
        .card(|ctx: &CardCtx<'_, ClipFromTake>| {
            let p = &ctx.params;
            let take = ctx.inputs.takes.get(RECORDING_PORT).copied().flatten();
            let kind = take.map_or(TakeKind::Browser, |t| t.kind);
            let speed = p.speed_for(kind);
            let styles = ctx.inputs.style.get(STYLE_PORT).map_or(0, Vec::len);
            let [follow, cursor] = style_warnings(ctx);
            let mut rows = vec![
                follow,
                cursor,
                take.map(|t| row(Icon::Video, format!("{} · {}", take_kind_label(t.kind), format_duration(t.duration_ms / speed)))),
            ];
            rows.extend(unused_styles(ctx, kind));
            rows.push(Some(row(Icon::Palette, if styles == 0 { "The usual style".to_owned() } else { plural(numbers::float(styles), "style setting") }).muted_if(styles == 0)));
            rows.push((!exact_eq(speed, 1.0)).then(|| row(Icon::Gauge, format!("{speed}× speed"))));
            rows
        })
        .compile(|ctx: &Ctx<'_, ClipFromTake>| {
            let p = &ctx.params;
            let rec = ctx.input(RECORDING_PORT).and_then(CompileValue::as_recording);
            let speed = p.speed_for(rec.map_or(TakeKind::Browser, RecordingValue::kind));
            let mut entries = entries_of(ctx.input(STYLE_PORT));
            entries.extend(rec.map_or_default(|r| r.edits.clone()));
            let duration_ms = rec.map_or(DEFAULT_TAKE_MS, |r| r.duration_ms);
            let frames = ms_to_frames(duration_ms / speed, ctx.settings.fps);
            let style = entries_of(ctx.input(STYLE_PORT));
            let props = match rec {
                Some(rec) => clip_from_take_props(rec, speed, &style)?,
                None => None,
            };
            let (Some(rec), Some(props)) = (rec, props) else {
                let placeholder = placeholder_clip(
                    ctx.node_id,
                    "Not recorded yet",
                    "Run to record the take. The camera moves appear here afterwards.",
                    frames,
                    ItemExtra::default(),
                );
                return Ok(Some(outputs([(CLIP_PORT, CompileValue::Clip(retime_with(placeholder, &entries, ctx.settings.fps)?))])));
            };
            let clip = recording_clip(ctx.node_id, props, frames, ItemExtra { recording: Some(rec.clone()), ..ItemExtra::default() });
            Ok(Some(outputs([(CLIP_PORT, CompileValue::Clip(retime_with(clip, &entries, ctx.settings.fps)?))])))
        })
}

fn camera_node() -> NodeDef {
    NodeDef::style_modifier::<Camera>(CAMERA_TYPE, "Camera", Category::Camera)
        .describe("How the camera follows the take: when it zooms to the actions, how close and how fast it moves.")
        .card(|ctx: &CardCtx<'_, Camera>| {
            let p = &ctx.params;
            vec![
                Some(if p.zoom {
                    row(Icon::ZoomIn, format!("Up to {}×", p.max_scale))
                } else {
                    row(Icon::ZoomOut, "No zoom").tone(Tone::Muted)
                }),
                p.zoom.then(|| row(Icon::Hourglass, format!("Hold {}", format_duration(f64::from(p.hold))))),
            ]
        })
        .restyle(|props: RecordingProps, p: &Camera, _| RecordingProps {
            camera: CameraProps {
                zoom: p.zoom,
                tuning: p.tuning(),
            },
            ..props
        })
}

fn shot_node() -> NodeDef {
    NodeDef::style_modifier::<Shot>(SHOT_TYPE, "Shot", Category::Camera)
        .describe("How the take sits in the frame: a floating window, the whole page, or a window on a plain ground.")
        .card(|ctx: &CardCtx<'_, Shot>| {
            let p = &ctx.params;
            vec![
                Some(row(Icon::AppWindow, p.style.label())),
                p.window_size.map(|size| row(Icon::Maximize, format!("{size}% of the frame"))),
            ]
        })
        .restyle(|props: RecordingProps, p: &Shot, _| RecordingProps {
            shot: p.apply(props.shot.clone()),
            ..props
        })
}

fn apply_style() -> NodeDef {
    NodeDef::of::<ApplyStyle>(
        "camera.applyStyle",
        "Apply style",
        Category::Camera,
        Phase::Compile,
    )
    .describe(
        "Gives every recording in a clip the same style, for takes joined in a Sequence or Layers.",
    )
    .inputs(vec![
        (CLIP_PORT, port(PortType::Clip, "Clip")),
        (STYLE_PORT, port(PortType::RecordingStyle, "Style")),
    ])
    .outputs(vec![(CLIP_PORT, port(PortType::Clip, "Clip"))])
    .card(|ctx: &CardCtx<'_, ApplyStyle>| {
        let entries = ctx.inputs.style.get(STYLE_PORT).map_or(0, Vec::len);
        vec![
            Some(
                row(
                    Icon::Palette,
                    plural(numbers::float(entries), "style setting"),
                )
                .muted_if(entries == 0),
            ),
            matches!(ctx.inputs.takes.get(CLIP_PORT), Some(None)).then(|| {
                row(Icon::TriangleAlert, "No recording in this clip")
                    .tone(Tone::Warn)
                    .titled("A style changes only recordings.")
            }),
        ]
    })
    .compile(|ctx: &Ctx<'_, ApplyStyle>| {
        let Some(clip) = ctx.input(CLIP_PORT).and_then(CompileValue::as_clip) else {
            return Ok(Some(IndexMap::default()));
        };
        let entries = entries_of(ctx.input(STYLE_PORT));
        let failed = std::cell::RefCell::new(None);
        let styled = patch_recordings(clip, &|props| match restyle_with(props.clone(), &entries) {
            Ok(p) => p,
            Err(e) => {
                failed.borrow_mut().get_or_insert(e);
                props
            }
        });
        if let Some(e) = failed.into_inner() {
            return Err(e.into());
        }
        Ok(Some(outputs([(
            CLIP_PORT,
            CompileValue::Clip(retime_with(styled, &entries, ctx.settings.fps)?),
        )])))
    })
}

fn manual_zoom() -> NodeDef {
    NodeDef::recording_modifier::<ManualZoom>("camera.manualZoom", "Zoom here", Category::Camera)
        .describe("Zooms to a spot of your choice at a set time.")
        .card(|ctx: &CardCtx<'_, ManualZoom>| {
            let p = &ctx.params;
            vec![
                Some(row(
                    Icon::ZoomIn,
                    format!("{}× at {}", p.scale, format_duration(f64::from(p.at_ms))),
                )),
                Some(row(
                    Icon::Hourglass,
                    format!("Hold {}", format_duration(f64::from(p.hold_ms))),
                )),
                take_row(
                    ctx.inputs.takes.get(RECORDING_PORT).copied().flatten(),
                    p.at_ms,
                ),
            ]
        })
        .restyle(|props: RecordingProps, p: &ManualZoom, node_id| {
            let (at, hold) = (f64::from(p.at_ms), f64::from(p.hold_ms));
            focus_beats(
                props,
                node_id,
                "zoom",
                p.scale,
                &[
                    FocusSpot {
                        id: "a",
                        t: at,
                        x: p.x,
                        y: p.y,
                        t_depart: Some((at - f64::from(p.lead_ms)).max(0.0)),
                    },
                    FocusSpot {
                        id: "b",
                        t: at + hold,
                        x: p.x,
                        y: p.y,
                        t_depart: None,
                    },
                ],
            )
        })
}

fn pan() -> NodeDef {
    NodeDef::recording_modifier::<Pan>("camera.pan", "Pan", Category::Camera)
        .describe("Moves the camera from one spot to another.")
        .card(|ctx: &CardCtx<'_, Pan>| {
            let p = &ctx.params;
            vec![
                Some(row(
                    Icon::Move,
                    format!(
                        "{} → {}",
                        format_duration(f64::from(p.at_ms)),
                        format_duration(f64::from(p.at_ms + p.duration_ms))
                    ),
                )),
                Some(row(Icon::ZoomIn, format!("{}×", p.scale))),
                take_row(
                    ctx.inputs.takes.get(RECORDING_PORT).copied().flatten(),
                    p.at_ms,
                ),
            ]
        })
        .restyle(|mut props: RecordingProps, p: &Pan, node_id| {
            if !p.zoom_out_when_far {
                props.camera.tuning.longest_pan_px = Some(NO_PAN_LIMIT_PX);
            }
            let at = f64::from(p.at_ms);
            focus_beats(
                props,
                node_id,
                "pan",
                p.scale,
                &[
                    FocusSpot {
                        id: "a",
                        t: at,
                        x: p.from_x,
                        y: p.from_y,
                        t_depart: None,
                    },
                    FocusSpot {
                        id: "b",
                        t: at + f64::from(p.duration_ms),
                        x: p.to_x,
                        y: p.to_y,
                        t_depart: None,
                    },
                ],
            )
        })
}

fn follow_cursor() -> NodeDef {
    NodeDef::style_modifier::<FollowCursor>(FOLLOW_CURSOR_TYPE, "Follow cursor", Category::Camera)
        .describe("Keeps the camera on the cursor, good for tall formats.")
        .card(|ctx: &CardCtx<'_, FollowCursor>| {
            let p = &ctx.params;
            vec![Some(row(
                Icon::LocateFixed,
                format!(
                    "{}× · re-aim every {}",
                    p.scale,
                    format_duration(f64::from(p.every_ms))
                ),
            ))]
        })
        .restyle(|mut props: RecordingProps, p: &FollowCursor, node_id| {
            if !p.zoom_out_on_jumps {
                props.camera.tuning.longest_pan_px = Some(NO_PAN_LIMIT_PX);
            }
            let beats = follow_cursor_beats(&props.log, p.scale, f64::from(p.every_ms), node_id);
            props.extra_beats.extend(beats);
            props
        })
}

fn recording_log(clip: &ClipValue) -> Option<&opsis_types::actions::ActionLog> {
    clip.recording
        .as_ref()
        .and_then(|TaggedRecording::Recording(r)| r.log.as_ref())
}

fn idle_speedup() -> NodeDef {
    NodeDef::style_modifier::<IdleSpeedup>(
        "camera.idleSpeedup",
        "Skip quiet parts",
        Category::Camera,
    )
    .describe("Speeds through moments where nothing happens.")
    .card(|ctx: &CardCtx<'_, IdleSpeedup>| {
        let p = &ctx.params;
        vec![Some(row(
            Icon::FastForward,
            format!(
                "{}× when quiet over {}",
                p.rate,
                format_duration(f64::from(p.threshold_ms))
            ),
        ))]
    })
    .retime(|clip: ClipValue, p: &IdleSpeedup, at| {
        let Some(log) = recording_log(&clip) else {
            return clip;
        };
        let spans = idle_spans(log, f64::from(p.threshold_ms), f64::from(p.pad_ms));
        if spans.is_empty() {
            return clip;
        }
        let segments: Vec<(f64, f64, f64)> = spans
            .into_iter()
            .map(|(from, to)| (from, to, p.rate))
            .collect();
        remap_clip(
            &clip,
            at.node_id,
            at.fps,
            &eased_segments(&segments, f64::from(p.ease_ms)),
        )
    })
}

fn speed_ramp() -> NodeDef {
    NodeDef::recording_modifier::<SpeedRamp>(SPEED_RAMP_TYPE, "Speed ramp", Category::Camera)
        .describe("Speeds up or slows down chosen parts.")
        .card(|ctx: &CardCtx<'_, SpeedRamp>| {
            let parts = ctx.params.segments.len();
            vec![Some(
                row(Icon::Gauge, plural(numbers::float(parts), "part")).muted_if(parts == 0),
            )]
        })
        .retime(|clip: ClipValue, p: &SpeedRamp, at| {
            let mut segments: Vec<(f64, f64, f64)> = p
                .segments
                .iter()
                .map(|s| (f64::from(s.from_ms), f64::from(s.to_ms), s.rate))
                .collect();
            segments.sort_by(|a, b| a.0.total_cmp(&b.0));
            remap_clip(
                &clip,
                at.node_id,
                at.fps,
                &eased_segments(&segments, f64::from(p.ease_ms)),
            )
        })
}

fn crop() -> NodeDef {
    NodeDef::clip_modifier::<Crop>("camera.crop", "Crop", Category::Camera)
        .describe("Cuts the recording down to an area.")
        .card(|ctx: &CardCtx<'_, Crop>| {
            let p = &ctx.params;
            let pct = |v: f64| percent_text(v, f64::INFINITY);
            vec![Some(row(
                Icon::Crop,
                format!(
                    "{} × {} from {}, {}",
                    pct(p.w),
                    pct(p.h),
                    pct(p.x),
                    pct(p.y)
                ),
            ))]
        })
        .compiles(|ctx: &Ctx<'_, Crop>| {
            let p = &ctx.params;
            with_clip(ctx, |clip| {
                let props = GroupProps {
                    crop: Some(Rect {
                        x: p.x,
                        y: p.y,
                        w: p.w,
                        h: p.h,
                    }),
                    crop_fill: p.fill.then_some(true),
                    ..GroupProps::default()
                };
                wrap_clip(clip, ctx.node_id, props, None)
            })
        })
}

fn device_frame() -> NodeDef {
    NodeDef::clip_modifier::<DeviceFrame>("camera.deviceFrame", "Device frame", Category::Camera)
        .describe("Puts the video inside a laptop or phone.")
        .card(|ctx: &CardCtx<'_, DeviceFrame>| {
            let p = &ctx.params;
            vec![
                Some(row(p.device.icon(), p.device.label())),
                (p.device == Device::Laptop && p.entrance != DeviceEntrance::None)
                    .then(|| row(Icon::Sparkles, p.entrance.label())),
            ]
        })
        .compiles(|ctx: &Ctx<'_, DeviceFrame>| {
            let p = &ctx.params;
            with_clip(ctx, |clip| {
                let props = element_meta(p.device.component()).map_or_else(Map::new, |meta| {
                    let mut values = meta.default_props.clone();
                    values.extend(p.element_values());
                    element_props(meta, &values)
                });
                let screened = patch_recordings(clip, &|r| RecordingProps {
                    shot: ShotProps {
                        in_device: Some(true),
                        ..r.shot
                    },
                    ..r
                });
                let wrapper = ElementRef {
                    component: p.device.component().to_owned(),
                    props,
                };
                wrap_clip(
                    &screened,
                    ctx.node_id,
                    GroupProps {
                        wrapper: Some(wrapper),
                        ..GroupProps::default()
                    },
                    Some(clip.duration_in_frames),
                )
            })
        })
}

fn window_frame() -> NodeDef {
    NodeDef::style_modifier::<WindowFrame>(WINDOW_FRAME_TYPE, "Window frame", Category::Camera)
        .describe("Draws a browser window's title bar around the recording.")
        .card(|ctx: &CardCtx<'_, WindowFrame>| {
            let p = &ctx.params;
            vec![
                Some(row(
                    Icon::AppWindow,
                    format!(
                        "{} · {}",
                        p.window_style.label(),
                        humanize(frame_theme_name(p.frame_theme))
                    ),
                )),
                Some(p.bar().row()),
            ]
        })
        .restyle(|props: RecordingProps, p: &WindowFrame, _| with_window_frame(props, p))
        .host_defaults(|platform: Platform| {
            let mut values = Values::new();
            values.insert(
                "windowStyle".into(),
                serde_json::to_value(host_window_style(platform)).expect("style serializes"),
            );
            values
        })
}

fn backdrop() -> NodeDef {
    NodeDef::style_modifier::<BackdropStyle>("camera.backdrop", "Backdrop", Category::Camera)
        .describe("The background behind the floating window: a studio image, colour or gradient.")
        .card(|ctx: &CardCtx<'_, BackdropStyle>| vec![Some(ctx.params.card_row())])
        .restyle(
            |props: RecordingProps, p: &BackdropStyle, _| match p.chosen() {
                Some(chosen) => RecordingProps {
                    shot: ShotProps {
                        backdrop: chosen,
                        backdrop_chosen: Some(true),
                        ..props.shot
                    },
                    ..props
                },
                None => props,
            },
        )
}

fn motion_blur() -> NodeDef {
    NodeDef::style_modifier::<MotionBlur>("camera.motionBlur", "Motion blur", Category::Camera)
        .describe("Adds blur to fast camera moves.")
        .card(|ctx: &CardCtx<'_, MotionBlur>| {
            vec![Some(toggle_row(Icon::Wind, ctx.params.enabled))]
        })
        .restyle(|props: RecordingProps, p: &MotionBlur, _| RecordingProps {
            shot: ShotProps {
                motion_blur: p.enabled,
                shutter_angle: Some(f64::from(p.shutter_angle)),
                ..props.shot
            },
            ..props
        })
}

pub fn nodes() -> Vec<NodeDef> {
    vec![
        clip_from_take().look(NodeLook::of("tabler:movie")),
        camera_node().look(NodeLook::of("lucide:scan-search")),
        shot_node().look(NodeLook::of("lucide:frame")),
        apply_style().look(NodeLook::of("lucide:paint-roller")),
        manual_zoom().look(NodeLook::of("lucide:zoom-in")),
        pan().look(NodeLook::of("lucide:move").moves(NodeMotion::Pan)),
        follow_cursor().look(NodeLook::of("lucide:crosshair").moves(NodeMotion::Track)),
        idle_speedup().look(NodeLook::of("lucide:skip-forward").moves(NodeMotion::NudgeRight)),
        speed_ramp().look(NodeLook::of("lucide:chart-spline")),
        crop().look(NodeLook::of("lucide:crop")),
        device_frame().look(NodeLook::of("lucide:laptop").morphs("lucide:smartphone")),
        window_frame().look(NodeLook::of("lucide:app-window-mac")),
        backdrop().look(NodeLook::of("tabler:background")),
        motion_blur().look(NodeLook::of("lucide:wind").moves(NodeMotion::Smear)),
    ]
}
