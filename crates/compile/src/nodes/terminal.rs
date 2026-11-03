use opsis_macros::{choice, settings};
use opsis_types::cards::{CardRow, Icon, Tone};
use opsis_types::looks::{NodeLook, NodeMotion};
use opsis_types::numbers;
use opsis_types::recording::{RecordingProps, ScreenProps};
use opsis_types::terminal::{
    CellTarget, EnvVar, LookIn, PointerButton, ShellChoice, TerminalAction, TerminalCursor,
    TerminalLook, TerminalSessionValue, TerminalStep, WaitFor,
};
use opsis_types::values::{CompileValue, RecordingValue, TakeKind, TakeSource, TerminalSource};
use opsis_types::workflow::{DEFAULT_TAKE_MS, PortType};

use crate::errors::SettingsError;
use crate::node::{
    CardCtx, Category, Ctx, NodeDef, Outputs, Phase, RECORDING_PORT, RowExt, missing, outputs,
    port, row,
};
use crate::params::Params;
use crate::terminal::{
    DEFAULT_FONT_SIZE, DEFAULT_LINE_HEIGHT, DEFAULT_PADDING, DEFAULT_THEME_NAME, default_look,
    grid, regrid_actions, regrid_log, theme_named,
};
use crate::text::{format_duration, plural, truncate};

pub const TERMINAL_PORT: &str = "terminal";
pub const OPEN_TERMINAL_TYPE: &str = "terminal.open";
pub const RECORD_TERMINAL_TYPE: &str = "capture.terminal";
pub const TERMINAL_LOOK_TYPE: &str = "terminal.look";
const TEXT_SHOWN: usize = 40;

fn session_port() -> crate::node::PortSpec {
    port(PortType::TerminalSession, "Terminal")
}

#[settings(eq)]
pub struct EnvRow {
    #[setting("Name")]
    pub name: String,
    #[setting("Value")]
    pub value: String,
}

#[settings(eq)]
pub struct OpenTerminal {
    #[setting(
        "Shell",
        default = ShellChoice::Clean,
        help = "Clean starts the computer's usual shell with Opsis's own prompt and nothing from your setup."
    )]
    pub shell: ShellChoice,
    #[setting(
        "Folder",
        zero_label = "The project folder",
        help = "Where the shell starts, inside the project folder."
    )]
    pub folder: String,
    #[setting("Columns", default = 80, range(min = 20, max = 300))]
    pub cols: u32,
    #[setting("Rows", default = 24, range(min = 5, max = 100))]
    pub rows: u32,
    #[setting(
        "Typing speed",
        default = 50,
        range(min = 0, max = 1000),
        step = 5,
        unit = "ms",
        help = "The time between two typed characters."
    )]
    pub typing_ms: u32,
    #[setting(
        "Pointer speed",
        default = 600,
        range(min = 0, max = 5000),
        step = 50,
        unit = "ms",
        help = "How long the pointer takes to move to what it clicks.",
        advanced
    )]
    pub pointer_ms: u32,
    #[setting("Environment variables", advanced)]
    pub env: Vec<EnvRow>,
}

#[settings(eq)]
pub struct WaitAfter {
    #[setting(
        "Wait after",
        default = 500,
        range(min = 0, max = 600_000),
        step = 50,
        unit = "ms",
        help = "How long the take lingers after this action."
    )]
    pub wait_after_ms: u32,
}

#[settings(check, eq)]
pub struct RunCommand {
    #[setting("Command", default = "echo hello")]
    pub command: String,
    #[setting(
        "Wait for",
        default = WaitFor::Prompt,
        help = "The prompt works with the clean shell. Quiet output waits until nothing is printed for a moment."
    )]
    pub wait: WaitFor,
    #[setting(
        "Time limit",
        default = 60000,
        range(min = 1000, max = 3_600_000),
        step = 1000,
        unit = "ms",
        advanced
    )]
    pub limit_ms: u32,
    #[setting(
        "Zoom to the output",
        default = true,
        help = "The camera moves to what the command printed."
    )]
    pub zoom_output: bool,
    #[serde(flatten)]
    pub after: WaitAfter,
}

