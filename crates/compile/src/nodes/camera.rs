use indexmap::IndexMap;
use opsis_macros::{choice, settings};
use opsis_types::actions::{Action, ActionKind, ActionLog, Point, Rect};
use opsis_types::cards::{CardRow, Icon, Tone};
use opsis_types::looks::{NodeLook, NodeMotion};
use opsis_types::numbers;
use opsis_types::recording::{
    AddressMode, CameraEasing, CameraProps, CameraStyle, CameraTuning, FootageChain, FrameTheme,
    RecordingDevice, RecordingProps, ScreenProps, ShotProps, WindowProps, WindowStyle,
};
use opsis_types::timeline::{ElementRef, GroupProps};
use opsis_types::values::{
    ClipValue, CompileValue, RecordingValue, StyleEntry, TaggedRecording, TakeKind, TakeSource,
};
use opsis_types::workflow::{DEFAULT_TAKE_MS, PortType};
use serde_json::Value;

use crate::asset_ref::base_name;
use crate::backdrops::Backdrop;
use crate::card_rows::toggle_row;
use crate::clip::{
    ItemExtra, clip_frame_of_recording, patch_recordings, placeholder_clip, recording_clip,
    wrap_clip,
};
use crate::color::is_color;
use crate::elements::element_meta;
use crate::errors::SettingsError;
use crate::node::{
    CLIP_PORT, CardCtx, Category, Ctx, InputTake, NodeDef, Phase, Platform, RECORDING_PORT, RowExt,
    STYLE_PORT, missing, opt, outputs, port, row, with_clip,
};
use crate::nodes::elements::element_props;
use crate::nodes::terminal::with_look;
use crate::params::{Params, Values};
use crate::recording::{
    DEFAULT_ADDRESS_BAR, DEFAULT_FRAME_THEME, DEFAULT_TAKE_SPEED, default_recording_props,
    follow_cursor_beats, host_window_style, idle_spans, point_rect, recording_speed,
};
use crate::registry::node_def;
use crate::remap::{RemapSegment, eased_segments, remap_duration, remap_from_segments};
use crate::terminal::default_look;

const NO_PAN_LIMIT_PX: f64 = 1.0e9;
use crate::style::{restyle_with, retime_with};
use crate::text::{display_address, format_duration, humanize, percent_text, plural};
use crate::time::{EPSILON, ms_to_frames, recording_ms_to_frames, whole_frames};

pub const CLIP_FROM_TAKE_TYPE: &str = "camera.take";
pub const CAMERA_TYPE: &str = "camera.camera";
pub const FRAME_TYPE: &str = "camera.frame";
pub const CAMERA_MOVE_TYPE: &str = "camera.move";
pub const MOTION_BLUR_TYPE: &str = "camera.motionBlur";
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

#[settings(eq)]
pub struct ClipFromTake {}

pub const fn usual_speed(kind: TakeKind) -> f64 {
    match kind {
        TakeKind::Browser => DEFAULT_TAKE_SPEED,
        TakeKind::Terminal => 1.0,
    }
}

#[choice]
pub enum Follows {
    #[default]
    #[label("The actions")]
    Actions,
    #[label("The cursor")]
    Cursor,
}

