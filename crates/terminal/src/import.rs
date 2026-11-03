use std::time::Duration;

use opsis_types::actions::{
    Action, ActionKind, ActionLog, CaretSample, Point, Rect, TitleChange, Viewport,
};
use opsis_types::numbers::millis;
use opsis_types::terminal::TerminalTheme;
use opsis_vt::{Event as VtEvent, Terminal, VtError};

use crate::cast::{Cast, Event, EventData, Header, limit_idle_time};
use crate::take::{visible_actions, visible_caret, visible_titles};
use crate::timeline::{CHAPTER_PREFIX, Timeline};

const SCROLLBACK: u32 = 1000;
const TYPING_GAP: Duration = Duration::from_secs(1);
const CELL_MIDDLE: f64 = 0.5;
const CAST_THEME: &str = "Cast file";

const SEQUENCES: [(&str, &str); 14] = [
    ("\r", "Enter"),
    ("\n", "Enter"),
    ("\t", "Tab"),
    ("\u{7f}", "Backspace"),
    ("\u{8}", "Backspace"),
    ("\u{1b}", "Escape"),
    ("\u{1b}[A", "ArrowUp"),
    ("\u{1b}[B", "ArrowDown"),
    ("\u{1b}[C", "ArrowRight"),
    ("\u{1b}[D", "ArrowLeft"),
    ("\u{1b}OA", "ArrowUp"),
    ("\u{1b}OB", "ArrowDown"),
    ("\u{1b}OC", "ArrowRight"),
    ("\u{1b}OD", "ArrowLeft"),
];

#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("The cast file shows nothing: it has no output")]
    NoOutput,
    #[error("The cast file's terminal has no size")]
    NoSize,
    #[error(transparent)]
    Terminal(#[from] VtError),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Imported {
    pub header: Header,
    pub events: Vec<Event>,
    pub log: ActionLog,
    pub length: Duration,
    pub theme: Option<TerminalTheme>,
}

fn key_name(text: &str) -> Option<String> {
    if let Some((_, name)) = SEQUENCES.iter().find(|(seq, _)| *seq == text) {
        return Some((*name).to_owned());
    }
    let mut chars = text.chars();
    let (Some(c), None) = (chars.next(), chars.next()) else {
        return None;
    };
    let code = u32::from(c);
    (1..=26)
        .contains(&code)
        .then(|| char::from_u32(code + u32::from(b'A') - 1))
        .flatten()
        .map(|letter| format!("Control+{letter}"))
}

fn printable(text: &str) -> bool {
    !text.is_empty() && !text.chars().any(char::is_control)
}

struct Typing {
    t: Duration,
    text: String,
    times: Vec<f64>,
    from: (u16, u16),
}

struct Replay {
    terminal: Terminal,
    caret: Vec<CaretSample>,
    titles: Vec<TitleChange>,
    actions: Vec<Action>,
    typing: Option<Typing>,
    cols: u16,
}

fn cell_rect(col: u16, row: u16, cols: u16, rows: u16) -> Rect {
    Rect {
        x: f64::from(col),
        y: f64::from(row),
        w: f64::from(cols.max(1)),
        h: f64::from(rows.max(1)),
    }
}

impl Replay {
    fn cursor(&mut self) -> Result<(u16, u16), VtError> {
        let cursor = self.terminal.frame()?.cursor();
        Ok((cursor.x, cursor.y))
    }

    fn output(&mut self, at: Duration, text: &str) -> Result<(), VtError> {
        self.terminal.feed(text.as_bytes());
        self.terminal.take_replies();
        for event in self.terminal.take_events() {
            if matches!(event, VtEvent::Title) {
                let title = self.terminal.title();
                if self.titles.last().is_none_or(|t| t.title != title) {
                    self.titles.push(TitleChange {
                        t: millis(at),
                        title,
                    });
                }
            }
        }
        let (col, row) = self.cursor()?;
        if self
            .caret
            .last()
            .is_none_or(|c| (c.col, c.row) != (col, row))
        {
            self.caret.push(CaretSample {
                t: millis(at),
                col,
                row,
            });
        }
        Ok(())
    }

