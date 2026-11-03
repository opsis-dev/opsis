use std::collections::BTreeMap;
use std::time::Duration;

use serde::{Deserialize, Deserializer};

use super::util::deserialize_time;
use super::v3::{deserialize_colour, deserialize_palette};
use super::{Cast, CastColour, CastError, CastTheme, Event, EventData, Header, Version};

#[derive(Deserialize)]
struct V2Header {
    version: u8,
    width: u16,
    height: u16,
    timestamp: Option<u64>,
    idle_time_limit: Option<f64>,
    command: Option<String>,
    title: Option<String>,
    env: Option<BTreeMap<String, Option<String>>>,
    theme: Option<V2Theme>,
}

#[derive(Deserialize)]
struct V2Theme {
    #[serde(deserialize_with = "deserialize_colour")]
    fg: CastColour,
    #[serde(deserialize_with = "deserialize_colour")]
    bg: CastColour,
    #[serde(deserialize_with = "deserialize_palette")]
    palette: Vec<CastColour>,
}

#[derive(Debug, Deserialize)]
struct V2Event {
    #[serde(deserialize_with = "deserialize_time")]
    time: Duration,
    #[serde(deserialize_with = "deserialize_code")]
    code: V2EventCode,
    data: String,
}

#[derive(PartialEq, Eq, Debug)]
enum V2EventCode {
    Output,
    Input,
    Resize,
    Marker,
    Other(char),
}

pub struct Parser(V2Header);

pub fn open(header_line: &str) -> Result<Parser, CastError> {
    let header = serde_json::from_str::<V2Header>(header_line)
        .map_err(|e| CastError::Header(e.to_string()))?;
    if header.version != 2 {
        return Err(CastError::Version);
    }
    Ok(Parser(header))
}

impl Parser {
    pub fn parse<'a>(self, lines: impl Iterator<Item = &'a str>) -> Result<Cast, CastError> {
        let term_type = self
            .0
            .env
            .as_ref()
            .and_then(|env| env.get("TERM").cloned())
            .unwrap_or_default();
        let term_theme = self.0.theme.map(|t| CastTheme {
            fg: t.fg,
            bg: t.bg,
            palette: t.palette,
        });
        let env = self.0.env.map(|env| {
            env.into_iter()
                .filter_map(|(k, v)| v.map(|v| (k, v)))
                .collect()
        });
        let header = Header {
            term_cols: self.0.width,
            term_rows: self.0.height,
            term_type,
            term_version: None,
            term_theme,
            timestamp: self.0.timestamp,
            idle_time_limit: self.0.idle_time_limit,
            command: self.0.command,
            title: self.0.title,
            env,
        };
        let events = lines
            .enumerate()
            .filter(|(_, line)| !line.is_empty())
            .map(|(i, line)| parse_event(line, i + 2))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Cast {
            version: Version::Two,
            header,
            events,
        })
    }
}

fn parse_event(line: &str, number: usize) -> Result<Event, CastError> {
    let failed = |message: String| CastError::Event {
        line: number,
        message,
    };
    let event = serde_json::from_str::<V2Event>(line).map_err(|e| failed(e.to_string()))?;
    let data = match event.code {
        V2EventCode::Output => EventData::Output(event.data),
        V2EventCode::Input => EventData::Input(event.data),
        V2EventCode::Resize => {
            let (cols, rows) = super::v3::parse_size(&event.data).map_err(failed)?;
            EventData::Resize(cols, rows)
        }
        V2EventCode::Marker => EventData::Marker(event.data),
        V2EventCode::Other(c) => EventData::Other(c, event.data),
    };
    Ok(Event {
        time: event.time,
        data,
    })
}

fn deserialize_code<'de, D>(deserializer: D) -> Result<V2EventCode, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    let value: &str = Deserialize::deserialize(deserializer)?;
    match value {
        "o" => Ok(V2EventCode::Output),
        "i" => Ok(V2EventCode::Input),
        "r" => Ok(V2EventCode::Resize),
        "m" => Ok(V2EventCode::Marker),
        s => s
            .chars()
            .next()
            .map(V2EventCode::Other)
            .ok_or_else(|| Error::custom("missing event code")),
    }
}