#[settings]
pub struct Camera {
    #[setting(
        "Follows",
        help = "The actions zooms to each action in turn. The cursor keeps the cursor in view, good for tall shapes."
    )]
    pub follows: Follows,
    #[setting("Zoom to the actions", default = true, show_if = { "follows": ["actions"] })]
    pub zoom: bool,
    #[setting(
        "Maximum zoom",
        default = 1.74,
        range(min = 1.1, max = 2.5),
        step = 0.02,
        unit = "×",
        help = "Higher zooms closer but can look soft.",
        show_if = { "follows": ["actions"], "zoom": ["true"] }
    )]
    pub max_scale: f64,
    #[setting(
        "Hold each zoom at least",
        default = 1300,
        range(min = 0, max = 10000),
        step = 50,
        unit = "ms",
        show_if = { "follows": ["actions"], "zoom": ["true"] }
    )]
    pub hold: u32,
    #[setting(
        "Least zoom",
        default = 1.18,
        range(min = 1, max = 2),
        step = 0.02,
        unit = "×",
        show_if = { "follows": ["actions"], "zoom": ["true"] },
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
        show_if = { "follows": ["actions"], "zoom": ["true"] },
        advanced
    )]
    pub first_max_scale: f64,
    #[setting(
        "Room around the target",
        default = 80,
        range(min = 0, max = 400),
        step = 5,
        unit = "px",
        show_if = { "follows": ["actions"], "zoom": ["true"] },
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
        show_if = { "follows": ["actions"], "zoom": ["true"] },
        advanced
    )]
    pub target_share: u32,
    #[setting(
        "Off centre",
        default = 0.26,
        range(min = 0, max = 0.5),
        step = 0.01,
        help = "How far a target sits from the middle, leaving room on the side it faces.",
        show_if = { "follows": ["actions"], "zoom": ["true"] },
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
        show_if = { "follows": ["actions"], "zoom": ["true"] },
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
        show_if = { "follows": ["actions"], "zoom": ["true"] },
        advanced
    )]
    pub stay_zoomed_within_ms: u32,
    #[setting(
        "Hold after typing",
        default = 400,
        range(min = 0, max = 5000),
        step = 50,
        unit = "ms",
        show_if = { "follows": ["actions"], "zoom": ["true"] },
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
        show_if = { "follows": ["actions"], "zoom": ["true"] },
        advanced
    )]
    pub rest_at_end_ms: u32,
    #[setting(
        "Cursor zoom",
        default = 1.35,
        range(min = 1.1, max = 2.5),
        step = 0.05,
        unit = "×",
        show_if = { "follows": ["cursor"] }
    )]
    pub cursor_scale: f64,
    #[setting(
        "Re-aim every",
        default = 700,
        range(min = 200, max = 5000),
        step = 50,
        unit = "ms",
        show_if = { "follows": ["cursor"] },
        advanced
    )]
    pub every_ms: u32,
    #[setting(
        "Zoom out on big jumps",
        default = true,
        help = "When the cursor jumps far, the camera zooms out and back in instead of sliding.",
        show_if = { "follows": ["cursor"] },
        advanced
    )]
    pub zoom_out_on_jumps: bool,
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

#[choice]
pub enum FrameKind {
    #[default]
    #[label("Window")]
    Window,
    #[label("Whole page")]
    Page,
    #[label("Device")]
    Device,
}

#[choice]
pub enum WindowShot {
    #[default]
    #[label("Floating")]
    Classic,
    #[label("On a stage")]
    Stage,
}

impl WindowShot {
    const fn style(self) -> CameraStyle {
        match self {
            Self::Classic => CameraStyle::Classic,
            Self::Stage => CameraStyle::Stage,
        }
    }
}

#[choice]
pub enum PageShot {
    #[default]
    #[label("Proof")]
    Proof,
    #[label("Ledger")]
    Ledger,
}

impl PageShot {
    const fn style(self) -> CameraStyle {
        match self {
            Self::Proof => CameraStyle::Proof,
            Self::Ledger => CameraStyle::Ledger,
        }
    }
}

