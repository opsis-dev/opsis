use std::sync::LazyLock;

use opsis_macros::{choice, settings};
use opsis_types::actions::{ClickButton, Point};
use opsis_types::cards::{CardRow, Icon, Tone};
use opsis_types::looks::{NodeLook, NodeMotion};
use opsis_types::numbers;
use opsis_types::values::{
    BrowserAction, BrowserSessionValue, CompileValue, SessionBrowser, SessionStep,
};
use opsis_types::workflow::PortType;
use regex::Regex;
use serde::de::DeserializeOwned;

use crate::asset_ref::mime_subtype;
use crate::card_rows::{
    CODE_TEXT_MAX, LABELLED_TEXT_MAX, TAG_TEXT_MAX, element_row, element_row_or, file_row,
    join_parts, quote, quoted, quoted_row, quoted_row_with,
};
use crate::errors::{PathError, SettingsError};
use crate::node::{
    CardCtx, Category, Claim, Ctx, NodeDef, Outputs, Phase, PortSpec, RowExt, Rows, SESSION_PORT,
    missing, outputs, port, row,
};
use crate::params::Params;
use crate::text::{
    display_address, format_duration, format_when, non_blank_lines, plural, truncate,
};
use crate::vars::PAUSE_MARK;

#[settings(eq)]
pub struct Target {
    #[setting(
        "Element",
        help = "Pick it on the page, or type its visible text.",
        widget = "target",
        check = { "rule": "not-blank", "message": "Pick an element" }
    )]
    pub target: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Role of the picked element", hidden)]
    pub target_role: Option<String>,
}

#[settings(eq)]
pub struct PauseAfter {
    #[setting(
        "Pause after",
        default = 600,
        range(min = 0, max = 600_000),
        step = 50,
        unit = "ms",
        help = "How long the video lingers after this action."
    )]
    pub after: u32,
}

#[settings(eq)]
pub struct Silence {
    #[setting("No sound", advanced, help = "Action sounds leave this action out.")]
    pub silent: bool,
}

impl Silence {
    fn row(&self) -> Option<CardRow> {
        self.silent
            .then(|| row(Icon::VolumeX, "No sound").tone(Tone::Muted))
    }
}

#[settings(eq)]
pub struct Beat {
    #[serde(flatten)]
    pub pause: PauseAfter,
    #[setting(
        "Zoom group",
        help = "Actions in the same group share one zoom instead of zooming in and out each time.",
        advanced
    )]
    pub cluster: String,
    #[setting("Zoom in on this action", default = true)]
    pub zoom: bool,
}

#[settings(eq)]
pub struct Frame {
    #[setting(
        "Zoom to fit",
        widget = "target",
        help = "Zoom to a bigger area, like the whole card, instead of the element.",
        advanced
    )]
    pub frame: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Role of the picked element", hidden)]
    pub frame_role: Option<String>,
}

#[settings]
pub struct Pointer {
    #[serde(flatten)]
    pub frame: Frame,
    #[setting(
        "Zoom amount",
        range(min = 0, max = 3),
        step = 0.05,
        unit = "×",
        zero_label = "Auto",
        advanced
    )]
    pub zoom_scale: f64,
    #[setting(
        "Cursor travel time",
        range(min = 0, max = 10000),
        step = 50,
        unit = "ms",
        zero_label = "Auto",
        advanced
    )]
    pub travel_ms: u32,
}

#[choice]
pub enum ColorScheme {
    #[label("Light")]
    Light,
    #[label("Dark")]
    Dark,
    #[label("System")]
    NoPreference,
}

#[settings]
pub struct Open {
    #[setting(
        "Window width",
        default = 1920,
        range(min = 320, max = 3840),
        step = 10,
        unit = "px",
        presets_label = "Size",
        presets = [
            { "label": "1280 × 720", "values": { "viewportWidth": 1280, "viewportHeight": 720 } },
            { "label": "1440 × 900", "values": { "viewportWidth": 1440, "viewportHeight": 900 } },
            { "label": "1920 × 1080", "values": { "viewportWidth": 1920, "viewportHeight": 1080 } }
        ]
    )]
    pub viewport_width: u32,
    #[setting(
        "Window height",
        default = 1080,
        range(min = 240, max = 2160),
        step = 10,
        unit = "px"
    )]
    pub viewport_height: u32,
    #[setting(
        "Screenshot sharpness",
        default = 2.0,
        range(min = 1, max = 4),
        help = "Only affects screenshots. 2× or more gives crisp 4K images.",
        step = 1,
        unit = "×",
        advanced
    )]
    pub dpr: f64,
    #[setting("Mobile device")]
    pub mobile: bool,
    #[setting(
        "Browser identity",
        help = "Pretend to be a specific browser. Leave empty for the default.",
        advanced
    )]
    pub user_agent: String,
    #[setting(
        "Language",
        default = "en-US",
        help = "For example en-US, de-DE, ja-JP.",
        zero_label = "This computer's language",
        check = { "rule": "locale-or-empty", "message": "Use a language like en-US, de-DE or ja-JP" }
    )]
    pub locale: String,
    #[setting(
        "Time zone",
        widget = "timezone",
        check = { "rule": "time-zone-or-empty", "message": "Pick a time zone from the list" },
        advanced
    )]
    pub timezone: String,
    #[setting(
        "Theme",
        default = ColorScheme::Light
    )]
    pub color_scheme: ColorScheme,
    #[setting(
        "Recording sharpness",
        default = 1.0,
        range(min = 1, max = 4),
        help = "Records at a higher resolution so zoomed-in shots stay sharp.",
        step = 0.25,
        unit = "×",
        advanced
    )]
    pub capture_scale: f64,
    #[setting(
        "Show scrollbars",
        help = "Off hides them, as most product videos do.",
        advanced
    )]
    pub show_scrollbars: bool,
    #[setting(
        "Ask for less motion",
        help = "Tells the page the viewer prefers reduced motion, so it skips its own animations.",
        advanced
    )]
    pub reduce_motion: bool,
}

#[choice]
pub enum DoneWhen {
    #[label("The address changes")]
    UrlChange,
    #[label("An element appears")]
    Selector,
    #[label("I close the window")]
    WindowClosed,
}