impl Params for RunCommand {
    fn check(&self) -> Result<(), SettingsError> {
        if self.command.trim().is_empty() {
            return Err(SettingsError::rule("Type a command"));
        }
        Ok(())
    }
}

#[settings(eq)]
pub struct TypeText {
    #[setting("Text", widget = "textarea")]
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Typing speed",
        range(min = 0, max = 1000),
        unit = "ms",
        zero_label = "The terminal's",
        "type" = "integer"
    )]
    pub speed_ms: Option<u32>,
    #[serde(flatten)]
    pub after: WaitAfter,
}

#[settings(eq)]
pub struct PressTerminalKeys {
    #[setting(
        "Keys",
        default = "Enter",
        widget = "keys",
        help = "Click the field and press the shortcut."
    )]
    pub keys: String,
    #[setting("Repeat", default = 1, range(min = 1, max = 100))]
    pub repeat: u32,
    #[serde(flatten)]
    pub after: WaitAfter,
}

#[choice]
pub enum TargetKind {
    #[default]
    #[label("Text on the screen")]
    Text,
    #[label("A cell")]
    Cell,
}

#[settings(eq)]
pub struct TargetSetting {
    #[setting("Target", default = TargetKind::Text)]
    pub kind: TargetKind,
    #[setting("Text", show_if = { "kind": ["text"] }, help = "The first place this text shows on the screen.")]
    pub text: String,
    #[setting("Column", range(min = 0, max = 299), show_if = { "kind": ["cell"] })]
    pub col: u32,
    #[setting("Row", range(min = 0, max = 99), show_if = { "kind": ["cell"] })]
    pub row: u32,
}

impl TargetSetting {
    fn target(&self) -> CellTarget {
        match self.kind {
            TargetKind::Text => CellTarget::Text {
                text: self.text.clone(),
            },
            TargetKind::Cell => CellTarget::Cell {
                col: u16::try_from(self.col).unwrap_or(u16::MAX),
                row: u16::try_from(self.row).unwrap_or(u16::MAX),
            },
        }
    }

    fn row(&self, icon: Icon) -> CardRow {
        match self.kind {
            TargetKind::Text if self.text.trim().is_empty() => missing(icon, "Pick the text"),
            TargetKind::Text => row(icon, format!("“{}”", truncate(&self.text, TEXT_SHOWN))),
            TargetKind::Cell => row(icon, format!("Column {}, row {}", self.col, self.row)),
        }
    }
}

#[settings(eq)]
pub struct TerminalClick {
    #[serde(flatten)]
    pub target: TargetSetting,
    #[setting("Button", default = PointerButton::Left)]
    pub button: PointerButton,
    #[setting("Double-click")]
    pub double: bool,
    #[serde(flatten)]
    pub after: WaitAfter,
}

#[settings(eq)]
pub struct TerminalDrag {
    #[setting("From", default = TargetKind::Text)]
    pub from_kind: TargetKind,
    #[setting("From text", show_if = { "fromKind": ["text"] })]
    pub from_text: String,
    #[setting("From column", range(min = 0, max = 299), show_if = { "fromKind": ["cell"] })]
    pub from_col: u32,
    #[setting("From row", range(min = 0, max = 99), show_if = { "fromKind": ["cell"] })]
    pub from_row: u32,
    #[setting("To", default = TargetKind::Cell)]
    pub to_kind: TargetKind,
    #[setting("To text", show_if = { "toKind": ["text"] })]
    pub to_text: String,
    #[setting("To column", range(min = 0, max = 299), show_if = { "toKind": ["cell"] })]
    pub to_col: u32,
    #[setting("To row", range(min = 0, max = 99), show_if = { "toKind": ["cell"] })]
    pub to_row: u32,
    #[serde(flatten)]
    pub after: WaitAfter,
}

impl TerminalDrag {
    fn ends(&self) -> (TargetSetting, TargetSetting) {
        (
            TargetSetting {
                kind: self.from_kind,
                text: self.from_text.clone(),
                col: self.from_col,
                row: self.from_row,
            },
            TargetSetting {
                kind: self.to_kind,
                text: self.to_text.clone(),
                col: self.to_col,
                row: self.to_row,
            },
        )
    }
}