#[choice]
pub enum BackgroundKind {
    #[default]
    #[label("The style's own")]
    Own,
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
pub struct Frame {
    #[setting("Kind")]
    pub kind: FrameKind,
    #[setting("Device", default = RecordingDevice::Laptop, show_if = { "kind": ["device"] })]
    pub device: RecordingDevice,
    #[setting("Camera style", show_if = { "kind": ["window"] })]
    pub window_shot: WindowShot,
    #[setting("Camera style", show_if = { "kind": ["page"] })]
    pub page_shot: PageShot,
    #[setting(
        "Window style",
        default = FrameChoice::Macos,
        help = "None leaves the window without a frame or title bar.",
        show_if = { "kind": ["window"] }
    )]
    pub window_style: FrameChoice,
    #[setting(
        "Frame colour",
        default = DEFAULT_FRAME_THEME,
        show_if = { "kind": ["window"], "windowStyle": ["macos", "windows", "linux"] }
    )]
    pub frame_theme: FrameTheme,
    #[setting(
        "Title bar",
        default = DEFAULT_ADDRESS_BAR,
        help = "Shows the page's address or the program's title in the title bar.",
        show_if = { "kind": ["window"], "windowStyle": ["macos", "windows", "linux"] }
    )]
    pub title_bar: bool,
    #[setting(
        "Title bar text",
        zero_label = "The take's own",
        help = "Shown instead of the page's address or the program's title, such as app.example.com.",
        show_if = { "kind": ["window"], "windowStyle": ["macos", "windows", "linux"], "titleBar": ["true"] }
    )]
    pub title: String,
    #[setting(
        "Show",
        default = AddressMode::Fixed,
        show_if = { "kind": ["window"], "windowStyle": ["macos", "windows", "linux"], "titleBar": ["true"], "title": ["*"] },
        show_if_take = ["browser"],
        advanced
    )]
    pub address_mode: AddressMode,
    #[setting(
        "Title bar font",
        widget = "font",
        zero_label = "The computer's",
        show_if = { "kind": ["window"], "windowStyle": ["macos", "windows", "linux"] },
        advanced
    )]
    pub font: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Window size",
        range(min = 40, max = 100),
        unit = "%",
        zero_label = "The style's own",
        help = "How much of the frame the window fills.",
        show_if = { "kind": ["window"] },
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
        show_if = { "kind": ["window"] },
        "type" = "integer",
        advanced
    )]
    pub corner_radius: Option<u32>,
    #[setting(
        "Window shadow",
        default = 1.0,
        range(min = 0, max = 1),
        step = 0.05,
        show_if = { "kind": ["window"] },
        advanced
    )]
    pub shadow: f64,
    #[setting("Background", show_if = { "kind": ["window"] })]
    pub background: BackgroundKind,
    #[setting("Studio image", default = Backdrop::Glaze, show_if = { "kind": ["window"], "background": ["image"] })]
    pub backdrop: Backdrop,
    #[setting("Colour", default = "#faf6f1", widget = "color", show_if = { "kind": ["window"], "background": ["solid"] })]
    pub color: String,
    #[setting(
        "Gradient",
        default = "linear-gradient(135deg, #1e1b4b 0%, #0f172a 100%)",
        widget = "gradient",
        show_if = { "kind": ["window"], "background": ["gradient"] }
    )]
    pub gradient: String,
    #[setting("Image", widget = "asset", accept = "image/*", show_if = { "kind": ["window"], "background": ["asset"] })]
    pub asset: String,
    #[setting(
        "Background blur",
        default = 48,
        range(min = 0, max = 200),
        step = 1,
        unit = "px",
        help = "Behind the whole page in another shape.",
        show_if = { "kind": ["page"] },
        advanced
    )]
    pub bleed_blur: u32,
    #[setting(
        "Background brightness",
        default = 0.55,
        range(min = 0, max = 1.5),
        step = 0.05,
        show_if = { "kind": ["page"] },
        advanced
    )]
    pub bleed_dim: f64,
    #[setting(
        "Lift the colours",
        default = true,
        help = "A touch more saturation, contrast and brightness on the screen.",
        advanced
    )]
    pub lift_colours: bool,
}

impl Params for Frame {
    fn check(&self) -> Result<(), SettingsError> {
        if self.kind != FrameKind::Window {
            return Ok(());
        }
        match self.background {
            BackgroundKind::Solid if !is_color(&self.color) => {
                Err(SettingsError::rule("Pick a colour"))
            }
            BackgroundKind::Gradient if self.gradient.trim().is_empty() => {
                Err(SettingsError::rule("Pick a gradient"))
            }
            BackgroundKind::Asset if self.asset.trim().is_empty() => {
                Err(SettingsError::rule("Choose an image"))
            }
            _ => Ok(()),
        }
    }
}

