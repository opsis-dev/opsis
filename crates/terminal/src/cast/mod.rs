mod util;
mod v1;
mod v2;
mod v3;

use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;
use std::time::Duration;

use opsis_types::terminal::TerminalTheme;
pub use util::Utf8Decoder;
pub use v3::V3Writer;

#[derive(Debug, thiserror::Error)]
pub enum CastError {
    #[error("The cast file is empty")]
    Empty,
    #[error("This isn't a cast file of version 1, 2 or 3")]
    Version,
    #[error("The cast file's header can't be read: {0}")]
    Header(String),
    #[error("Line {line} of the cast file can't be read: {message}")]
    Event { line: usize, message: String },
    #[error("The cast file can't be read: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Version {
    One,
    Two,
    Three,
}

impl PartialEq<u8> for Version {
    fn eq(&self, other: &u8) -> bool {
        matches!(
            (self, other),
            (Self::One, 1) | (Self::Two, 2) | (Self::Three, 3)
        )
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::One => write!(f, "1"),
            Self::Two => write!(f, "2"),
            Self::Three => write!(f, "3"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CastColour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl CastColour {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn parse(hex: &str) -> Option<Self> {
        if hex.len() != 7 || !hex.starts_with('#') {
            return None;
        }
        let r = u8::from_str_radix(hex.get(1..3)?, 16).ok()?;
        let g = u8::from_str_radix(hex.get(3..5)?, 16).ok()?;
        let b = u8::from_str_radix(hex.get(5..7)?, 16).ok()?;
        Some(Self { r, g, b })
    }
}

impl fmt::Display for CastColour {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:0>2x}{:0>2x}{:0>2x}", self.r, self.g, self.b)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CastTheme {
    pub fg: CastColour,
    pub bg: CastColour,
    pub palette: Vec<CastColour>,
}

impl CastTheme {
    pub fn to_theme(&self, name: &str) -> TerminalTheme {
        let colour = |i: usize| self.palette.get(i).copied().unwrap_or(self.fg).to_string();
        TerminalTheme {
            name: name.to_owned(),
            fg: self.fg.to_string(),
            bg: self.bg.to_string(),
            cursor: self.fg.to_string(),
            palette: std::array::from_fn(colour),
        }
    }

    pub fn from_theme(theme: &TerminalTheme) -> Option<Self> {
        let palette = theme
            .palette
            .iter()
            .map(|c| CastColour::parse(c))
            .collect::<Option<Vec<_>>>()?;
        Some(Self {
            fg: CastColour::parse(&theme.fg)?,
            bg: CastColour::parse(&theme.bg)?,
            palette,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Header {
    pub term_cols: u16,
    pub term_rows: u16,
    pub term_type: Option<String>,
    pub term_version: Option<String>,
    pub term_theme: Option<CastTheme>,
    pub timestamp: Option<u64>,
    pub idle_time_limit: Option<f64>,
    pub command: Option<String>,
    pub title: Option<String>,
    pub env: Option<BTreeMap<String, String>>,
}

impl Default for Header {
    fn default() -> Self {
        Self {
            term_cols: 80,
            term_rows: 24,
            term_type: None,
            term_version: None,
            term_theme: None,
            timestamp: None,
            idle_time_limit: None,
            command: None,
            title: None,
            env: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub time: Duration,
    pub data: EventData,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventData {
    Output(String),
    Input(String),
    Resize(u16, u16),
    Marker(String),
    Exit(i32),
    Other(char, String),
}

impl Event {
    pub const fn output(time: Duration, text: String) -> Self {
        Self {
            time,
            data: EventData::Output(text),
        }
    }

    pub const fn input(time: Duration, text: String) -> Self {
        Self {
            time,
            data: EventData::Input(text),
        }
    }

    pub const fn resize(time: Duration, size: (u16, u16)) -> Self {
        Self {
            time,
            data: EventData::Resize(size.0, size.1),
        }
    }

    pub const fn marker(time: Duration, label: String) -> Self {
        Self {
            time,
            data: EventData::Marker(label),
        }
    }

    pub const fn exit(time: Duration, status: i32) -> Self {
        Self {
            time,
            data: EventData::Exit(status),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Cast {
    pub version: Version,
    pub header: Header,
    pub events: Vec<Event>,
}

impl Cast {
    pub fn duration(&self) -> Duration {
        self.events.last().map_or(Duration::ZERO, |e| e.time)
    }
}

pub fn read_path(path: impl AsRef<Path>) -> Result<Cast, CastError> {
    read(&std::fs::read_to_string(path)?)
}

pub fn read(text: &str) -> Result<Cast, CastError> {
    let mut lines = text.lines();
    let first = lines.next().ok_or(CastError::Empty)?;
    if first.trim().is_empty() {
        return Err(CastError::Empty);
    }
    if let Ok(parser) = v3::open(first) {
        return parser.parse(lines);
    }
    if let Ok(parser) = v2::open(first) {
        return parser.parse(lines);
    }
    v1::load(text).map_err(|_| CastError::Version)
}

pub fn limit_idle_time(events: Vec<Event>, limit: Duration) -> Vec<Event> {
    let mut prev_time = Duration::ZERO;
    let mut offset = Duration::ZERO;
    events
        .into_iter()
        .map(|event| {
            let delay = event.time.saturating_sub(prev_time);
            if delay > limit {
                offset += delay.saturating_sub(limit);
            }
            prev_time = event.time;
            Event {
                time: event.time.saturating_sub(offset),
                ..event
            }
        })
        .collect()
}

pub fn accelerate(events: Vec<Event>, speed: f64) -> Vec<Event> {
    events
        .into_iter()
        .map(|event| Event {
            time: event.time.div_f64(speed),
            ..event
        })
        .collect()
}

pub fn write_v3(header: &Header, events: &[Event]) -> String {
    let mut writer = V3Writer::new();
    let mut out = writer.header(header);
    for event in events {
        out.push_str(&writer.event(event));
    }
    out
}