#[settings(check, eq)]
pub struct SignIn {
    #[setting(
        "Sign-in page",
        default = "{{Site address}}/login",
        check = { "rule": "not-blank", "message": "Add the sign-in page" }
    )]
    pub url: String,
    #[setting(
        "Save as",
        default = "default",
        help = "The name the sign-in is kept under. Sign in nodes with the same name share it.",
        advanced
    )]
    pub secret_name: String,
    #[setting(
        "Finish when",
        default = DoneWhen::WindowClosed
    )]
    pub done_when: DoneWhen,
    #[setting(
        "Element that appears after signing in",
        help = "A CSS selector, e.g. .account-menu.",
        show_if = { "doneWhen": ["selector"] }
    )]
    pub selector: String,
    #[setting(
        "Give up after",
        default = 300,
        range(min = 10, max = 3600),
        unit = "s",
        advanced
    )]
    pub timeout_sec: u32,
}

impl Params for SignIn {
    fn check(&self) -> Result<(), SettingsError> {
        if self.done_when == DoneWhen::Selector && self.selector.trim().is_empty() {
            return Err(SettingsError::rule(
                "Add the element that appears after signing in",
            ));
        }
        Ok(())
    }
}

#[settings(eq)]
pub struct StartRecording {}

#[choice]
pub enum WaitUntil {
    #[label("Loaded")]
    Load,
    #[label("Started showing")]
    Domcontentloaded,
    #[label("Settled")]
    Networkidle,
    #[label("Responded")]
    Commit,
}

#[settings(eq)]
pub struct Navigate {
    #[setting("Page address", default = "{{Site address}}")]
    pub url: String,
    #[setting(
        "Wait until the page has",
        default = WaitUntil::Load,
        advanced
    )]
    pub wait_until: WaitUntil,
    #[setting(
        "Pause after",
        default = 700,
        range(min = 0, max = 600_000),
        step = 50,
        unit = "ms",
        help = "How long the video lingers on the new page."
    )]
    pub after: u32,
    #[setting(
        "Give up after",
        default = 30000,
        range(min = 1000, max = 600_000),
        step = 1000,
        unit = "ms",
        help = "How long to wait for the page before the take fails.",
        advanced
    )]
    pub timeout_ms: u32,
}

#[settings]
pub struct Click {
    #[serde(flatten)]
    pub target: Target,
    #[setting("Button", default = ClickButton::Left)]
    pub button: ClickButton,
    #[setting("Double-click", show_if = { "button": ["left"] })]
    pub double: bool,
    #[serde(flatten)]
    pub beat: Beat,
    #[serde(flatten)]
    pub pointer: Pointer,
    #[setting(
        "Hover before clicking",
        default = 220,
        range(min = 0, max = 10000),
        step = 50,
        unit = "ms",
        advanced
    )]
    pub hover_ms: u32,
    #[serde(flatten)]
    pub sound: Silence,
    #[setting(
        "Find it within",
        default = 15000,
        range(min = 1000, max = 120_000),
        step = 1000,
        unit = "ms",
        help = "How long to wait for the element to appear before the take fails.",
        advanced
    )]
    pub find_within_ms: u32,
    #[setting(
        "Scroll to it smoothly",
        help = "When the element is off screen. Off jumps to it.",
        advanced
    )]
    pub scroll_smoothly: bool,
}

impl Click {
    pub fn double_clicks(&self) -> bool {
        self.double && self.button == ClickButton::Left
    }
}

#[settings]
pub struct Type {
    #[serde(flatten)]
    pub target: Target,
    #[setting(
        "Text to type",
        widget = "textarea",
        pauses,
        help = "Use Add pause to hold still for a moment in the middle of typing.",
        check = { "rule": "not-blank", "message": "Add text to type" }
    )]
    pub text: String,
    #[setting("Pause length", range(min = 0, max = 10000), step = 50, unit = "ms")]
    pub chunk_pause_ms: u32,
    #[setting("Press Enter after typing")]
    pub press_enter: bool,
    #[serde(flatten)]
    pub beat: Beat,
    #[serde(flatten)]
    pub pointer: Pointer,
    #[serde(flatten)]
    pub sound: Silence,
    #[setting(
        "Hover before clicking in",
        default = 220,
        range(min = 0, max = 10000),
        step = 50,
        unit = "ms",
        advanced
    )]
    pub hover_ms: u32,
    #[setting(
        "Time per key",
        default = 30,
        range(min = 0, max = 1000),
        step = 5,
        unit = "ms",
        help = "How long each key is held, which sets the typing speed."
    )]
    pub key_ms: u32,
    #[setting(
        "Clear the field first",
        default = true,
        help = "Off adds to what the field already holds.",
        advanced
    )]
    pub clear_first: bool,
}

#[settings]
pub struct Hover {
    #[serde(flatten)]
    pub target: Target,
    #[serde(flatten)]
    pub beat: Beat,
    #[serde(flatten)]
    pub pointer: Pointer,
    #[setting(
        "Hover for",
        default = 600,
        range(min = 0, max = 10000),
        step = 50,
        unit = "ms"
    )]
    pub hover_ms: u32,
}

#[choice]
pub enum DragMode {
    #[label("Onto an element")]
    Element,
    #[label("By a distance")]
    By,
    #[label("Along a path")]
    Path,
}

#[choice]
pub enum DragMethod {
    #[label("Smooth, with the cursor")]
    Smooth,
    #[label("Quick, for pages that ignore the smooth drag")]
    Quick,
}

