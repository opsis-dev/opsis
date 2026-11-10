use opsis_compile::nodes::camera::{
    FRAME_TYPE, Frame, FrameChoice, SPEED_RAMP_TYPE, SpeedRamp, SpeedSegment,
};
use opsis_compile::nodes::terminal::{OpenTerminal, TERMINAL_LOOK_TYPE, TerminalLookSettings};
use opsis_compile::terminal::{default_theme, theme_named};
use opsis_types::numbers::{truncate, whole};
use opsis_types::recording::FrameTheme;
use opsis_types::terminal::{
    EnvVar, LookIn, ShellChoice, TerminalAction, TerminalSessionValue, TerminalStep, TerminalTheme,
    WaitFor,
};
use opsis_types::values::StyleEntry;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use super::model::{Command, Key, Tape, TapeError};
use super::parser::parse;
use super::writer::line;

pub const WAIT_TIMEOUT_MS: u32 = 15_000;
pub const WAIT_PATTERN: &str = ">$";
const QUIET_SLEEP_MS: u32 = 1000;
const VHS_WIDTH: f64 = 1200.0;
const VHS_HEIGHT: f64 = 600.0;
const VHS_PADDING: f64 = 60.0;
const VHS_FONT_SIZE: f64 = 22.0;
const VHS_LETTER_SPACING: f64 = 1.0;
const VHS_LINE_HEIGHT: f64 = 1.0;
const VHS_BAR: f64 = 30.0;
const CELL_WIDTH_EM: f64 = 0.6;
const LABEL_LENGTH: usize = 48;

#[derive(Debug, Clone, PartialEq)]
pub struct TapeSteps {
    pub session: TerminalSessionValue,
    pub theme: Option<TerminalTheme>,
    pub style: Vec<StyleEntry>,
    pub speed: Option<f64>,
    pub ignored: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Default)]
struct Size {
    width: Option<f64>,
    height: Option<f64>,
    cols: Option<u16>,
    rows: Option<u16>,
    font: Option<f64>,
    letter_spacing: Option<f64>,
    line_height: Option<f64>,
    padding: Option<f64>,
    margin: Option<f64>,
    bar: bool,
}

impl Size {
    fn grid(&self) -> (u16, u16) {
        let font = self.font.unwrap_or(VHS_FONT_SIZE);
        let cell_w = font.mul_add(
            CELL_WIDTH_EM,
            self.letter_spacing.unwrap_or(VHS_LETTER_SPACING),
        );
        let cell_h = (font * self.line_height.unwrap_or(VHS_LINE_HEIGHT)).max(1.0);
        let edge = 2.0 * (self.padding.unwrap_or(VHS_PADDING) + self.margin.unwrap_or(0.0));
        let bar = if self.bar { VHS_BAR } else { 0.0 };
        let fit =
            |space: f64, cell: f64| -> u16 { truncate((space / cell.max(1.0)).floor().max(1.0)) };
        let cols = self
            .cols
            .unwrap_or_else(|| fit(self.width.unwrap_or(VHS_WIDTH) - edge, cell_w));
        let rows = self
            .rows
            .unwrap_or_else(|| fit(self.height.unwrap_or(VHS_HEIGHT) - edge - bar, cell_h));
        (cols.clamp(20, 300), rows.clamp(5, 100))
    }
}

