use std::path::Path;
use std::time::Duration;

use opsis_types::numbers::millis;
use opsis_types::terminal::{PointerMove, ShellChoice, TerminalInput, TerminalTheme};
use opsis_vt::{CellPoint, KeyAction, KeyMods, MouseAction, MouseButton};

use crate::frames::{FramesError, write};
use crate::input::mouse_button;
use crate::prepare::snapshot;
use crate::session::{Session, SessionError};
use crate::shell::{ShellError, shell_command};
use crate::tape::{Command, Key, Tape};

pub const PAUSE: Duration = Duration::from_millis(500);
const SLEEP_STEP_MS: f64 = 10.0;
const WHEEL_LINES: i32 = 3;
const EXIT_LINE: &[u8] = b"exit\r";

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TapeRecorder {
    commands: Vec<Command>,
    typed: String,
    last: Option<Duration>,
    skipped: u32,
}

impl TapeRecorder {
    pub fn input(&mut self, at: Duration, input: &TerminalInput) {
        match input {
            TerminalInput::Text { text } => {
                self.pause(at);
                self.typed.push_str(text);
            }
            TerminalInput::Paste { text } => {
                self.pause(at);
                self.flush();
                self.commands.push(Command::Copy { text: text.clone() });
                self.commands.push(Command::Paste);
            }
            TerminalInput::Key {
                key,
                ctrl,
                alt,
                shift,
            } => {
                self.pause(at);
                self.flush();
                self.key(key, *ctrl, *alt, *shift);
            }
            TerminalInput::Pointer { .. }
            | TerminalInput::Wheel { .. }
            | TerminalInput::Focus { .. } => {}
        }
    }

    pub const fn skipped(&self) -> u32 {
        self.skipped
    }

    fn pause(&mut self, at: Duration) {
        if let Some(last) = self.last {
            let gap = at.saturating_sub(last);
            if gap >= PAUSE {
                self.flush();
                let ms = (millis(gap) / SLEEP_STEP_MS).round() * SLEEP_STEP_MS;
                self.commands.push(Command::Sleep {
                    ms: opsis_types::numbers::whole(ms),
                });
            }
        }
        self.last = Some(at);
    }

    fn flush(&mut self) {
        if !self.typed.is_empty() {
            self.commands.push(Command::Type {
                speed_ms: None,
                text: std::mem::take(&mut self.typed),
            });
        }
    }

    fn key(&mut self, key: &str, ctrl: bool, alt: bool, shift: bool) {
        if ctrl {
            let mut keys = Vec::new();
            if alt {
                keys.push("Alt".to_owned());
            }
            if shift {
                keys.push("Shift".to_owned());
            }
            keys.push(key.to_owned());
            self.commands.push(Command::Ctrl { keys });
            return;
        }
        if alt {
            self.commands.push(Command::Alt {
                key: key.to_owned(),
            });
            return;
        }
        let Some(named) = Key::ALL.into_iter().find(|k| k.chord() == key) else {
            self.skipped += 1;
            return;
        };
        if shift {
            self.commands.push(Command::Shift {
                key: named.word().to_owned(),
            });
            return;
        }
        if let Some(Command::Key {
            key: last,
            speed_ms: None,
            repeat,
        }) = self.commands.last_mut()
            && *last == named
        {
            *repeat += 1;
            return;
        }
        self.commands.push(Command::Key {
            key: named,
            speed_ms: None,
            repeat: 1,
        });
    }