#[settings(check)]
pub struct Drag {
    #[setting(
        "Element to drag",
        help = "Pick it on the page, type its visible text, or give a spot as across, down.",
        widget = "target",
        check = { "rule": "not-blank", "message": "Pick an element" }
    )]
    pub target: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Role of the picked element", hidden)]
    pub target_role: Option<String>,
    #[setting(
        "Drag",
        default = DragMode::Element
    )]
    pub mode: DragMode,
    #[setting("Drop on", widget = "target", show_if = { "mode": ["element"] })]
    pub to: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Role of the picked element", hidden)]
    pub to_role: Option<String>,
    #[setting(
        "Across",
        default = 300,
        range(min = -10000, max = 10000),
        step = 10,
        unit = "px",
        show_if = { "mode": ["by"] }
    )]
    pub dx: i32,
    #[setting(
        "Down",
        range(min = -10000, max = 10000),
        step = 10,
        unit = "px",
        show_if = { "mode": ["by"] }
    )]
    pub dy: i32,
    #[setting(
        "Path",
        widget = "textarea",
        help = "One point per line, as across, down in pixels from where the drag starts.",
        presets = [
            { "label": "Circle", "values": { "path": "0, 0\n9, -46\n35, -85\n74, -111\n120, -120\n166, -111\n205, -85\n231, -46\n240, 0\n231, 46\n205, 85\n166, 111\n120, 120\n74, 111\n35, 85\n9, 46\n0, 0" } },
            { "label": "Tick", "values": { "path": "0, 0\n40, 40\n120, -80" } },
            { "label": "Zigzag", "values": { "path": "0, 0\n60, -60\n120, 0\n180, -60\n240, 0" } },
            { "label": "Square", "values": { "path": "0, 0\n200, 0\n200, 200\n0, 200\n0, 0" } }
        ],
        show_if = { "mode": ["path"] }
    )]
    pub path: String,
    #[setting(
        "Hold before moving",
        default = 150,
        range(min = 0, max = 5000),
        step = 50,
        unit = "ms",
        show_if = { "method": ["smooth"] },
        advanced
    )]
    pub hold_ms: u32,
    #[setting(
        "Drag time",
        range(min = 0, max = 10000),
        step = 50,
        unit = "ms",
        zero_label = "Auto",
        show_if = { "method": ["smooth"] }
    )]
    pub drag_ms: u32,
    #[setting(
        "Hold before dropping",
        default = 120,
        range(min = 0, max = 5000),
        step = 10,
        unit = "ms",
        show_if = { "method": ["smooth"] },
        advanced
    )]
    pub drop_hold_ms: u32,
    #[setting(
        "How",
        default = DragMethod::Smooth,
        advanced
    )]
    pub method: DragMethod,
    #[serde(flatten)]
    pub beat: Beat,
    #[serde(flatten)]
    pub pointer: Pointer,
    #[setting(
        "Hover before picking it up",
        default = 220,
        range(min = 0, max = 10000),
        step = 50,
        unit = "ms",
        advanced
    )]
    pub hover_ms: u32,
}

impl Params for Drag {
    fn check(&self) -> Result<(), SettingsError> {
        if self.mode == DragMode::Path {
            read_path(&self.path)
                .map_err(|problem| SettingsError::Rule(problem.to_string().into()))?;
        }
        if self.mode == DragMode::Element && self.to.trim().is_empty() {
            return Err(SettingsError::rule("Pick where to drop it"));
        }
        Ok(())
    }
}

#[choice]
pub enum ScrollMode {
    #[label("An element")]
    Element,
    #[label("A distance")]
    By,
}

#[choice]
pub enum ScrollAlign {
    #[label("At the top")]
    Start,
    #[default]
    #[label("In the middle")]
    Center,
    #[label("At the bottom")]
    End,
    #[label("Just in view")]
    Nearest,
}

impl ScrollAlign {
    pub const fn block(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Center => "center",
            Self::End => "end",
            Self::Nearest => "nearest",
        }
    }
}

#[settings(check, eq)]
pub struct Scroll {
    #[setting(
        "Scroll to",
        default = ScrollMode::Element
    )]
    pub mode: ScrollMode,
    #[setting("Element", widget = "target", show_if = { "mode": ["element"] })]
    pub target: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Role of the picked element", hidden)]
    pub target_role: Option<String>,
    #[setting(
        "Distance",
        default = 600,
        range(min = -20000, max = 20000),
        help = "Negative scrolls up.",
        step = 50,
        unit = "px",
        show_if = { "mode": ["by"] }
    )]
    pub delta_y: i32,
    #[setting(
        "Across",
        range(min = -20000, max = 20000),
        help = "Negative scrolls left.",
        step = 50,
        unit = "px",
        show_if = { "mode": ["by"] },
        advanced
    )]
    pub delta_x: i32,
    #[setting(
        "Where it ends up",
        show_if = { "mode": ["element"] },
        advanced
    )]
    pub align: ScrollAlign,
    #[setting("Smooth", default = true)]
    pub smooth: bool,
    #[setting(
        "Scroll time",
        default = 900,
        range(min = 0, max = 10000),
        step = 50,
        unit = "ms"
    )]
    pub duration_ms: u32,
    #[serde(flatten)]
    pub pause: PauseAfter,
}

impl Params for Scroll {
    fn check(&self) -> Result<(), SettingsError> {
        if self.mode == ScrollMode::Element && self.target.trim().is_empty() {
            return Err(SettingsError::rule("Pick an element"));
        }
        Ok(())
    }
}

#[settings]
pub struct Select {
    #[serde(flatten)]
    pub target: Target,
    #[setting("Option", check = { "rule": "not-blank", "message": "Choose an option" })]
    pub value: String,
    #[serde(flatten)]
    pub beat: Beat,
    #[serde(flatten)]
    pub pointer: Pointer,
    #[setting(
        "Hover before choosing",
        default = 220,
        range(min = 0, max = 10000),
        step = 50,
        unit = "ms",
        advanced
    )]
    pub hover_ms: u32,
}

#[settings(eq)]
pub struct Upload {
    #[setting(
        "File field",
        help = "Pick the upload field, or type its label.",
        widget = "target",
        check = { "rule": "not-blank", "message": "Pick an element" }
    )]
    pub target: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Role of the picked element", hidden)]
    pub target_role: Option<String>,
    #[setting(
        "File",
        widget = "asset",
        accept = "*/*",
        check = [
            { "rule": "not-blank", "message": "Choose a file" },
            { "rule": "in-project", "message": "Choose a file inside the project folder" }
        ]
    )]
    pub file: String,
    #[serde(flatten)]
    pub pause: PauseAfter,
}

#[settings(eq)]
pub struct PressKeys {
    #[setting(
        "Keys",
        default = "Enter",
        widget = "keys",
        help = "Click the field and press the shortcut."
    )]
    pub keys: String,
    #[serde(flatten)]
    pub pause: PauseAfter,
    #[serde(flatten)]
    pub sound: Silence,
}

