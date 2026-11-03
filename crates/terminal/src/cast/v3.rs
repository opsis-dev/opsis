use std::collections::BTreeMap;
use std::time::Duration;

use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::util::{Quantizer, deserialize_time};
use super::{Cast, CastColour, CastError, CastTheme, Event, EventData, Header, Version};

const NANOS_PER_MILLI: u64 = 1_000_000;
const MILLIS_PER_SECOND: u128 = 1_000;

#[derive(Deserialize)]
struct V3Header {
    version: u8,
    term: V3Term,
    timestamp: Option<u64>,
    idle_time_limit: Option<f64>,
    command: Option<String>,
    title: Option<String>,
    env: Option<BTreeMap<String, String>>,
}

#[derive(Deserialize)]
struct V3Term {
    cols: u16,
    rows: u16,
    #[serde(rename = "type")]
    term_type: Option<String>,
    version: Option<String>,
    theme: Option<V3Theme>,
}

#[derive(Deserialize)]
struct V3Theme {
    #[serde(deserialize_with = "deserialize_colour")]
    fg: CastColour,
    #[serde(deserialize_with = "deserialize_colour")]
    bg: CastColour,
    #[serde(deserialize_with = "deserialize_palette")]
    palette: Vec<CastColour>,
}

#[derive(Debug, Deserialize)]
struct V3Event {
    #[serde(deserialize_with = "deserialize_time")]
    time: Duration,
    #[serde(deserialize_with = "deserialize_code")]
    code: V3EventCode,
    data: String,
}

#[derive(PartialEq, Eq, Debug)]
enum V3EventCode {
    Output,
    Input,
    Resize,
    Marker,
    Exit,
    Other(char),
}

pub struct Parser {
    header: V3Header,
}

pub fn open(header_line: &str) -> Result<Parser, CastError> {
    let header = serde_json::from_str::<V3Header>(header_line)
        .map_err(|e| CastError::Header(e.to_string()))?;
    if header.version != 3 {
        return Err(CastError::Version);
    }
    Ok(Parser { header })
}

pub fn parse_size(data: &str) -> Result<(u16, u16), String> {
    let (cols, rows) = data
        .split_once('x')
        .ok_or_else(|| "invalid size value in resize event".to_owned())?;
    let cols = cols
        .parse()
        .map_err(|e| format!("invalid cols value in resize event: {e}"))?;
    let rows = rows
        .parse()
        .map_err(|e| format!("invalid rows value in resize event: {e}"))?;
    Ok((cols, rows))
}

impl Parser {
    pub fn parse<'a>(self, lines: impl Iterator<Item = &'a str>) -> Result<Cast, CastError> {
        let h = self.header;
        let header = Header {
            term_cols: h.term.cols,
            term_rows: h.term.rows,
            term_type: h.term.term_type,
            term_version: h.term.version,
            term_theme: h.term.theme.map(|t| CastTheme {
                fg: t.fg,
                bg: t.bg,
                palette: t.palette,
            }),
            timestamp: h.timestamp,
            idle_time_limit: h.idle_time_limit,
            command: h.command,
            title: h.title,
            env: h.env,
        };
        let mut prev_time = Duration::ZERO;
        let mut events = Vec::new();
        for (i, line) in lines.enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let event = parse_event(line, i + 2, &mut prev_time)?;
            events.push(event);
        }
        Ok(Cast {
            version: Version::Three,
            header,
            events,
        })
    }
}

fn parse_event(line: &str, number: usize, prev_time: &mut Duration) -> Result<Event, CastError> {
    let failed = |message: String| CastError::Event {
        line: number,
        message,
    };
    let event = serde_json::from_str::<V3Event>(line).map_err(|e| failed(e.to_string()))?;
    let data = match event.code {
        V3EventCode::Output => EventData::Output(event.data),
        V3EventCode::Input => EventData::Input(event.data),
        V3EventCode::Resize => {
            let (cols, rows) = parse_size(&event.data).map_err(failed)?;
            EventData::Resize(cols, rows)
        }
        V3EventCode::Marker => EventData::Marker(event.data),
        V3EventCode::Exit => {
            EventData::Exit(event.data.parse().map_err(|e: std::num::ParseIntError| {
                failed(format!("invalid exit status: {e}"))
            })?)
        }
        V3EventCode::Other(c) => EventData::Other(c, event.data),
    };
    let time = *prev_time + event.time;
    *prev_time = time;
    Ok(Event { time, data })
}

fn deserialize_code<'de, D>(deserializer: D) -> Result<V3EventCode, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    let value: &str = Deserialize::deserialize(deserializer)?;
    match value {
        "o" => Ok(V3EventCode::Output),
        "i" => Ok(V3EventCode::Input),
        "r" => Ok(V3EventCode::Resize),
        "m" => Ok(V3EventCode::Marker),
        "x" => Ok(V3EventCode::Exit),
        s => s
            .chars()
            .next()
            .map(V3EventCode::Other)
            .ok_or_else(|| Error::custom("missing event code")),
    }
}