#[settings(check, eq)]
pub struct TerminalWheel {
    #[serde(flatten)]
    pub target: TargetSetting,
    #[setting(
        "Notches",
        default = 3,
        range(min = -50, max = 50),
        help = "Below zero turns the wheel up."
    )]
    pub notches: i32,
    #[serde(flatten)]
    pub after: WaitAfter,
}

impl Params for TerminalWheel {
    fn check(&self) -> Result<(), SettingsError> {
        if self.notches == 0 {
            return Err(SettingsError::rule("Turn the wheel at least one notch"));
        }
        Ok(())
    }
}

#[settings(check, eq)]
pub struct WaitText {
    #[setting(
        "Text",
        default = "done",
        help = "A pattern, such as Ready|Listening on [0-9]+"
    )]
    pub pattern: String,
    #[setting("Look in", default = LookIn::Screen)]
    pub scope: LookIn,
    #[setting(
        "Time limit",
        default = 10000,
        range(min = 100, max = 3_600_000),
        step = 100,
        unit = "ms"
    )]
    pub limit_ms: u32,
    #[serde(flatten)]
    pub after: WaitAfter,
}

impl Params for WaitText {
    fn check(&self) -> Result<(), SettingsError> {
        if self.pattern.trim().is_empty() {
            return Err(SettingsError::rule("Type the text to wait for"));
        }
        regex::Regex::new(&self.pattern)
            .map(|_| ())
            .map_err(|_| SettingsError::rule("This isn't a pattern Opsis can search for"))
    }
}

#[settings(eq)]
pub struct NoSettings {}

#[settings(eq)]
pub struct TerminalChapter {
    #[setting("Title", default = "Chapter")]
    pub title: String,
}

#[settings(eq)]
pub struct RecordTerminal {
    #[setting("Name", default = "Terminal take")]
    pub name: String,
    #[setting(
        "Pause before the first action",
        default = 500,
        range(min = 0, max = 600_000),
        step = 50,
        unit = "ms"
    )]
    pub pause_before_ms: u32,
    #[setting(
        "Keep recording after the last action",
        default = 1500,
        range(min = 0, max = 600_000),
        step = 50,
        unit = "ms",
        help = "Gives the camera time to settle at the end."
    )]
    pub keep_after_ms: u32,
}

#[settings(check)]
pub struct TerminalLookSettings {
    #[setting("Theme", default = DEFAULT_THEME_NAME.to_owned(), help = "A theme's name, such as Dracula or Catppuccin Mocha.")]
    pub theme: String,
    #[setting("Font size", default = DEFAULT_FONT_SIZE, range(min = 8, max = 64), unit = "px")]
    pub font_size: u32,
    #[setting(
        "Line height",
        default = DEFAULT_LINE_HEIGHT,
        range(min = 1, max = 2),
        step = 0.05,
        unit = "×"
    )]
    pub line_height: f64,
    #[setting(
        "Letter spacing",
        range(min = -2, max = 8),
        step = 0.5,
        unit = "px",
        advanced
    )]
    pub letter_spacing: f64,
    #[setting("Padding", default = DEFAULT_PADDING, range(min = 0, max = 200), unit = "px")]
    pub padding: u32,
    #[setting("Cursor shape", default = TerminalCursor::Program)]
    pub cursor: TerminalCursor,
    #[setting("Cursor blinks", default = true)]
    pub cursor_blink: bool,
}

impl Params for TerminalLookSettings {
    fn check(&self) -> Result<(), SettingsError> {
        if theme_named(&self.theme).is_none() {
            return Err(SettingsError::rule("There's no theme with this name"));
        }
        Ok(())
    }
}

impl TerminalLookSettings {
    pub fn look(&self) -> TerminalLook {
        TerminalLook {
            theme: theme_named(&self.theme).unwrap_or_else(|| default_look(None).theme),
            font_size: self.font_size,
            line_height: self.line_height,
            letter_spacing: self.letter_spacing,
            padding: self.padding,
            cursor: self.cursor,
            cursor_blink: self.cursor_blink,
        }
    }
}