#[derive(Debug)]
struct Mapping {
    steps: Vec<TerminalStep>,
    session: TerminalSessionValue,
    size: Size,
    look: Option<TerminalLookSettings>,
    theme: Option<TerminalTheme>,
    window_bar: bool,
    radius: Option<u32>,
    speed: Option<f64>,
    typing_ms: Option<u32>,
    wait_ms: u32,
    wait_pattern: Option<String>,
    ignored: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JsonTheme {
    #[serde(default)]
    name: String,
    background: Option<String>,
    foreground: Option<String>,
    cursor: Option<String>,
    black: Option<String>,
    red: Option<String>,
    green: Option<String>,
    yellow: Option<String>,
    blue: Option<String>,
    magenta: Option<String>,
    cyan: Option<String>,
    white: Option<String>,
    bright_black: Option<String>,
    bright_red: Option<String>,
    bright_green: Option<String>,
    bright_yellow: Option<String>,
    bright_blue: Option<String>,
    bright_magenta: Option<String>,
    bright_cyan: Option<String>,
    bright_white: Option<String>,
}

impl JsonTheme {
    fn theme(self) -> TerminalTheme {
        let base = default_theme();
        let pick =
            |value: Option<String>, fallback: &String| value.unwrap_or_else(|| fallback.clone());
        let ansi = [
            self.black,
            self.red,
            self.green,
            self.yellow,
            self.blue,
            self.magenta,
            self.cyan,
            self.white,
            self.bright_black,
            self.bright_red,
            self.bright_green,
            self.bright_yellow,
            self.bright_blue,
            self.bright_magenta,
            self.bright_cyan,
            self.bright_white,
        ];
        let mut palette = base.palette.clone();
        for (slot, colour) in palette.iter_mut().zip(ansi) {
            if let Some(colour) = colour {
                *slot = colour;
            }
        }
        let fg = pick(self.foreground, &base.fg);
        TerminalTheme {
            name: if self.name.is_empty() {
                "Tape theme".to_owned()
            } else {
                self.name
            },
            bg: pick(self.background, &base.bg),
            cursor: pick(self.cursor, &fg),
            fg,
            palette,
        }
    }
}

fn params(value: &impl Serialize) -> Map<String, Value> {
    match serde_json::to_value(value) {
        Ok(Value::Object(map)) => map,
        _ => Map::new(),
    }
}

fn entry(node_id: &str, kind: &str, value: &impl Serialize) -> StyleEntry {
    StyleEntry {
        node_id: node_id.to_owned(),
        kind: kind.to_owned(),
        params: params(value),
    }
}

fn is_dark(colour: &str) -> bool {
    let hex = colour.trim_start_matches('#');
    let channel = |at: usize| {
        hex.get(at..at + 2)
            .and_then(|c| u8::from_str_radix(c, 16).ok())
            .map_or(0.0, f64::from)
    };
    if hex.len() < 6 {
        return true;
    }
    let luma = 0.0722f64.mul_add(
        channel(4),
        0.2126f64.mul_add(channel(0), 0.7152 * channel(2)),
    );
    luma < 128.0
}

fn number(value: &str) -> Option<f64> {
    value.trim_end_matches("px").parse::<f64>().ok()
}

fn millis(value: &str) -> Option<u32> {
    value
        .strip_suffix("ms")
        .and_then(|n| n.parse::<f64>().ok())
        .map(whole)
}

fn label(command: &Command) -> String {
    let text = line(command).join(" ");
    if text.chars().count() <= LABEL_LENGTH {
        return text;
    }
    let cut: String = text.chars().take(LABEL_LENGTH - 1).collect();
    format!("{cut}…")
}

impl Mapping {
    fn look(&mut self) -> &mut TerminalLookSettings {
        self.look.get_or_insert_with(TerminalLookSettings::default)
    }

    fn set(&mut self, name: &str, value: &str) {
        match name {
            "Shell" => self.session.shell = ShellChoice::Own,
            "Columns" => self.size.cols = value.parse().ok(),
            "Rows" => self.size.rows = value.parse().ok(),
            "Width" => self.size.width = number(value),
            "Height" => self.size.height = number(value),
            "TypingSpeed" => {
                let ms = millis(value);
                if self.steps.is_empty() {
                    self.session.typing_ms = ms.unwrap_or(self.session.typing_ms);
                } else {
                    self.typing_ms = ms;
                }
            }
            "WaitTimeout" => self.wait_ms = millis(value).unwrap_or(WAIT_TIMEOUT_MS),
            "WaitPattern" => self.wait_pattern = Some(value.to_owned()),
            "Theme" => self.theme(value),
            "FontSize" => {
                self.size.font = number(value);
                if let Some(size) = number(value) {
                    self.look().font_size = whole(size);
                }
            }
            "LineHeight" => {
                self.size.line_height = number(value);
                if let Some(height) = number(value) {
                    self.look().line_height = height;
                }
            }
            "LetterSpacing" => {
                self.size.letter_spacing = number(value);
                if let Some(spacing) = number(value) {
                    self.look().letter_spacing = spacing;
                }
            }
            "Padding" => {
                self.size.padding = number(value);
                if let Some(padding) = number(value) {
                    self.look().padding = whole(padding);
                }
            }
            "CursorBlink" => self.look().cursor_blink = value != "false",
            "WindowBar" => {
                self.window_bar = !value.is_empty();
                self.size.bar = self.window_bar;
            }
            "BorderRadius" => self.radius = number(value).map(whole),
            "Margin" => self.size.margin = number(value),
            "PlaybackSpeed" => self.speed = number(value).filter(|s| *s > 0.0),
            other => self.ignored.push(format!("Set {other}")),
        }
    }