#[choice]
pub enum WaitMode {
    #[label("An element")]
    Element,
    #[label("A quiet page")]
    NetworkIdle,
    #[label("An address")]
    Url,
    #[label("A fixed time")]
    Ms,
}

#[settings(check, eq)]
pub struct Wait {
    #[setting(
        "Wait for",
        default = WaitMode::Element
    )]
    pub mode: WaitMode,
    #[setting("Element", widget = "target", show_if = { "mode": ["element"] })]
    pub target: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Role of the picked element", hidden)]
    pub target_role: Option<String>,
    #[setting(
        "Address pattern",
        help = "Use * as a wildcard, e.g. */dashboard*.",
        show_if = { "mode": ["url"] }
    )]
    pub url: String,
    #[setting(
        "Time",
        default = 1000,
        range(min = 0, max = 600_000),
        step = 50,
        unit = "ms",
        show_if = { "mode": ["ms"] }
    )]
    pub ms: u32,
    #[setting(
        "Give up after",
        default = 20000,
        range(min = 500, max = 600_000),
        step = 50,
        unit = "ms",
        show_if = { "mode": ["element", "network-idle", "url"] },
        advanced
    )]
    pub timeout_ms: u32,
}

impl Params for Wait {
    fn check(&self) -> Result<(), SettingsError> {
        match self.mode {
            WaitMode::Element if self.target.trim().is_empty() => {
                Err(SettingsError::rule("Pick what to wait for"))
            }
            WaitMode::Url if self.url.trim().is_empty() => {
                Err(SettingsError::rule("Add an address pattern"))
            }
            _ => Ok(()),
        }
    }
}

#[settings(eq)]
pub struct Assert {
    #[setting("Text", check = { "rule": "not-blank", "message": "Add the text to look for" })]
    pub text: String,
    #[setting("Only inside (optional)", widget = "target", advanced)]
    pub target: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Role of the picked element", hidden)]
    pub target_role: Option<String>,
    #[setting(
        "Give up after",
        default = 10000,
        range(min = 500, max = 600_000),
        step = 50,
        unit = "ms",
        advanced
    )]
    pub timeout_ms: u32,
    #[setting(
        "Exact text",
        help = "Off matches the text inside longer text, in any case.",
        advanced
    )]
    pub exact: bool,
}

#[settings(eq)]
pub struct Hoist {
    #[serde(flatten)]
    pub target: Target,
    #[setting(
        "Find it within",
        default = 15000,
        range(min = 1000, max = 120_000),
        step = 1000,
        unit = "ms",
        advanced
    )]
    pub timeout_ms: u32,
}

#[settings(eq)]
pub struct SetState {
    #[setting(
        "Script",
        default = "localStorage.setItem('onboarded', '1');",
        widget = "code"
    )]
    pub script: String,
    #[setting("Reload the page after")]
    pub reload: bool,
    #[setting(
        "On every page",
        help = "Runs it on every page that loads, not only once.",
        advanced
    )]
    pub every_page: bool,
}

#[settings(eq)]
pub struct SwitchTheme {
    #[setting("Theme", default = ColorScheme::Dark)]
    pub color_scheme: ColorScheme,
}

#[settings(eq)]
pub struct ChangeLanguage {
    #[setting(
        "Language",
        default = "de-DE",
        help = "For example en-US, de-DE, ja-JP.",
        zero_label = "This computer's language",
        check = { "rule": "locale-or-empty", "message": "Use a language like en-US, de-DE or ja-JP" }
    )]
    pub locale: String,
    #[setting(
        "Reload the page after",
        default = true,
        help = "Most sites read the language only when a page loads."
    )]
    pub reload: bool,
}

#[settings(eq)]
pub struct MockRoute {
    #[setting(
        "Address pattern",
        default = "**/api/**",
        help = "Use * as a wildcard.",
        check = { "rule": "not-blank", "message": "Add an address pattern" }
    )]
    pub url: String,
    #[setting("Status code", default = 200, range(min = 100, max = 599))]
    pub status: u32,
    #[setting("Type", default = "application/json", advanced)]
    pub content_type: String,
    #[setting("Response", default = r#"{"ok":true}"#, widget = "code")]
    pub body: String,
    #[setting(
        "Reply after",
        range(min = 0, max = 60000),
        step = 50,
        unit = "ms",
        help = "Holds the reply back, to show a loading state.",
        advanced
    )]
    pub delay_ms: u32,
}

#[choice]
pub enum ClockMode {
    #[label("Frozen")]
    Fixed,
    #[label("Paused")]
    Paused,
}

#[settings(eq)]
pub struct FreezeClock {
    #[setting(
        "Date and time",
        default = "2026-10-01T09:41:00",
        help = "As a clock in the browser's time zone shows it.",
        widget = "datetime",
        check = { "rule": "wall-time", "message": "Pick a date and time" }
    )]
    pub time: String,
    #[setting(
        "Clock",
        default = ClockMode::Fixed
    )]
    pub mode: ClockMode,
}

#[settings(eq)]
pub struct HideElements {
    #[setting("Cookie banners", default = true)]
    pub cookie_banners: bool,
    #[setting("Chat widgets", default = true)]
    pub chat_widgets: bool,
    #[setting(
        "Also hide",
        widget = "textarea",
        help = "CSS selectors, one per line."
    )]
    pub selectors: String,
}