pub fn with_look(mut props: RecordingProps, look: TerminalLook) -> RecordingProps {
    let ScreenProps::Terminal {
        cols,
        rows,
        look: current,
        ..
    } = &mut props.screen
    else {
        return props;
    };
    let to = grid(&look, *cols, *rows);
    let from = props.log.cells;
    **current = look;
    regrid_log(&mut props.log, to);
    if let Some(from) = from {
        regrid_actions(&mut props.extra_beats, from, to);
    }
    props
}

fn append(ctx: &Ctx<'_, impl Sized>, action: TerminalAction, wait_after_ms: u32) -> Outputs {
    let Some(CompileValue::TerminalSession(session)) = ctx.input(TERMINAL_PORT) else {
        return Outputs::new();
    };
    let mut session = session.clone();
    session.steps.push(TerminalStep {
        id: ctx.node_id.to_owned(),
        node_id: ctx.node_id.to_owned(),
        label: ctx.label.map_or_else(|| ctx.kind.to_owned(), str::to_owned),
        action,
        wait_after_ms,
    });
    outputs([(TERMINAL_PORT, CompileValue::TerminalSession(session))])
}

fn chain<P: Params>(kind: &'static str, title: &str, description: &str) -> NodeDef {
    NodeDef::of::<P>(kind, title, Category::Terminal, Phase::Capture)
        .describe(description)
        .virtual_node()
        .inputs(vec![(TERMINAL_PORT, session_port())])
        .outputs(vec![(TERMINAL_PORT, session_port())])
}

fn after_row(ms: u32) -> Option<CardRow> {
    (ms > 0).then(|| {
        row(
            Icon::Timer,
            format!("Wait {}", format_duration(f64::from(ms))),
        )
    })
}

fn open_terminal() -> NodeDef {
    NodeDef::of::<OpenTerminal>(
        OPEN_TERMINAL_TYPE,
        "Open terminal",
        Category::Terminal,
        Phase::Capture,
    )
    .describe("Starts a terminal for the actions that follow: its shell, folder and size.")
    .virtual_node()
    .outputs(vec![(TERMINAL_PORT, session_port())])
    .card(|ctx: &CardCtx<'_, OpenTerminal>| {
        let p = &ctx.params;
        vec![
            Some(row(Icon::Monitor, format!("{} × {}", p.cols, p.rows))),
            Some(row(Icon::Code, p.shell.label())),
            (!p.folder.trim().is_empty()).then(|| row(Icon::File, p.folder.trim().to_owned())),
        ]
    })
    .compiles(|ctx: &Ctx<'_, OpenTerminal>| {
        let p = &ctx.params;
        outputs([(
            TERMINAL_PORT,
            CompileValue::TerminalSession(TerminalSessionValue {
                shell: p.shell,
                folder: p.folder.trim().to_owned(),
                cols: u16::try_from(p.cols).unwrap_or(u16::MAX),
                rows: u16::try_from(p.rows).unwrap_or(u16::MAX),
                env: p
                    .env
                    .iter()
                    .map(|e| EnvVar {
                        name: e.name.clone(),
                        value: e.value.clone(),
                    })
                    .collect(),
                typing_ms: p.typing_ms,
                pointer_ms: p.pointer_ms,
                steps: Vec::new(),
            }),
        )])
    })
}