impl Frame {
    fn bar(&self) -> Bar<'_> {
        Bar {
            shown: self.title_bar,
            text: &self.title,
            mode: self.address_mode,
        }
    }

    fn chosen_background(&self) -> Option<String> {
        match self.background {
            BackgroundKind::Own => None,
            BackgroundKind::Image => Some(self.backdrop.id().to_owned()),
            BackgroundKind::Solid => Some(self.color.clone()),
            BackgroundKind::Gradient => Some(self.gradient.clone()),
            BackgroundKind::Asset => (!self.asset.is_empty()).then(|| self.asset.clone()),
        }
    }

    fn background_row(&self) -> Option<CardRow> {
        match self.background {
            BackgroundKind::Own => None,
            BackgroundKind::Image => Some(row(Icon::Image, humanize(self.backdrop.id()))),
            BackgroundKind::Solid => Some(row(Icon::Palette, self.color.clone())),
            BackgroundKind::Gradient => {
                Some(row(Icon::Blend, "Gradient").titled(self.gradient.clone()))
            }
            BackgroundKind::Asset if self.asset.is_empty() => {
                Some(missing(Icon::Image, "Choose an image"))
            }
            BackgroundKind::Asset => Some(row(Icon::Image, base_name(&self.asset))),
        }
    }

    fn kind_row(&self) -> CardRow {
        match self.kind {
            FrameKind::Window if self.window_style == FrameChoice::None => row(
                Icon::AppWindow,
                format!("{} · no window frame", self.window_shot.label()),
            ),
            FrameKind::Window => row(
                Icon::AppWindow,
                format!(
                    "{} · {} · {}",
                    self.window_shot.label(),
                    self.window_style.label(),
                    humanize(frame_theme_name(self.frame_theme))
                ),
            ),
            FrameKind::Page => row(
                Icon::AppWindow,
                format!("Whole page · {}", self.page_shot.label()),
            ),
            FrameKind::Device => row(
                match self.device {
                    RecordingDevice::Laptop => Icon::Laptop,
                    RecordingDevice::Phone => Icon::Smartphone,
                },
                self.device.label(),
            ),
        }
    }
}

fn framed(props: RecordingProps, p: &Frame) -> RecordingProps {
    let style = match p.kind {
        FrameKind::Window => p.window_shot.style(),
        FrameKind::Page => p.page_shot.style(),
        FrameKind::Device => props.shot.style,
    };
    let background = (p.kind == FrameKind::Window)
        .then(|| p.chosen_background())
        .flatten();
    let shot = ShotProps {
        style,
        device: (p.kind == FrameKind::Device).then_some(p.device),
        window_fit: p.window_size.map(|percent| f64::from(percent) / 100.0),
        radius: p.corner_radius.map(f64::from),
        rim_glow: Some(p.shadow),
        lift_colours: Some(p.lift_colours),
        bleed_blur: Some(f64::from(p.bleed_blur)),
        bleed_dim: Some(p.bleed_dim),
        backdrop_chosen: background.as_ref().map(|_| true),
        backdrop: background.unwrap_or_else(|| props.shot.backdrop.clone()),
        ..props.shot
    };
    let window = if p.kind == FrameKind::Window {
        let bar = p.bar();
        let text = bar.typed().map(str::to_owned);
        WindowProps {
            chrome: p.window_style.style().is_some(),
            style: p.window_style.style(),
            theme: Some(p.frame_theme),
            font: crate::fonts::font_prop(&p.font),
            bar: Some(bar.shown),
            mode: text.as_ref().map(|_| bar.mode),
            text,
        }
    } else {
        props.window.clone()
    };
    RecordingProps {
        shot,
        window,
        ..props
    }
}