    fn finish_typing(&mut self) -> Result<(), VtError> {
        let Some(typing) = self.typing.take() else {
            return Ok(());
        };
        let (col, row) = self.cursor()?;
        let (from_col, from_row) = typing.from;
        let rect = if row == from_row {
            cell_rect(from_col, from_row, col.saturating_sub(from_col), 1)
        } else {
            cell_rect(0, from_row, self.cols, row.saturating_sub(from_row) + 1)
        };
        let id = format!("typed-{}", self.actions.len());
        let mut action = Action::new(id, ActionKind::Type, millis(typing.t));
        action.t_end = typing.times.last().copied();
        action.text = Some(typing.text);
        action.key_times = Some(typing.times);
        action.point = Some(Point {
            x: rect.w.mul_add(CELL_MIDDLE, rect.x),
            y: rect.h.mul_add(CELL_MIDDLE, rect.y),
        });
        action.rect = Some(rect);
        self.actions.push(action);
        Ok(())
    }

    fn input(&mut self, at: Duration, text: &str) -> Result<(), VtError> {
        if printable(text) {
            let continues = self
                .typing
                .as_ref()
                .and_then(|t| t.times.last())
                .is_some_and(|last| millis(at) - last <= millis(TYPING_GAP));
            if !continues {
                self.finish_typing()?;
                let from = self.cursor()?;
                self.typing = Some(Typing {
                    t: at,
                    text: String::new(),
                    times: Vec::new(),
                    from,
                });
            }
            if let Some(typing) = self.typing.as_mut() {
                typing.text.push_str(text);
                typing.times.extend(text.chars().map(|_| millis(at)));
            }
            return Ok(());
        }
        self.finish_typing()?;
        let label = key_name(text).unwrap_or_else(|| "A key".to_owned());
        let id = format!("key-{}", self.actions.len());
        let mut action = Action::new(id, ActionKind::Key, millis(at));
        action.label = Some(label);
        action.key_times = Some(vec![millis(at)]);
        self.actions.push(action);
        Ok(())
    }

    fn marker(&mut self, at: Duration, label: &str) -> Result<(), VtError> {
        if label.starts_with("opsis:") && !label.starts_with(CHAPTER_PREFIX) {
            return Ok(());
        }
        self.finish_typing()?;
        let title = label.strip_prefix(CHAPTER_PREFIX).unwrap_or(label);
        let id = format!("chapter-{}", self.actions.len());
        let mut chapter = Action::new(id, ActionKind::Chapter, millis(at));
        chapter.label = Some(title.to_owned());
        self.actions.push(chapter);
        Ok(())
    }
}

pub fn import_cast(cast: Cast, limit: Option<Duration>) -> Result<Imported, ImportError> {
    let header = cast.header;
    if header.term_cols == 0 || header.term_rows == 0 {
        return Err(ImportError::NoSize);
    }
    if !cast
        .events
        .iter()
        .any(|e| matches!(&e.data, EventData::Output(text) if !text.is_empty()))
    {
        return Err(ImportError::NoOutput);
    }
    let limit = limit.or_else(|| {
        header
            .idle_time_limit
            .filter(|s| s.is_finite() && *s > 0.0)
            .map(Duration::from_secs_f64)
    });
    let events = match limit {
        Some(limit) => limit_idle_time(cast.events, limit),
        None => cast.events,
    };
    let mut replay = Replay {
        terminal: Terminal::new(header.term_cols, header.term_rows, SCROLLBACK)?,
        caret: Vec::new(),
        titles: Vec::new(),
        actions: Vec::new(),
        typing: None,
        cols: header.term_cols,
    };
    for event in &events {
        match &event.data {
            EventData::Output(text) => replay.output(event.time, text)?,
            EventData::Input(text) => replay.input(event.time, text)?,
            EventData::Resize(cols, rows) => {
                replay.terminal.resize(*cols, *rows)?;
                replay.cols = *cols;
            }
            EventData::Marker(label) => replay.marker(event.time, label)?,
            EventData::Exit(_) | EventData::Other(..) => {}
        }
    }
    replay.finish_typing()?;
    let timeline = Timeline::from_events(&events);
    let log = ActionLog {
        viewport: Viewport {
            width: u32::from(header.term_cols),
            height: u32::from(header.term_rows),
        },
        dpr: 1.0,
        t0: 0.0,
        duration_ms: millis(timeline.length()),
        actions: visible_actions(&replay.actions, &timeline),
        cursor_track: None,
        addresses: Vec::new(),
        caret_track: visible_caret(&replay.caret, &timeline),
        titles: visible_titles(&replay.titles, &timeline),
        cells: None,
    };
    let theme = header.term_theme.as_ref().map(|t| t.to_theme(CAST_THEME));
    Ok(Imported {
        length: timeline.length(),
        header: Header {
            idle_time_limit: None,
            ..header
        },
        events,
        log,
        theme,
    })
}