fn run_command() -> NodeDef {
    chain::<RunCommand>(
        "terminal.run",
        "Run command",
        "Types a command, presses Enter and waits for it to finish.",
    )
    .action()
    .card(|ctx: &CardCtx<'_, RunCommand>| {
        let p = &ctx.params;
        vec![
            Some(if p.command.trim().is_empty() {
                missing(Icon::Code, "Type a command")
            } else {
                row(Icon::Code, truncate(&p.command, TEXT_SHOWN))
            }),
            (p.wait == WaitFor::Quiet)
                .then(|| row(Icon::Hourglass, "Waits for quiet output").tone(Tone::Muted)),
            (!p.zoom_output).then(|| row(Icon::ZoomOut, "No zoom to the output").tone(Tone::Muted)),
            after_row(p.after.wait_after_ms),
        ]
    })
    .compiles(|ctx: &Ctx<'_, RunCommand>| {
        let p = &ctx.params;
        append(
            ctx,
            TerminalAction::Run {
                command: p.command.clone(),
                wait: p.wait,
                limit_ms: p.limit_ms,
                zoom_output: p.zoom_output,
            },
            p.after.wait_after_ms,
        )
    })
}

fn type_text() -> NodeDef {
    chain::<TypeText>(
        "terminal.type",
        "Type",
        "Types text into the terminal, one character at a time.",
    )
    .action()
    .card(|ctx: &CardCtx<'_, TypeText>| {
        let p = &ctx.params;
        vec![
            Some(if p.text.is_empty() {
                missing(Icon::Keyboard, "Type the text")
            } else {
                row(Icon::Keyboard, truncate(&p.text, TEXT_SHOWN))
            }),
            after_row(p.after.wait_after_ms),
        ]
    })
    .compiles(|ctx: &Ctx<'_, TypeText>| {
        let p = &ctx.params;
        append(
            ctx,
            TerminalAction::Type {
                text: p.text.clone(),
                speed_ms: p.speed_ms,
            },
            p.after.wait_after_ms,
        )
    })
}

fn press_keys() -> NodeDef {
    chain::<PressTerminalKeys>(
        "terminal.keys",
        "Press keys",
        "Presses a key or a shortcut in the terminal.",
    )
    .action()
    .card(|ctx: &CardCtx<'_, PressTerminalKeys>| {
        let p = &ctx.params;
        let keys = crate::nodes::browser::chord_keys(&p.keys).join(" + ");
        vec![
            Some(if p.repeat > 1 {
                row(Icon::Command, format!("{keys} × {}", p.repeat))
            } else {
                row(Icon::Command, keys)
            }),
            p.keys.split('+').any(|k| k.trim() == "Meta").then(|| {
                row(Icon::TriangleAlert, "A terminal can't receive the Meta key").tone(Tone::Warn)
            }),
            after_row(p.after.wait_after_ms),
        ]
    })
    .compiles(|ctx: &Ctx<'_, PressTerminalKeys>| {
        let p = &ctx.params;
        append(
            ctx,
            TerminalAction::Keys {
                keys: p.keys.clone(),
                repeat: p.repeat,
            },
            p.after.wait_after_ms,
        )
    })
}

fn click() -> NodeDef {
    chain::<TerminalClick>(
        "terminal.click",
        "Click",
        "Clicks text or a cell, for programs that read the mouse.",
    )
    .action()
    .card(|ctx: &CardCtx<'_, TerminalClick>| {
        let p = &ctx.params;
        vec![
            Some(p.target.row(Icon::MousePointerClick)),
            (p.button != PointerButton::Left || p.double).then(|| {
                row(
                    Icon::MousePointer,
                    if p.double {
                        format!("{} double-click", p.button.label())
                    } else {
                        format!("{} button", p.button.label())
                    },
                )
            }),
            after_row(p.after.wait_after_ms),
        ]
    })
    .compiles(|ctx: &Ctx<'_, TerminalClick>| {
        let p = &ctx.params;
        append(
            ctx,
            TerminalAction::Click {
                target: p.target.target(),
                button: p.button,
                double: p.double,
            },
            p.after.wait_after_ms,
        )
    })
}

fn drag() -> NodeDef {
    chain::<TerminalDrag>(
        "terminal.drag",
        "Drag",
        "Presses on one place, moves to another and lets go, for programs that read the mouse.",
    )
    .action()
    .card(|ctx: &CardCtx<'_, TerminalDrag>| {
        let (from, to) = ctx.params.ends();
        vec![
            Some(from.row(Icon::Grab)),
            Some(to.row(Icon::MoveUpRight)),
            after_row(ctx.params.after.wait_after_ms),
        ]
    })
    .compiles(|ctx: &Ctx<'_, TerminalDrag>| {
        let (from, to) = ctx.params.ends();
        append(
            ctx,
            TerminalAction::Drag {
                from: from.target(),
                to: to.target(),
            },
            ctx.params.after.wait_after_ms,
        )
    })
}