#[settings]
pub struct CameraMove {
    #[setting(
        "At",
        default = 1000,
        range(min = 0, max = 600_000),
        step = 50,
        unit = "ms"
    )]
    pub at_ms: u32,
    #[setting(
        "Start from a spot",
        help = "The camera starts on one spot and moves to the other. Off, it moves to the spot from wherever it is."
    )]
    pub from_spot: bool,
    #[setting(
        "From",
        default = 0.3,
        range(min = 0, max = 1),
        step = 0.01,
        widget = "spot",
        time_key = "atMs",
        help = "Click the frame to choose where to start.",
        show_if = { "fromSpot": ["true"] }
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
        "Move for",
        default = 1500,
        range(min = 100, max = 600_000),
        step = 50,
        unit = "ms",
        show_if = { "fromSpot": ["true"] }
    )]
    pub move_ms: u32,
    #[setting(
        "To",
        default = 0.5,
        range(min = 0, max = 1),
        step = 0.01,
        widget = "spot",
        time_key = "atMs",
        help = "Click the frame to choose where the camera ends up."
    )]
    pub x: f64,
    #[setting(
        "To vertical",
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
        "Hold",
        default = 1500,
        range(min = 0, max = 600_000),
        step = 50,
        unit = "ms",
        help = "How long the camera stays on the spot it ends on."
    )]
    pub hold_ms: u32,
    #[setting(
        "Start moving before",
        default = 400,
        range(min = 0, max = 5000),
        step = 50,
        unit = "ms",
        help = "How long before the moment the camera starts to move.",
        show_if = { "fromSpot": ["false"] },
        advanced
    )]
    pub lead_ms: u32,
    #[setting(
        "Zoom out for far moves",
        default = true,
        help = "When the spots are far apart, the camera zooms out and back in instead of sliding.",
        show_if = { "fromSpot": ["true"] },
        advanced
    )]
    pub zoom_out_when_far: bool,
}

impl CameraMove {
    fn spots(&self) -> Vec<FocusSpot> {
        let at = f64::from(self.at_ms);
        if !self.from_spot {
            return vec![
                FocusSpot {
                    id: "a",
                    t: at,
                    x: self.x,
                    y: self.y,
                    t_depart: Some((at - f64::from(self.lead_ms)).max(0.0)),
                },
                FocusSpot {
                    id: "b",
                    t: at + f64::from(self.hold_ms),
                    x: self.x,
                    y: self.y,
                    t_depart: None,
                },
            ];
        }
        let arrive = at + f64::from(self.move_ms);
        let mut spots = vec![
            FocusSpot {
                id: "a",
                t: at,
                x: self.from_x,
                y: self.from_y,
                t_depart: None,
            },
            FocusSpot {
                id: "b",
                t: arrive,
                x: self.x,
                y: self.y,
                t_depart: None,
            },
        ];
        if self.hold_ms > 0 {
            spots.push(FocusSpot {
                id: "c",
                t: arrive + f64::from(self.hold_ms),
                x: self.x,
                y: self.y,
                t_depart: None,
            });
        }
        spots
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
                effects: None,
                preview: terminal.preview.clone(),
            };
            with_look(props, |_| look)
        }
    };
    props.speed = speed;
    props.camera = CameraProps {
        zoom: Camera::default().zoom,
        tuning: Camera::default().tuning(),
    };
    let frame = Frame::default();
    props.shot = ShotProps {
        style: frame.window_shot.style(),
        rim_glow: Some(frame.shadow),
        lift_colours: Some(frame.lift_colours),
        bleed_blur: Some(f64::from(frame.bleed_blur)),
        bleed_dim: Some(frame.bleed_dim),
        ..props.shot
    };
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

fn follows_cursor(entries: &[StyleEntry]) -> bool {
    entries
        .iter()
        .rev()
        .find(|e| e.kind == CAMERA_TYPE)
        .and_then(|e| e.params.get("follows"))
        .and_then(Value::as_str)
        == Some("cursor")
}