pub fn deserialize_colour<'de, D>(deserializer: D) -> Result<CastColour, D::Error>
where
    D: Deserializer<'de>,
{
    let value: &str = Deserialize::deserialize(deserializer)?;
    CastColour::parse(value).ok_or_else(|| serde::de::Error::custom("invalid hex triplet"))
}

pub fn deserialize_palette<'de, D>(deserializer: D) -> Result<Vec<CastColour>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: &str = Deserialize::deserialize(deserializer)?;
    let mut colours: Vec<CastColour> = value.split(':').filter_map(CastColour::parse).collect();
    match colours.len() {
        8 => colours.extend_from_within(..),
        16 => {}
        _ => return Err(serde::de::Error::custom("expected 8 or 16 hex triplets")),
    }
    Ok(colours)
}

struct HeaderOut<'a>(&'a Header);
struct TermOut<'a>(&'a Header);
struct ThemeOut<'a>(&'a CastTheme);

impl Serialize for HeaderOut<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let h = self.0;
        let env = h.env.as_ref().filter(|env| !env.is_empty());
        let len = 2
            + usize::from(h.timestamp.is_some())
            + usize::from(h.idle_time_limit.is_some())
            + usize::from(h.command.is_some())
            + usize::from(h.title.is_some())
            + usize::from(env.is_some());
        let mut map = serializer.serialize_map(Some(len))?;
        map.serialize_entry("version", &3)?;
        map.serialize_entry("term", &TermOut(h))?;
        if let Some(timestamp) = h.timestamp {
            map.serialize_entry("timestamp", &timestamp)?;
        }
        if let Some(limit) = h.idle_time_limit {
            map.serialize_entry("idle_time_limit", &limit)?;
        }
        if let Some(command) = &h.command {
            map.serialize_entry("command", command)?;
        }
        if let Some(title) = &h.title {
            map.serialize_entry("title", title)?;
        }
        if let Some(env) = env {
            map.serialize_entry("env", env)?;
        }
        map.end()
    }
}

impl Serialize for TermOut<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let h = self.0;
        let len = 2
            + usize::from(h.term_type.is_some())
            + usize::from(h.term_version.is_some())
            + usize::from(h.term_theme.is_some());
        let mut map = serializer.serialize_map(Some(len))?;
        map.serialize_entry("cols", &h.term_cols)?;
        map.serialize_entry("rows", &h.term_rows)?;
        if let Some(term_type) = &h.term_type {
            map.serialize_entry("type", term_type)?;
        }
        if let Some(version) = &h.term_version {
            map.serialize_entry("version", version)?;
        }
        if let Some(theme) = &h.term_theme {
            map.serialize_entry("theme", &ThemeOut(theme))?;
        }
        map.end()
    }
}

impl Serialize for ThemeOut<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let theme = self.0;
        let palette = theme
            .palette
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(":");
        let mut map = serializer.serialize_map(Some(3))?;
        map.serialize_entry("fg", &theme.fg.to_string())?;
        map.serialize_entry("bg", &theme.bg.to_string())?;
        map.serialize_entry("palette", &palette)?;
        map.end()
    }
}

#[derive(Debug)]
pub struct V3Writer {
    prev_time: Duration,
    quantizer: Quantizer,
}

impl Default for V3Writer {
    fn default() -> Self {
        Self::new()
    }
}

impl V3Writer {
    pub fn new() -> Self {
        Self {
            prev_time: Duration::ZERO,
            quantizer: Quantizer::new(NANOS_PER_MILLI),
        }
    }

    pub fn header(&mut self, header: &Header) -> String {
        let mut line = serde_json::to_string(&HeaderOut(header)).unwrap_or_default();
        line.push('\n');
        line
    }

    pub fn event(&mut self, event: &Event) -> String {
        let (code, data) = match &event.data {
            EventData::Output(data) => ('o', json_string(data)),
            EventData::Input(data) => ('i', json_string(data)),
            EventData::Resize(cols, rows) => ('r', json_string(&format!("{cols}x{rows}"))),
            EventData::Marker(data) => ('m', json_string(data)),
            EventData::Exit(status) => ('x', json_string(&status.to_string())),
            EventData::Other(code, data) => (*code, json_string(data)),
        };
        let dt = event.time.saturating_sub(self.prev_time);
        self.prev_time = event.time;
        let millis = self.quantizer.next(dt.as_nanos()) / u128::from(NANOS_PER_MILLI);
        format!(
            "[{}, {}, {data}]\n",
            format_millis(millis),
            json_string(&code.to_string()),
        )
    }
}

fn json_string(text: &str) -> String {
    serde_json::to_string(text).unwrap_or_default()
}

fn format_millis(millis: u128) -> String {
    format!(
        "{}.{:03}",
        millis / MILLIS_PER_SECOND,
        millis % MILLIS_PER_SECOND
    )
}

#[cfg(test)]
mod tests {
    use super::format_millis;

    #[test]
    fn format_time() {
        assert_eq!(format_millis(0), "0.000");
        assert_eq!(format_millis(666), "0.666");
        assert_eq!(format_millis(1000), "1.000");
        assert_eq!(format_millis(12345), "12.345");
    }
}