#[settings(eq)]
pub struct Chapter {
    #[setting("Title", default = "Chapter")]
    pub title: String,
    #[setting("Description")]
    pub description: String,
    #[setting("Show a chapter card in the video")]
    pub show_in_video: bool,
    #[setting(
        "Card shown for",
        default = 2000,
        range(min = 0, max = 600_000),
        step = 50,
        unit = "ms",
        show_if = { "showInVideo": ["true"] }
    )]
    pub duration_ms: u32,
    #[setting(
        "Pause after the card",
        default = 2000,
        range(min = 0, max = 600_000),
        step = 50,
        unit = "ms",
        show_if = { "showInVideo": ["true"] },
        advanced
    )]
    pub after_ms: u32,
    #[setting("Card colour", default = "rgba(0, 0, 0, 0.7)", widget = "color", check = colour, show_if = { "showInVideo": ["true"] }, advanced)]
    pub card_color: String,
    #[setting("Edge colour", default = "rgba(255, 255, 255, 0.15)", widget = "color", check = colour, show_if = { "showInVideo": ["true"] }, advanced)]
    pub edge_color: String,
    #[setting("Text colour", default = "#ffffff", widget = "color", check = colour, help = "The description is a softer shade of it.", show_if = { "showInVideo": ["true"] }, advanced)]
    pub text_color: String,
    #[setting("Title size", default = 28, range(min = 10, max = 120), unit = "px", show_if = { "showInVideo": ["true"] }, advanced)]
    pub title_size: u32,
    #[setting("Description size", default = 15, range(min = 8, max = 80), unit = "px", show_if = { "showInVideo": ["true"] }, advanced)]
    pub description_size: u32,
    #[setting("Card width", default = 560, range(min = 200, max = 1600), step = 10, unit = "px", show_if = { "showInVideo": ["true"] }, advanced)]
    pub card_width: u32,
    #[setting(
        "Card position",
        show_if = { "showInVideo": ["true"] },
        advanced
    )]
    pub position: CardPosition,
    #[setting("Blur behind", default = 2, range(min = 0, max = 40), unit = "px", show_if = { "showInVideo": ["true"] }, advanced)]
    pub blur: u32,
    #[setting("Fade", default = 300, range(min = 0, max = 3000), step = 50, unit = "ms", show_if = { "showInVideo": ["true"] }, advanced)]
    pub fade_ms: u32,
}

#[choice]
pub enum CardPosition {
    #[label("Top")]
    Top,
    #[default]
    #[label("Middle")]
    Center,
    #[label("Bottom")]
    Bottom,
}

impl CardPosition {
    pub const fn flex(self) -> &'static str {
        match self {
            Self::Top => "flex-start",
            Self::Center => "center",
            Self::Bottom => "flex-end",
        }
    }
}

#[settings(eq)]
pub struct OverlayHtml {
    #[setting(
        "HTML",
        default = r#"<div style="font:600 32px system-ui">Hello</div>"#,
        widget = "code"
    )]
    pub html: String,
    #[setting(
        "Shown for",
        default = 2000,
        range(min = 0, max = 600_000),
        step = 50,
        unit = "ms",
        help = "The next actions carry on while it's shown.",
        zero_label = "Until the end"
    )]
    pub duration_ms: u32,
    #[setting("Fade", range(min = 0, max = 3000), step = 50, unit = "ms", advanced)]
    pub fade_ms: u32,
}

static POINT_TEXT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*(-?\d+(?:\.\d+)?)\s*,\s*(-?\d+(?:\.\d+)?)\s*$").expect("pattern")
});

pub fn parse_point(text: &str) -> Option<Point> {
    let m = POINT_TEXT.captures(text)?;
    Some(Point {
        x: m[1].parse().ok()?,
        y: m[2].parse().ok()?,
    })
}

pub fn read_path(text: &str) -> Result<Vec<Point>, PathError> {
    let mut points = Vec::new();
    for (i, line) in text.split('\n').enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        points.push(parse_point(line).ok_or(PathError::Unreadable(i + 1))?);
    }
    if points.is_empty() {
        return Err(PathError::Empty);
    }
    if points.iter().all(|p| p.x == 0.0 && p.y == 0.0) {
        return Err(PathError::Still);
    }
    Ok(points)
}

pub use opsis_types::secrets::{secret_file_name, secret_name};

pub fn chord_keys(chord: &str) -> Vec<String> {
    let chars: Vec<char> = chord.chars().collect();
    let mut keys = Vec::new();
    let mut current = String::new();
    for (i, c) in chars.iter().enumerate() {
        if *c == '+' && i + 1 < chars.len() {
            keys.push(current.trim().to_owned());
            current.clear();
        } else {
            current.push(*c);
        }
    }
    keys.push(current.trim().to_owned());
    keys
}

pub const OPEN_BROWSER_TYPE: &str = "browser.open";

fn session_port() -> PortSpec {
    port(PortType::BrowserSession, "Browser")
}

fn append_step<P>(ctx: &Ctx<'_, P>, action: BrowserAction) -> Outputs {
    let Some(CompileValue::BrowserSession(session)) = ctx.input(SESSION_PORT) else {
        return Outputs::new();
    };
    let mut session = session.clone();
    session.steps.push(SessionStep {
        id: ctx.node_id.to_owned(),
        kind: action,
        label: ctx.label.map(str::to_owned),
        params: ctx.values.clone(),
    });
    outputs([(SESSION_PORT, CompileValue::BrowserSession(session))])
}