    fn theme(&mut self, value: &str) {
        if value.trim_start().starts_with('{') {
            match serde_json::from_str::<JsonTheme>(value) {
                Ok(theme) => self.theme = Some(theme.theme()),
                Err(error) => self
                    .warnings
                    .push(format!("The tape's theme couldn't be read: {error}")),
            }
            return;
        }
        if theme_named(value).is_some() {
            value.clone_into(&mut self.look().theme);
        } else {
            self.warnings.push(format!(
                "There's no theme called “{value}”, so the usual colours are used"
            ));
        }
    }

    fn push(
        &mut self,
        node_id: &str,
        command: &Command,
        action: TerminalAction,
        wait_after_ms: u32,
    ) {
        let index = self.steps.len();
        self.steps.push(TerminalStep {
            id: format!("{node_id}:{index}"),
            node_id: node_id.to_owned(),
            label: label(command),
            action,
            wait_after_ms,
        });
    }

    fn command(&mut self, node_id: &str, command: &Command) {
        let action = match command {
            Command::Type { speed_ms, text } => TerminalAction::Type {
                text: text.clone(),
                speed_ms: speed_ms.or(self.typing_ms),
            },
            Command::Key {
                key: Key::ScrollUp,
                repeat,
                ..
            } => TerminalAction::Scroll {
                lines: -i32::try_from(*repeat).unwrap_or(i32::MAX),
            },
            Command::Key {
                key: Key::ScrollDown,
                repeat,
                ..
            } => TerminalAction::Scroll {
                lines: i32::try_from(*repeat).unwrap_or(i32::MAX),
            },
            Command::Key {
                key,
                speed_ms,
                repeat,
            } => {
                let action = TerminalAction::Keys {
                    keys: key.chord().to_owned(),
                    repeat: *repeat,
                };
                self.push(node_id, command, action, speed_ms.unwrap_or(0));
                return;
            }
            Command::Ctrl { keys } => TerminalAction::Keys {
                keys: format!("Control+{}", keys.join("+")),
                repeat: 1,
            },
            Command::Alt { key } => TerminalAction::Keys {
                keys: format!("Alt+{key}"),
                repeat: 1,
            },
            Command::Shift { key } => TerminalAction::Keys {
                keys: format!("Shift+{key}"),
                repeat: 1,
            },
            Command::Sleep { ms } => TerminalAction::Sleep { ms: *ms },
            Command::Hide => TerminalAction::Hide,
            Command::Show => TerminalAction::Show,
            Command::Wait {
                scope,
                timeout_ms,
                pattern,
            } => {
                let limit_ms = timeout_ms.unwrap_or(self.wait_ms);
                match pattern.clone().or_else(|| self.wait_pattern.clone()) {
                    Some(pattern) => TerminalAction::WaitText {
                        pattern,
                        scope: *scope,
                        limit_ms,
                    },
                    None => TerminalAction::WaitPrompt { limit_ms },
                }
            }
            Command::Require { program } => TerminalAction::Require {
                program: program.clone(),
            },
            Command::Screenshot { .. } => TerminalAction::Screenshot,
            Command::Copy { text } => TerminalAction::Copy { text: text.clone() },
            Command::Paste => TerminalAction::Paste,
            Command::Env { name, value } => TerminalAction::Env {
                name: name.clone(),
                value: value.clone(),
            },
            Command::Set { name, value } => {
                self.set(name, value);
                return;
            }
            Command::Output { .. } => {
                self.ignored.push("Output".to_owned());
                return;
            }
            Command::Source { .. } => {
                self.ignored.push("Source".to_owned());
                return;
            }
        };
        self.push(node_id, command, action, 0);
    }