fn wheel() -> NodeDef {
    chain::<TerminalWheel>(
        "terminal.wheel",
        "Mouse wheel",
        "Turns the mouse wheel over text or a cell, for programs that read the mouse.",
    )
    .action()
    .card(|ctx: &CardCtx<'_, TerminalWheel>| {
        let p = &ctx.params;
        let way = if p.notches < 0 { "up" } else { "down" };
        vec![
            Some(p.target.row(Icon::Scroll)),
            Some(row(
                Icon::Move,
                format!(
                    "{} {way}",
                    plural(f64::from(p.notches.unsigned_abs()), "notch")
                ),
            )),
            after_row(p.after.wait_after_ms),
        ]
    })
    .compiles(|ctx: &Ctx<'_, TerminalWheel>| {
        let p = &ctx.params;
        append(
            ctx,
            TerminalAction::Wheel {
                target: p.target.target(),
                notches: p.notches,
            },
            p.after.wait_after_ms,
        )
    })
}

fn wait_text() -> NodeDef {
    chain::<WaitText>(
        "terminal.waitText",
        "Wait for text",
        "Waits until some text shows in the terminal.",
    )
    .action()
    .card(|ctx: &CardCtx<'_, WaitText>| {
        let p = &ctx.params;
        vec![
            Some(row(
                Icon::Hourglass,
                format!("“{}”", truncate(&p.pattern, TEXT_SHOWN)),
            )),
            Some(
                row(
                    Icon::TimerOff,
                    format!("Give up after {}", format_duration(f64::from(p.limit_ms))),
                )
                .tone(Tone::Muted),
            ),
            after_row(p.after.wait_after_ms),
        ]
    })
    .compiles(|ctx: &Ctx<'_, WaitText>| {
        let p = &ctx.params;
        append(
            ctx,
            TerminalAction::WaitText {
                pattern: p.pattern.clone(),
                scope: p.scope,
                limit_ms: p.limit_ms,
            },
            p.after.wait_after_ms,
        )
    })
}

fn start_recording() -> NodeDef {
    chain::<NoSettings>(
        "terminal.startRecording",
        "Start recording",
        "The film starts here. Actions before it set things up off camera.",
    )
    .card(|_: &CardCtx<'_, NoSettings>| vec![Some(row(Icon::Clapperboard, "The film starts here"))])
    .compiles(|ctx: &Ctx<'_, NoSettings>| append(ctx, TerminalAction::StartRecording, 0))
}

fn chapter() -> NodeDef {
    chain::<TerminalChapter>(
        "terminal.chapter",
        "Chapter",
        "Starts a chapter of the film here, listed in exports.",
    )
    .action()
    .card(|ctx: &CardCtx<'_, TerminalChapter>| {
        vec![Some(row(Icon::Bookmark, ctx.params.title.clone()))]
    })
    .compiles(|ctx: &Ctx<'_, TerminalChapter>| {
        append(
            ctx,
            TerminalAction::Chapter {
                title: ctx.params.title.clone(),
            },
            0,
        )
    })
}

fn clear_screen() -> NodeDef {
    chain::<WaitAfter>(
        "terminal.clear",
        "Clear screen",
        "Clears the terminal screen.",
    )
    .card(|ctx: &CardCtx<'_, WaitAfter>| vec![after_row(ctx.params.wait_after_ms)])
    .compiles(|ctx: &Ctx<'_, WaitAfter>| {
        append(ctx, TerminalAction::Clear, ctx.params.wait_after_ms)
    })
}

fn recording_of(result: Option<&opsis_types::values::NodeResult>) -> Option<&RecordingValue> {
    result
        .and_then(|r| r.output(RECORDING_PORT))
        .and_then(CompileValue::as_recording)
        .filter(|r| r.kind() == TakeKind::Terminal)
}