fn chain<P: Params>(action: BrowserAction, title: &str, description: &str) -> NodeDef {
    NodeDef::of::<P>(action.kind(), title, Category::Browser, Phase::Capture)
        .describe(description)
        .virtual_node()
        .inputs(vec![(SESSION_PORT, session_port())])
        .outputs(vec![(SESSION_PORT, session_port())])
        .compiles(move |ctx: &Ctx<'_, P>| append_step(ctx, action))
}

fn action<P: Params>(action: BrowserAction, title: &str, description: &str) -> NodeDef {
    chain::<P>(action, title, description).action()
}

fn card<P: DeserializeOwned + 'static>(def: NodeDef, f: fn(&P) -> Rows) -> NodeDef {
    def.card(move |ctx: &CardCtx<'_, P>| f(&ctx.params))
}

fn pause(after: u32) -> Option<CardRow> {
    (after > 0).then(|| {
        row(
            Icon::Timer,
            format!("Pause {}", format_duration(f64::from(after))),
        )
    })
}

fn give_up_row(timeout_ms: u32) -> CardRow {
    row(
        Icon::TimerOff,
        format!("Give up after {}", format_duration(f64::from(timeout_ms))),
    )
    .tone(Tone::Muted)
}

fn zoom_row(beat: &Beat) -> Option<CardRow> {
    if !beat.zoom {
        return Some(row(Icon::ZoomOut, "No zoom").tone(Tone::Muted));
    }
    (!beat.cluster.trim().is_empty()).then(|| {
        row(
            Icon::ZoomIn,
            format!("Zoom group “{}”", truncate(&beat.cluster, TAG_TEXT_MAX)),
        )
    })
}

fn with_beat(mut rows: Rows, beat: &Beat) -> Rows {
    rows.push(pause(beat.pause.after));
    rows.push(zoom_row(beat));
    rows
}

fn target_row(icon: Icon, t: &Target) -> CardRow {
    element_row(icon, &t.target, t.target_role.as_deref())
}

fn click_row(p: &Click) -> CardRow {
    let r = target_row(Icon::MousePointerClick, &p.target);
    let how = if p.button == ClickButton::Right {
        "Right-click"
    } else if p.double_clicks() {
        "Double-click"
    } else {
        return r;
    };
    if r.tone == Some(Tone::Warn) {
        return r;
    }
    CardRow {
        text: format!("{how} {}", r.text),
        ..r
    }
}

fn address_row(url: &str) -> CardRow {
    row(Icon::Link, display_address(url, true)).titled(url)
}

fn drop_row(p: &Drag) -> CardRow {
    match p.mode {
        DragMode::Element => {
            let r = element_row_or(
                Icon::Move,
                &p.to,
                p.to_role.as_deref(),
                "Pick where to drop it",
            );
            if r.tone == Some(Tone::Warn) {
                r
            } else {
                CardRow {
                    text: format!("Onto {}", r.text),
                    ..r
                }
            }
        }
        DragMode::By => row(Icon::Move, format!("By {}, {} px", p.dx, p.dy)),
        DragMode::Path => match read_path(&p.path) {
            Ok(points) => row(
                Icon::Spline,
                format!("Along {}", plural(numbers::float(points.len()), "point")),
            ),
            Err(problem) => missing(Icon::Spline, problem.to_string()),
        },
    }
}

fn wait_row(p: &Wait) -> CardRow {
    match p.mode {
        WaitMode::Element => element_row_or(
            Icon::Eye,
            &p.target,
            p.target_role.as_deref(),
            "Pick what to wait for",
        ),
        WaitMode::NetworkIdle => row(Icon::Wifi, "Until the page is quiet"),
        WaitMode::Url if p.url.trim().is_empty() => missing(Icon::Link, "Add an address pattern"),
        WaitMode::Url => row(
            Icon::Link,
            format!("Until {}", display_address(&p.url, true)),
        ),
        WaitMode::Ms => row(Icon::Hourglass, format_duration(f64::from(p.ms))),
    }
}

static TAG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<[^>]+>").expect("pattern"));

fn open() -> NodeDef {
    NodeDef::of::<Open>(
        OPEN_BROWSER_TYPE,
        "Open browser",
        Category::Browser,
        Phase::Capture,
    )
    .describe("Starts a browser window for the actions that follow: size, device and language.")
    .virtual_node()
    .outputs(vec![(SESSION_PORT, session_port())])
    .card(|ctx: &CardCtx<'_, Open>| {
        let p = &ctx.params;
        let size = format!("{} × {}", p.viewport_width, p.viewport_height);
        let dark = p.color_scheme == ColorScheme::Dark;
        let look = join_parts([
            dark.then_some("Dark theme"),
            (!p.locale.is_empty() && p.locale != "en-US").then_some(p.locale.as_str()),
        ]);
        vec![
            Some(if p.mobile {
                row(Icon::Smartphone, format!("{size} · Mobile"))
            } else {
                row(Icon::Monitor, size)
            }),
            (!look.is_empty()).then(|| row(if dark { Icon::Moon } else { Icon::Languages }, look)),
        ]
    })
    .compiles(|ctx: &Ctx<'_, Open>| {
        outputs([(
            SESSION_PORT,
            CompileValue::BrowserSession(BrowserSessionValue {
                browser: SessionBrowser {
                    node_id: ctx.node_id.to_owned(),
                    params: ctx.values.clone(),
                },
                steps: Vec::new(),
                address: None,
            }),
        )])
    })
}

fn go_to_page() -> NodeDef {
    chain::<Navigate>(
        BrowserAction::Navigate,
        "Go to page",
        "Opens a page address.",
    )
    .compiles(|ctx: &Ctx<'_, Navigate>| {
        let mut out = append_step(ctx, BrowserAction::Navigate);
        if let Some(CompileValue::BrowserSession(session)) = out.get_mut(SESSION_PORT) {
            session.address = Some(ctx.params.url.clone());
        }
        out
    })
}

fn sign_in() -> NodeDef {
    NodeDef::of::<SignIn>(BrowserAction::SignIn.kind(), "Sign in", Category::Browser, Phase::Capture)
        .describe(
            "Opens a window where you sign in yourself, once. Opsis keeps the sign-in, encrypted, and the actions after this one use it.",
        )
        .inputs(vec![(SESSION_PORT, session_port())])
        .outputs(vec![(SESSION_PORT, session_port())])
        .compiles(|ctx: &Ctx<'_, SignIn>| append_step(ctx, BrowserAction::SignIn))
        .executes()
        .claims(|p: &SignIn| vec![Claim::SignIn(secret_name(&p.secret_name))])
        .card(|ctx: &CardCtx<'_, SignIn>| {
            let p = &ctx.params;
            let name = secret_name(&p.secret_name);
            let secrets = ctx.inputs.secrets.as_ref();
            let saved = secrets.is_some_and(|s| s.contains(&secret_file_name(&name)));
            vec![
                Some(row(Icon::KeyRound, format!("Saves as {}", quote(&name, LABELLED_TEXT_MAX)))),
                (!p.url.trim().is_empty()).then(|| address_row(&p.url)),
                secrets.map(|_| {
                    if saved { row(Icon::ShieldCheck, "Signed in") } else { row(Icon::ShieldAlert, "Not signed in yet").tone(Tone::Warn) }
                }),
                Some(row(Icon::RefreshCw, "Applies from the next page load").tone(Tone::Muted)),
            ]
        })
}