    fn style(&self, node_id: &str) -> Vec<StyleEntry> {
        let mut style = Vec::new();
        if let Some(look) = &self.look {
            style.push(entry(node_id, TERMINAL_LOOK_TYPE, look));
        }
        let margin = self.size.margin.filter(|m| *m > 0.0);
        if self.window_bar || self.radius.is_some() || margin.is_some() {
            let bg = self
                .look
                .as_ref()
                .and_then(|l| theme_named(&l.theme))
                .or_else(|| self.theme.clone())
                .unwrap_or_else(default_theme)
                .bg;
            let width = self.size.width.unwrap_or(VHS_WIDTH);
            let frame = Frame {
                window_style: if self.window_bar {
                    FrameChoice::Macos
                } else {
                    FrameChoice::None
                },
                frame_theme: if is_dark(&bg) {
                    FrameTheme::Dark
                } else {
                    FrameTheme::Light
                },
                title_bar: false,
                corner_radius: self.radius,
                window_size: margin
                    .map(|m| whole((2.0f64.mul_add(-m, width) / width * 100.0).clamp(40.0, 100.0))),
                ..Frame::default()
            };
            style.push(entry(node_id, FRAME_TYPE, &frame));
        }
        style
    }
}

pub fn tape_steps(tape: &Tape, node_id: &str) -> TapeSteps {
    let mut map = Mapping {
        steps: Vec::new(),
        session: OpenTerminal::default().session(),
        size: Size::default(),
        look: None,
        theme: None,
        window_bar: false,
        radius: None,
        speed: None,
        typing_ms: None,
        wait_ms: WAIT_TIMEOUT_MS,
        wait_pattern: None,
        ignored: Vec::new(),
        warnings: Vec::new(),
    };
    for command in &tape.commands {
        map.command(node_id, command);
    }
    let (cols, rows) = map.size.grid();
    map.session.cols = cols;
    map.session.rows = rows;
    map.session.steps = std::mem::take(&mut map.steps);
    map.ignored.dedup();
    TapeSteps {
        style: map.style(node_id),
        session: map.session,
        theme: map.theme,
        speed: map.speed,
        ignored: map.ignored,
        warnings: map.warnings,
    }
}

pub fn speed_entry(node_id: &str, rate: f64, duration_ms: u32) -> StyleEntry {
    let ramp = SpeedRamp {
        segments: vec![SpeedSegment {
            from_ms: 0,
            to_ms: duration_ms,
            rate: rate.clamp(0.1, 64.0),
        }],
        ease_ms: 0,
    };
    entry(node_id, SPEED_RAMP_TYPE, &ramp)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sourced {
    pub tape: Tape,
    pub files: Vec<String>,
}

pub fn parse_with_sources(
    text: &str,
    read: &dyn Fn(&str) -> Result<String, String>,
) -> Result<Sourced, Vec<TapeError>> {
    let tape = parse(text)?;
    let mut commands = Vec::new();
    let mut files = Vec::new();
    for command in tape.commands {
        let Command::Source { path } = &command else {
            commands.push(command);
            continue;
        };
        let other = read(path).map_err(|message| vec![source_error(path, &message)])?;
        let inner = parse(&other).map_err(|errors| {
            errors
                .into_iter()
                .map(|e| source_error(path, &e.to_string()))
                .collect::<Vec<_>>()
        })?;
        if inner
            .commands
            .iter()
            .any(|c| matches!(c, Command::Source { .. }))
        {
            return Err(vec![source_error(
                path,
                "A tape it plays plays another tape",
            )]);
        }
        files.push(path.clone());
        commands.extend(
            inner
                .commands
                .into_iter()
                .filter(|c| !matches!(c, Command::Output { .. })),
        );
    }
    Ok(Sourced {
        tape: Tape { commands },
        files,
    })
}

fn source_error(path: &str, message: &str) -> TapeError {
    TapeError {
        line: 0,
        column: 0,
        message: format!("{path}: {message}"),
    }
}

fn chord_command(keys: &str) -> Option<Command> {
    let parts: Vec<&str> = keys.split('+').collect();
    let (last, held) = parts.split_last()?;
    match held {
        [] => {
            let key = Key::ALL.into_iter().find(|k| k.chord() == *last)?;
            Some(Command::Key {
                key,
                speed_ms: None,
                repeat: 1,
            })
        }
        ["Control" | "Ctrl", rest @ ..] => Some(Command::Ctrl {
            keys: rest
                .iter()
                .chain(std::iter::once(last))
                .map(|s| (*s).to_owned())
                .collect(),
        }),
        ["Alt"] => Some(Command::Alt {
            key: (*last).to_owned(),
        }),
        ["Shift"] => Some(Command::Shift {
            key: (*last).to_owned(),
        }),
        _ => None,
    }
}

fn repeated(command: Command, repeat: u32) -> Vec<Command> {
    match command {
        Command::Key { key, speed_ms, .. } => vec![Command::Key {
            key,
            speed_ms,
            repeat,
        }],
        other => std::iter::repeat_n(other, usize::try_from(repeat).unwrap_or(1)).collect(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WrittenTape {
    pub tape: Tape,
    pub skipped: Vec<String>,
}

pub fn session_tape(session: &TerminalSessionValue) -> WrittenTape {
    let mut commands = vec![
        Command::Set {
            name: "Columns".to_owned(),
            value: session.cols.to_string(),
        },
        Command::Set {
            name: "Rows".to_owned(),
            value: session.rows.to_string(),
        },
        Command::Set {
            name: "TypingSpeed".to_owned(),
            value: format!("{}ms", session.typing_ms),
        },
    ];
    for EnvVar { name, value } in &session.env {
        commands.push(Command::Env {
            name: name.clone(),
            value: value.clone(),
        });
    }
    let starts = session
        .steps
        .iter()
        .any(|s| s.action == TerminalAction::StartRecording);
    if starts {
        commands.push(Command::Hide);
    }
    let mut skipped = Vec::new();
    for step in &session.steps {
        commands.extend(step_commands(step, &mut skipped));
        if step.wait_after_ms > 0 {
            commands.push(Command::Sleep {
                ms: step.wait_after_ms,
            });
        }
    }
    skipped.dedup();
    WrittenTape {
        tape: Tape { commands },
        skipped,
    }
}

fn step_commands(step: &TerminalStep, skipped: &mut Vec<String>) -> Vec<Command> {
    match &step.action {
        TerminalAction::Run {
            command,
            wait,
            limit_ms,
            ..
        } => {
            let finish = match wait {
                WaitFor::Prompt => Command::Wait {
                    scope: LookIn::Line,
                    timeout_ms: (*limit_ms != WAIT_TIMEOUT_MS).then_some(*limit_ms),
                    pattern: None,
                },
                WaitFor::Quiet => Command::Sleep { ms: QUIET_SLEEP_MS },
            };
            vec![
                Command::Type {
                    speed_ms: None,
                    text: command.clone(),
                },
                Command::Key {
                    key: Key::Enter,
                    speed_ms: None,
                    repeat: 1,
                },
                finish,
            ]
        }
        TerminalAction::Type { text, speed_ms } => vec![Command::Type {
            speed_ms: *speed_ms,
            text: text.clone(),
        }],
        TerminalAction::Keys { keys, repeat } => {
            if let Some(command) = chord_command(keys) {
                return repeated(command, *repeat);
            }
            let mut chars = keys.chars();
            if let (Some(c), None) = (chars.next(), chars.next()) {
                return repeated(
                    Command::Type {
                        speed_ms: None,
                        text: c.to_string(),
                    },
                    *repeat,
                );
            }
            skipped.push(step.label.clone());
            Vec::new()
        }
        TerminalAction::WaitText {
            pattern,
            scope,
            limit_ms,
        } => vec![Command::Wait {
            scope: *scope,
            timeout_ms: (*limit_ms != WAIT_TIMEOUT_MS).then_some(*limit_ms),
            pattern: Some(pattern.clone()),
        }],
        TerminalAction::WaitPrompt { limit_ms } => vec![Command::Wait {
            scope: LookIn::Line,
            timeout_ms: (*limit_ms != WAIT_TIMEOUT_MS).then_some(*limit_ms),
            pattern: None,
        }],
        TerminalAction::Sleep { ms } => vec![Command::Sleep { ms: *ms }],
        TerminalAction::StartRecording | TerminalAction::Show => vec![Command::Show],
        TerminalAction::Hide => vec![Command::Hide],
        TerminalAction::Clear => vec![Command::Ctrl {
            keys: vec!["L".to_owned()],
        }],
        TerminalAction::Scroll { lines } => {
            let key = if *lines < 0 {
                Key::ScrollUp
            } else {
                Key::ScrollDown
            };
            vec![Command::Key {
                key,
                speed_ms: None,
                repeat: lines.unsigned_abs(),
            }]
        }
        TerminalAction::Copy { text } => vec![Command::Copy { text: text.clone() }],
        TerminalAction::Paste => vec![Command::Paste],
        TerminalAction::Env { name, value } => vec![Command::Env {
            name: name.clone(),
            value: value.clone(),
        }],
        TerminalAction::Require { program } => vec![Command::Require {
            program: program.clone(),
        }],
        TerminalAction::Screenshot => vec![Command::Screenshot {
            path: format!("{}.png", step.id.replace(':', "-")),
        }],
        TerminalAction::Click { .. }
        | TerminalAction::Drag { .. }
        | TerminalAction::Wheel { .. }
        | TerminalAction::Chapter { .. } => {
            skipped.push(step.label.clone());
            Vec::new()
        }
    }
}
