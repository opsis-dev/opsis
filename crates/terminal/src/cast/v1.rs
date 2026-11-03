use std::collections::BTreeMap;
use std::time::Duration;

use serde::Deserialize;

use super::util::deserialize_time;
use super::{Cast, CastError, Event, Header, Version};

#[derive(Debug, Deserialize)]
struct V1 {
    version: u8,
    width: u16,
    height: u16,
    command: Option<String>,
    title: Option<String>,
    env: Option<BTreeMap<String, Option<String>>>,
    stdout: Vec<V1OutputEvent>,
}

#[derive(Debug, Deserialize)]
struct V1OutputEvent {
    #[serde(deserialize_with = "deserialize_time")]
    time: Duration,
    data: String,
}

pub fn load(json: &str) -> Result<Cast, CastError> {
    let cast: V1 = serde_json::from_str(json).map_err(|e| CastError::Header(e.to_string()))?;
    if cast.version != 1 {
        return Err(CastError::Version);
    }
    let term_type = cast
        .env
        .as_ref()
        .and_then(|env| env.get("TERM").cloned())
        .unwrap_or_default();
    let env = cast.env.map(|env| {
        env.into_iter()
            .filter_map(|(k, v)| v.map(|v| (k, v)))
            .collect()
    });
    let header = Header {
        term_cols: cast.width,
        term_rows: cast.height,
        term_type,
        term_version: None,
        term_theme: None,
        timestamp: None,
        idle_time_limit: None,
        command: cast.command,
        title: cast.title,
        env,
    };
    let mut time = Duration::ZERO;
    let events = cast
        .stdout
        .into_iter()
        .map(|event| {
            time += event.time;
            Event::output(time, event.data)
        })
        .collect();
    Ok(Cast {
        version: Version::One,
        header,
        events,
    })
}