fn style_warnings(ctx: &CardCtx<'_, ClipFromTake>) -> [Option<CardRow>; 2] {
    let no_track = ctx
        .inputs
        .takes
        .get(RECORDING_PORT)
        .is_some_and(|t| t.is_some_and(|t| t.kind == TakeKind::Browser && !t.cursor_track));
    let entries = ctx
        .inputs
        .style
        .get(STYLE_PORT)
        .map_or(&[][..], Vec::as_slice);
    let styled = |kind: &str| entries.iter().any(|e| e.kind == kind);
    [
        (no_track && follows_cursor(entries)).then(|| {
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
        .filter_map(|e| {
            let t = &e.kind;
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

fn in_device(clip: ClipValue, node_id: &str, device: RecordingDevice) -> ClipValue {
    let Some(meta) = element_meta(device.element()) else {
        return clip;
    };
    let screened = patch_recordings(&clip, &|r| RecordingProps {
        shot: ShotProps {
            in_device: Some(true),
            ..r.shot
        },
        ..r
    });
    let props = element_props(meta, &meta.default_props);
    let group = GroupProps {
        wrapper: Some(ElementRef {
            component: meta.key.clone(),
            props,
        }),
        ..GroupProps::default()
    };
    wrap_clip(
        &screened,
        &format!("{node_id}:device"),
        group,
        Some(clip.duration_in_frames),
    )
}

fn clip_from_take() -> NodeDef {
    NodeDef::of::<ClipFromTake>(CLIP_FROM_TAKE_TYPE, "Clip from take", Category::Camera, Phase::Compile)
        .describe("Turns a take into a clip, shot the way its style says: the frame, the background and the camera moves. Browser takes play at 1.25×, terminal takes at their own speed; Speed changes it.")
        .inputs(vec![(RECORDING_PORT, port(PortType::Recording, "Recording")), (STYLE_PORT, opt(PortType::RecordingStyle, "Style"))])
        .outputs(vec![(CLIP_PORT, port(PortType::Clip, "Clip"))])
        .card(|ctx: &CardCtx<'_, ClipFromTake>| {
            let take = ctx.inputs.takes.get(RECORDING_PORT).copied().flatten();
            let kind = take.map_or(TakeKind::Browser, |t| t.kind);
            let speed = usual_speed(kind);
            let styles = ctx.inputs.style.get(STYLE_PORT).map_or(0, Vec::len);
            let [follow, cursor] = style_warnings(ctx);
            let mut rows = vec![
                follow,
                cursor,
                take.map(|t| row(Icon::Video, format!("{} · {}", take_kind_label(t.kind), format_duration(t.duration_ms / speed)))),
            ];
            rows.extend(unused_styles(ctx, kind));
            rows.push(Some(row(Icon::Palette, if styles == 0 { "The usual style".to_owned() } else { plural(numbers::float(styles), "style setting") }).muted_if(styles == 0)));
            rows
        })
        .compile(|ctx: &Ctx<'_, ClipFromTake>| {
            let rec = ctx.input(RECORDING_PORT).and_then(CompileValue::as_recording);
            let speed = usual_speed(rec.map_or(TakeKind::Browser, RecordingValue::kind));
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
            let device = props.shot.device;
            let clip = recording_clip(ctx.node_id, props, frames, ItemExtra { recording: Some(rec.clone()), ..ItemExtra::default() });
            let clip = retime_with(clip, &entries, ctx.settings.fps)?;
            let clip = match device {
                Some(device) => in_device(clip, ctx.node_id, device),
                None => clip,
            };
            Ok(Some(outputs([(CLIP_PORT, CompileValue::Clip(clip))])))
        })
}

fn camera_node() -> NodeDef {
    NodeDef::style_modifier::<Camera>(CAMERA_TYPE, "Camera", Category::Camera)
        .describe("How the camera follows the take: zooming to each action or keeping the cursor in view, how close and how fast it moves.")
        .card(|ctx: &CardCtx<'_, Camera>| {
            let p = &ctx.params;
            if p.follows == Follows::Cursor {
                return vec![Some(row(
                    Icon::LocateFixed,
                    format!(
                        "Follows the cursor · {}× · re-aim every {}",
                        p.cursor_scale,
                        format_duration(f64::from(p.every_ms))
                    ),
                ))];
            }
            vec![
                Some(if p.zoom {
                    row(Icon::ZoomIn, format!("Up to {}×", p.max_scale))
                } else {
                    row(Icon::ZoomOut, "No zoom").tone(Tone::Muted)
                }),
                p.zoom.then(|| row(Icon::Hourglass, format!("Hold {}", format_duration(f64::from(p.hold))))),
            ]
        })
        .restyle(|props: RecordingProps, p: &Camera, node_id| {
            let mut props = RecordingProps {
                camera: CameraProps {
                    zoom: p.zoom && p.follows == Follows::Actions,
                    tuning: p.tuning(),
                },
                ..props
            };
            if p.follows == Follows::Cursor {
                if !p.zoom_out_on_jumps {
                    props.camera.tuning.longest_pan_px = Some(NO_PAN_LIMIT_PX);
                }
                let beats = follow_cursor_beats(&props.log, p.cursor_scale, f64::from(p.every_ms), node_id);
                props.extra_beats.extend(beats);
            }
            props
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

fn camera_move() -> NodeDef {
    NodeDef::recording_modifier::<CameraMove>(CAMERA_MOVE_TYPE, "Camera move", Category::Camera)
        .describe("Moves the camera to a spot of your choice at a set time, from wherever it is or from another spot, and holds there.")
        .card(|ctx: &CardCtx<'_, CameraMove>| {
            let p = &ctx.params;
            let at = format_duration(f64::from(p.at_ms));
            vec![
                Some(if p.from_spot {
                    row(
                        Icon::Move,
                        format!(
                            "{at} → {} · {}×",
                            format_duration(f64::from(p.at_ms + p.move_ms)),
                            p.scale
                        ),
                    )
                } else {
                    row(Icon::ZoomIn, format!("{}× at {at}", p.scale))
                }),
                (p.hold_ms > 0).then(|| {
                    row(
                        Icon::Hourglass,
                        format!("Hold {}", format_duration(f64::from(p.hold_ms))),
                    )
                }),
                take_row(
                    ctx.inputs.takes.get(RECORDING_PORT).copied().flatten(),
                    p.at_ms,
                ),
            ]
        })
        .restyle(|mut props: RecordingProps, p: &CameraMove, node_id| {
            if p.from_spot && !p.zoom_out_when_far {
                props.camera.tuning.longest_pan_px = Some(NO_PAN_LIMIT_PX);
            }
            focus_beats(props, node_id, "move", p.scale, &p.spots())
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

fn frame_node() -> NodeDef {
    NodeDef::style_modifier::<Frame>(FRAME_TYPE, "Frame", Category::Camera)
        .describe("How the take sits in the frame: a window with its title bar on a background, the whole page, or the screen of a laptop or phone.")
        .card(|ctx: &CardCtx<'_, Frame>| {
            let p = &ctx.params;
            vec![
                Some(p.kind_row()),
                (p.kind == FrameKind::Window && p.window_style != FrameChoice::None).then(|| p.bar().row()),
                (p.kind == FrameKind::Window).then(|| p.background_row()).flatten(),
                (p.kind == FrameKind::Window)
                    .then(|| p.window_size.map(|size| row(Icon::Maximize, format!("{size}% of the frame"))))
                    .flatten(),
            ]
        })
        .restyle(|props: RecordingProps, p: &Frame, _| framed(props, p))
        .host_defaults(|platform: Platform| {
            let mut values = Values::new();
            values.insert(
                "windowStyle".into(),
                serde_json::to_value(host_window_style(platform)).expect("style serializes"),
            );
            values
        })
}

fn motion_blur() -> NodeDef {
    NodeDef::style_modifier::<MotionBlur>(MOTION_BLUR_TYPE, "Motion blur", Category::Camera)
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
        frame_node().look(NodeLook::of("lucide:app-window-mac")),
        apply_style().look(NodeLook::of("lucide:paint-roller")),
        camera_move().look(NodeLook::of("lucide:move").moves(NodeMotion::Pan)),
        idle_speedup().look(NodeLook::of("lucide:skip-forward").moves(NodeMotion::NudgeRight)),
        speed_ramp().look(NodeLook::of("lucide:chart-spline")),
        crop().look(NodeLook::of("lucide:crop")),
        crate::nodes::device_frame::device_frame(),
        motion_blur().look(NodeLook::of("lucide:wind").moves(NodeMotion::Smear)),
    ]
}