fn type_rows(p: &Type) -> Rows {
    let typed = p.text.replace(PAUSE_MARK, " … ");
    let enter = if p.press_enter { ", then Enter" } else { "" };
    let typed_row = quoted_row_with(Icon::Keyboard, &typed, "Add text to type", enter);
    let typed_row = if typed_row.tone.is_none() {
        typed_row.titled(typed)
    } else {
        typed_row
    };
    with_beat(
        vec![
            Some(target_row(Icon::Crosshair, &p.target)),
            Some(typed_row),
            p.sound.row(),
        ],
        &p.beat,
    )
}

fn hover_rows(p: &Hover) -> Rows {
    let hover = if p.hover_ms > 0 {
        row(
            Icon::Hourglass,
            format!("Hover {}", format_duration(f64::from(p.hover_ms))),
        )
    } else {
        row(Icon::Hourglass, "Passes over it").tone(Tone::Muted)
    };
    with_beat(
        vec![
            Some(target_row(Icon::MousePointer2, &p.target)),
            Some(hover),
        ],
        &p.beat,
    )
}

fn scroll_rows(p: &Scroll) -> Rows {
    let to = match p.mode {
        ScrollMode::Element => element_row(Icon::Scroll, &p.target, p.target_role.as_deref()),
        ScrollMode::By => row(
            Icon::Scroll,
            format!(
                "{} {} px",
                if p.delta_y >= 0 { "Down" } else { "Up" },
                p.delta_y.abs()
            ),
        ),
    };
    vec![
        Some(to),
        p.smooth.then(|| {
            row(
                Icon::Timer,
                format!("Smooth, {}", format_duration(f64::from(p.duration_ms))),
            )
        }),
        pause(p.pause.after),
    ]
}

fn press_rows(p: &PressKeys) -> Rows {
    let keys = if p.keys.trim().is_empty() {
        missing(Icon::Command, "Press a shortcut")
    } else {
        row(Icon::Command, chord_keys(&p.keys).join(" + "))
    };
    vec![Some(keys), p.sound.row(), pause(p.pause.after)]
}

fn set_state_rows(p: &SetState) -> Rows {
    let script = if p.script.trim().is_empty() {
        missing(Icon::Code, "Add a script")
    } else {
        row(
            Icon::Code,
            truncate(p.script.split('\n').next().unwrap_or(""), CODE_TEXT_MAX),
        )
        .titled(p.script.clone())
    };
    vec![
        Some(script),
        p.reload.then(|| row(Icon::RefreshCw, "Then reload")),
    ]
}

fn mock_route_rows(p: &MockRoute) -> Rows {
    let subtype = mime_subtype(&p.content_type).to_uppercase();
    let url = if p.url.trim().is_empty() {
        missing(Icon::Server, "Add an address pattern")
    } else {
        row(Icon::Server, truncate(&p.url, CODE_TEXT_MAX)).titled(p.url.clone())
    };
    let reply = if subtype.is_empty() {
        p.status.to_string()
    } else {
        format!("{} {subtype}", p.status)
    };
    vec![Some(url), Some(row(Icon::Reply, reply))]
}

fn hide_rows(p: &HideElements) -> Rows {
    let extra = non_blank_lines(&p.selectors).len();
    let parts = join_parts([
        p.cookie_banners.then(|| "Cookie banners".to_owned()),
        p.chat_widgets.then(|| "Chat widgets".to_owned()),
        (extra > 0).then(|| format!("+{extra} more")),
    ]);
    vec![Some(if parts.is_empty() {
        row(Icon::EyeOff, "Nothing yet").tone(Tone::Muted)
    } else {
        row(Icon::EyeOff, parts)
    })]
}

fn overlay_rows(p: &OverlayHtml) -> Rows {
    let shown = truncate(&TAG.replace_all(&p.html, " "), CODE_TEXT_MAX);
    let length = if p.duration_ms != 0 {
        format_duration(f64::from(p.duration_ms))
    } else {
        "Until the end".to_owned()
    };
    vec![
        Some(
            row(
                Icon::CodeXml,
                if shown.is_empty() {
                    "HTML".to_owned()
                } else {
                    shown
                },
            )
            .titled(p.html.clone()),
        ),
        Some(row(Icon::Timer, length)),
    ]
}