fn record_terminal() -> NodeDef {
    NodeDef::of::<RecordTerminal>(
        RECORD_TERMINAL_TYPE,
        "Record terminal take",
        Category::Capture,
        Phase::Capture,
    )
    .describe("Plays all connected terminal actions in one go and records the terminal.")
    .executes()
    .inputs(vec![(TERMINAL_PORT, session_port())])
    .outputs(vec![(
        RECORDING_PORT,
        port(PortType::Recording, "Recording"),
    )])
    .card(|ctx: &CardCtx<'_, RecordTerminal>| {
        let steps = ctx.inputs.terminal_steps.get(TERMINAL_PORT).copied();
        let rec = recording_of(ctx.result);
        vec![
            Some(steps.map_or_else(
                || missing(Icon::ListChecks, "Connect the terminal actions"),
                |n| row(Icon::ListChecks, plural(numbers::float(n), "action")).muted_if(n == 0),
            )),
            rec.map(|r| row(Icon::Clock, format_duration(r.duration_ms))),
            rec.and_then(|r| match &r.source {
                TakeSource::Terminal(t) => {
                    Some(row(Icon::Monitor, format!("{} × {}", t.cols, t.rows)))
                }
                TakeSource::Browser(_) => None,
            }),
        ]
    })
    .compiles(|ctx: &Ctx<'_, RecordTerminal>| {
        if let Some(rec) = recording_of(ctx.result) {
            return outputs([(RECORDING_PORT, CompileValue::Recording(rec.clone()))]);
        }
        let (cols, rows) = match ctx.input(TERMINAL_PORT) {
            Some(CompileValue::TerminalSession(s)) => (s.cols, s.rows),
            _ => (80, 24),
        };
        outputs([(
            RECORDING_PORT,
            CompileValue::Recording(RecordingValue {
                log: None,
                duration_ms: DEFAULT_TAKE_MS,
                node_id: ctx.node_id.to_owned(),
                edits: Vec::new(),
                source: TakeSource::Terminal(TerminalSource {
                    cast: None,
                    cols,
                    rows,
                    theme: None,
                }),
            }),
        )])
    })
}

fn terminal_look() -> NodeDef {
    NodeDef::style_modifier::<TerminalLookSettings>(
        TERMINAL_LOOK_TYPE,
        "Terminal look",
        Category::Terminal,
    )
    .describe("The look of a terminal take: its colours, type size, spacing and cursor.")
    .only_for(TakeKind::Terminal)
    .card(|ctx: &CardCtx<'_, TerminalLookSettings>| {
        let p = &ctx.params;
        let look = p.look();
        vec![
            Some(
                row(Icon::Palette, p.theme.clone())
                    .swatches(vec![look.theme.bg.clone(), look.theme.fg]),
            ),
            Some(row(
                Icon::Ruler,
                format!("{} px · {}× lines", p.font_size, p.line_height),
            )),
        ]
    })
    .restyle(|props: RecordingProps, p: &TerminalLookSettings, _| with_look(props, p.look()))
}

pub fn nodes() -> Vec<NodeDef> {
    vec![
        open_terminal().look(NodeLook::of("iconoir:terminal")),
        run_command().look(NodeLook::of("lucide:terminal").moves(NodeMotion::NudgeRight)),
        type_text().look(NodeLook::of("tabler:cursor-text")),
        press_keys().look(NodeLook::of("tabler:keyboard")),
        click().look(NodeLook::of("tabler:click")),
        drag().look(NodeLook::of("tabler:drag-drop")),
        wheel().look(NodeLook::of("lucide:mouse")),
        wait_text().look(NodeLook::of("tabler:hourglass")),
        start_recording().look(NodeLook::of("tabler:player-record")),
        chapter().look(NodeLook::of("tabler:bookmark")),
        clear_screen().look(NodeLook::of("tabler:clear-all")),
        record_terminal().look(NodeLook::of("tabler:terminal-2")),
        terminal_look().look(NodeLook::of("tabler:palette")),
    ]
}