    pub fn finish(mut self, at: Duration, cols: u16, rows: u16) -> Tape {
        self.pause(at);
        self.flush();
        let mut commands = vec![
            Command::Set {
                name: "Columns".to_owned(),
                value: cols.to_string(),
            },
            Command::Set {
                name: "Rows".to_owned(),
                value: rows.to_string(),
            },
        ];
        commands.append(&mut self.commands);
        Tape { commands }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LiveError {
    #[error(transparent)]
    Shell(#[from] ShellError),
    #[error(transparent)]
    Session(#[from] SessionError),
    #[error(transparent)]
    Terminal(#[from] opsis_vt::VtError),
    #[error(transparent)]
    Frames(#[from] FramesError),
}

#[derive(Debug)]
pub struct LiveTape {
    session: Session,
    recorder: TapeRecorder,
    cols: u16,
    rows: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct LiveOptions<'a> {
    pub folder: &'a Path,
    pub temp: &'a Path,
    pub cols: u16,
    pub rows: u16,
}

const fn cell(col: u16, row: u16) -> CellPoint {
    CellPoint { col, row }
}

impl LiveTape {
    pub fn start(options: &LiveOptions<'_>) -> Result<Self, LiveError> {
        let command = shell_command(ShellChoice::Clean, options.folder, options.temp, &[])?;
        let session = Session::start(&command, options.cols, options.rows)?;
        Ok(Self {
            session,
            recorder: TapeRecorder::default(),
            cols: options.cols,
            rows: options.rows,
        })
    }

    pub fn input(&mut self, input: &TerminalInput) -> Result<(), LiveError> {
        let bytes = {
            let mut terminal = self.session.terminal();
            match input {
                TerminalInput::Key {
                    key,
                    ctrl,
                    alt,
                    shift,
                } => {
                    let mods = KeyMods {
                        ctrl: *ctrl,
                        alt: *alt,
                        shift: *shift,
                        ..KeyMods::NONE
                    };
                    terminal.key_bytes(key, mods, KeyAction::Press)?
                }
                TerminalInput::Text { text } => text.as_bytes().to_vec(),
                TerminalInput::Paste { text } => terminal.paste_bytes(text),
                TerminalInput::Pointer {
                    action,
                    button,
                    col,
                    row,
                } => {
                    if !terminal.mouse_reporting() {
                        return Ok(());
                    }
                    let action = match action {
                        PointerMove::Press => MouseAction::Press,
                        PointerMove::Release => MouseAction::Release,
                        PointerMove::Move => MouseAction::Motion,
                    };
                    let button = button.map_or(MouseButton::None, mouse_button);
                    terminal.mouse_bytes(action, button, KeyMods::NONE, cell(*col, *row))
                }
                TerminalInput::Wheel { col, row, notches } => {
                    if !terminal.mouse_reporting() {
                        terminal.scroll(notches.saturating_mul(WHEEL_LINES));
                        return Ok(());
                    }
                    let button = if *notches < 0 {
                        MouseButton::WheelUp
                    } else {
                        MouseButton::WheelDown
                    };
                    (0..notches.unsigned_abs())
                        .flat_map(|_| {
                            terminal.mouse_bytes(
                                MouseAction::Press,
                                button,
                                KeyMods::NONE,
                                cell(*col, *row),
                            )
                        })
                        .collect()
                }
                TerminalInput::Focus { gained } => terminal.focus_bytes(*gained),
            }
        };
        let at = self.session.write(&bytes)?;
        self.recorder.input(at, input);
        Ok(())
    }

    pub fn pump(&self, wait: Duration) -> bool {
        let mut changed = false;
        let mut wait = wait;
        while self.session.next_chunk(wait).is_some() {
            changed = true;
            wait = Duration::ZERO;
        }
        changed
    }

    pub fn frame(&self, theme: &TerminalTheme) -> Result<Vec<u8>, LiveError> {
        let frames = snapshot(&mut self.session.terminal(), theme)?;
        Ok(write(&frames)?)
    }

    pub const fn skipped(&self) -> u32 {
        self.recorder.skipped()
    }

    pub fn finish(self) -> Tape {
        let at = self.session.now();
        let tape = self.recorder.finish(at, self.cols, self.rows);
        drop(self.session.finish(EXIT_LINE));
        tape
    }
}