pub fn nodes() -> Vec<NodeDef> {
    vec![
        open().look(NodeLook::of("lucide:app-window")),
        sign_in().look(NodeLook::of("lucide:key").moves(NodeMotion::Twist)),
        card(
            chain::<StartRecording>(
                BrowserAction::StartRecording,
                "Start recording",
                "The video starts here. The actions before it run off camera, like signing in and the first page load.",
            ),
            |_: &StartRecording| vec![Some(row(Icon::Video, "Video starts here"))],
        ).look(NodeLook::of("lucide:circle-play").morphs("lucide:circle-dot")),
        card(go_to_page(), |p: &Navigate| {
            let url = if p.url.trim().is_empty() {
                missing(Icon::Link, "Add an address")
            } else {
                address_row(&p.url)
            };
            vec![Some(url), pause(p.after)]
        }).look(NodeLook::of("lucide:globe")),
        card(
            action::<Click>(
                BrowserAction::Click,
                "Click",
                "Moves the cursor to an element and clicks it.",
            ),
            |p: &Click| with_beat(vec![Some(click_row(p)), p.sound.row()], &p.beat),
        ).look(NodeLook::of("lucide:mouse-pointer-click").moves(NodeMotion::Press)),
        card(
            action::<Type>(
                BrowserAction::Type,
                "Type text",
                "Types into a field, key by key.",
            ),
            type_rows,
        ).look(NodeLook::of("lucide:text-cursor-input")),
        card(
            action::<Hover>(
                BrowserAction::Hover,
                "Hover",
                "Moves the cursor over an element without clicking, so a menu or tooltip can appear.",
            ),
            hover_rows,
        ).look(NodeLook::of("lucide:pointer").moves(NodeMotion::Bob)),
        card(
            action::<Drag>(
                BrowserAction::Drag,
                "Drag",
                "Drags an element onto another one, by a distance, or along a path, like moving a card or drawing a shape.",
            ),
            |p: &Drag| {
                with_beat(
                    vec![
                        Some(element_row(Icon::Grab, &p.target, p.target_role.as_deref())),
                        Some(drop_row(p)),
                    ],
                    &p.beat,
                )
            },
        ).look(NodeLook::of("lucide:hand").morphs("lucide:hand-grab")),
        card(
            action::<Scroll>(
                BrowserAction::Scroll,
                "Scroll",
                "Scrolls to an element or by a distance.",
            ),
            scroll_rows,
        ).look(NodeLook::of("lucide:mouse").moves(NodeMotion::NudgeDown)),
        card(
            action::<Select>(
                BrowserAction::Select,
                "Choose option",
                "Picks an option in a dropdown.",
            ),
            |p: &Select| {
                vec![
                    Some(target_row(Icon::List, &p.target)),
                    Some(quoted_row(Icon::Check, &p.value, "Choose an option")),
                    pause(p.beat.pause.after),
                ]
            },
        ).look(NodeLook::of("lucide:square-chevron-down").morphs("lucide:list-checks")),
        card(
            action::<Upload>(
                BrowserAction::Upload,
                "Upload file",
                "Attaches a file from your project to a file field.",
            ),
            |p: &Upload| {
                let file = if p.file.is_empty() {
                    missing(Icon::File, "Choose a file")
                } else {
                    file_row(Icon::File, &p.file)
                };
                vec![
                    Some(element_row(
                        Icon::Upload,
                        &p.target,
                        p.target_role.as_deref(),
                    )),
                    Some(file),
                    pause(p.pause.after),
                ]
            },
        ).look(NodeLook::of("lucide:file-up").moves(NodeMotion::NudgeUp)),
        card(
            action::<PressKeys>(
                BrowserAction::Press,
                "Press keys",
                "Presses a key or shortcut, like Ctrl+K. Shows in keystroke chips.",
            ),
            press_rows,
        ).look(NodeLook::of("lucide:command").moves(NodeMotion::Press)),
        card(
            chain::<Wait>(
                BrowserAction::Wait,
                "Wait",
                "Waits for an element, a quiet page, an address, or a fixed time. Waiting for something is more reliable than a fixed time.",
            ),
            |p: &Wait| {
                vec![
                    Some(wait_row(p)),
                    (p.mode != WaitMode::Ms).then(|| give_up_row(p.timeout_ms)),
                ]
            },
        ).look(NodeLook::of("lucide:hourglass").moves(NodeMotion::TurnOver)),
        card(
            chain::<Assert>(
                BrowserAction::Assert,
                "Check text",
                "Stops the recording early if some text is not on the page, so you don't export a broken take.",
            ),
            |p: &Assert| {
                vec![
                    Some(quoted_row(
                        Icon::CircleCheck,
                        &p.text,
                        "Add the text to look for",
                    )),
                    Some(give_up_row(p.timeout_ms)),
                ]
            },
        ).look(NodeLook::of("lucide:spell-check")),
        card(
            chain::<Hoist>(
                BrowserAction::Hoist,
                "Find element early",
                "Looks an element up ahead of time so a later action doesn't pause on screen while searching.",
            ),
            |p: &Hoist| vec![Some(target_row(Icon::Crosshair, &p.target))],
        ).look(NodeLook::of("lucide:search-check")),
        card(
            chain::<SetState>(
                BrowserAction::SetState,
                "Run page script",
                "Runs a small script in the page, e.g. to skip onboarding or turn on a feature. Put it before Go to page to run it as the site first opens.",
            ),
            set_state_rows,
        ).look(NodeLook::of("lucide:braces").morphs("lucide:play")),
        card(
            chain::<MockRoute>(
                BrowserAction::MockRoute,
                "Fake a response",
                "Makes the site show fixed data for matching requests made after it, so every take looks the same. Put it before Go to page to cover the first load.",
            ),
            mock_route_rows,
        ).look(NodeLook::of("lucide:server-cog")),
        card(
            chain::<SwitchTheme>(
                BrowserAction::SwitchTheme,
                "Switch theme",
                "Switches the page between light and dark, as changing the computer's setting would, without reloading it.",
            ),
            |p: &SwitchTheme| {
                let icon = if p.color_scheme == ColorScheme::Dark { Icon::Moon } else { Icon::Sun };
                vec![Some(row(icon, p.color_scheme.label()))]
            },
        ).look(NodeLook::of("lucide:sun-moon")),
        card(
            chain::<ChangeLanguage>(
                BrowserAction::ChangeLanguage,
                "Change language",
                "Changes the language the page is told the computer uses, as for a visitor from another country.",
            ),
            |p: &ChangeLanguage| {
                let language = match p.locale.trim() {
                    "" => "This computer's language",
                    given => given,
                };
                vec![
                    Some(row(Icon::Languages, language)),
                    p.reload.then(|| row(Icon::RefreshCw, "Reloads the page")),
                ]
            },
        ).look(NodeLook::of("lucide:languages")),
        card(
            chain::<FreezeClock>(
                BrowserAction::FreezeClock,
                "Freeze time",
                "Pins the date and time the page sees, so clocks and \"x minutes ago\" never change between takes.",
            ),
            |p: &FreezeClock| {
                vec![Some(row(
                    Icon::Clock,
                    format!("{} · {}", format_when(&p.time), p.mode.label()),
                ))]
            },
        ).look(NodeLook::of("lucide:calendar-clock")),
        card(
            chain::<HideElements>(
                BrowserAction::HideElements,
                "Hide clutter",
                "Hides cookie banners, chat bubbles and anything else you list.",
            ),
            hide_rows,
        ).look(NodeLook::of("lucide:eye").morphs("lucide:eye-off")),
        card(
            chain::<Chapter>(
                BrowserAction::Chapter,
                "Chapter",
                "Starts a new chapter. Can show a chapter card in the video.",
            ),
            |p: &Chapter| {
                vec![
                    Some(row(Icon::Bookmark, quoted(&p.title))),
                    p.show_in_video.then(|| {
                        row(
                            Icon::Presentation,
                            format!("Card for {}", format_duration(f64::from(p.duration_ms))),
                        )
                    }),
                ]
            },
        ).look(NodeLook::of("lucide:bookmark")),
        card(
            chain::<OverlayHtml>(
                BrowserAction::OverlayHtml,
                "HTML overlay",
                "Shows your own HTML on top of the page while recording.",
            ),
            overlay_rows,
        ).look(NodeLook::of("lucide:file-code")),
    ]
}
