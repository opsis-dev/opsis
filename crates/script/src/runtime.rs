use std::cell::Cell;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use mlua::chunk::ChunkMode;
use mlua::{Error, HookTriggers, Lua, LuaOptions, MultiValue, StdLib, Value, VmState};
use opsis_types::script::{ScriptFailure, ScriptLineKind};

use crate::folder::Folder;
use crate::host::ScriptHost;
use crate::output::{Sink, Streams};
use crate::sandbox::{Control, chunk_name};

pub const MEMORY_LIMIT: usize = 64 * 1024 * 1024;
pub const CHECK_EVERY: u32 = 10_000;
const TRACEBACK: &str = "\nstack traceback:";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Script {
    pub file: String,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ending {
    pub value: Option<serde_json::Value>,
    pub failure: Option<ScriptFailure>,
    pub stopped: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum ScriptError {
    #[error("Couldn't open the project folder: {0}")]
    Folder(#[from] std::io::Error),
    #[error("{0}")]
    Lua(#[from] Error),
}

fn root_cause(error: &Error) -> &Error {
    match error {
        Error::CallbackError { cause, .. } | Error::WithContext { cause, .. } => root_cause(cause),
        other => other,
    }
}

pub fn location(text: &str) -> Option<(String, u32)> {
    let mut from = 0;
    while let Some(found) = text[from..].find(".lua:") {
        let end = from + found + ".lua".len();
        let start = text[..end]
            .rfind(|c: char| c.is_whitespace() || c == '"' || c == '\'')
            .map_or(0, |i| i + 1);
        let digits: String = text[end + 1..]
            .chars()
            .take_while(char::is_ascii_digit)
            .collect();
        let after = text[end + 1 + digits.len()..].starts_with(':');
        if !digits.is_empty()
            && after
            && start < end
            && let Ok(line) = digits.parse()
        {
            return Some((text[start..end].to_owned(), line));
        }
        from = end;
    }
    None
}

pub fn failure_of(error: &Error, file: &str) -> ScriptFailure {
    let whole = error.to_string();
    let root = root_cause(error);
    let text = match root {
        Error::ExternalError(inner) => inner.to_string(),
        Error::RuntimeError(text) | Error::SyntaxError { message: text, .. } => text.clone(),
        Error::MemoryError(_) => "The script ran out of memory.".to_owned(),
        other => other.to_string(),
    };
    let message = text
        .split(TRACEBACK)
        .next()
        .unwrap_or_default()
        .trim()
        .to_owned();
    let place = location(&message).or_else(|| location(&whole));
    let message = match (&place, location(&message)) {
        (Some((at, line)), None) => format!("{at}:{line}: {message}"),
        _ => message,
    };
    ScriptFailure {
        message,
        file: place
            .as_ref()
            .map_or_else(|| file.to_owned(), |(at, _)| at.clone()),
        line: place.map(|(_, line)| line),
    }
}

fn first_value(lua: &Lua, values: MultiValue) -> Option<serde_json::Value> {
    match values.into_iter().next() {
        None | Some(Value::Nil) => None,
        Some(value) => match crate::values::from_lua::<serde_json::Value>(lua, value) {
            Ok(json) => Some(json),
            Err(error) => {
                log::debug!("a script's value isn't JSON: {error}");
                None
            }
        },
    }
}

fn state(control: &Rc<Control>) -> Result<Lua, Error> {
    let lua = Lua::new_with(
        StdLib::COROUTINE
            | StdLib::TABLE
            | StdLib::STRING
            | StdLib::UTF8
            | StdLib::MATH
            | StdLib::OS,
        LuaOptions::new(),
    )?;
    lua.set_memory_limit(MEMORY_LIMIT)?;
    let stop = Arc::clone(&control.stop);
    lua.set_global_hook(
        HookTriggers::new().every_nth_instruction(CHECK_EVERY),
        move |_, _| {
            if stop.load(std::sync::atomic::Ordering::Relaxed) {
                Err(Error::external(crate::sandbox::Halt::Stopped))
            } else {
                Ok(VmState::Continue)
            }
        },
    )?;
    Ok(lua)
}

pub fn run(
    script: &Script,
    root: &Path,
    host: Rc<dyn ScriptHost>,
    sink: Sink,
    stop: Arc<AtomicBool>,
) -> Ending {
    let control = Rc::new(Control {
        stop,
        exit: Cell::new(None),
    });
    let streams = Streams::new(sink);
    let outcome = (|| -> Result<Option<serde_json::Value>, ScriptError> {
        let folder = Rc::new(Folder::open(root)?);
        let lua = state(&control)?;
        crate::sandbox::install(&lua, &folder, &streams, &control)?;
        crate::module::install(&lua, host, &control)?;
        let values = lua
            .load(script.source.as_str())
            .set_name(chunk_name(&script.file))
            .set_mode(ChunkMode::Text)
            .call::<MultiValue>(())?;
        Ok(first_value(&lua, values))
    })();
    streams.flush_all();
    let stopped = matches!(control.halted(), Some(crate::sandbox::Halt::Stopped))
        && control.exit.get().is_none();
    let ending = match (outcome, control.exit.get()) {
        (_, Some(true)) => Ending {
            value: None,
            failure: None,
            stopped: false,
        },
        (_, Some(false)) => Ending {
            value: None,
            failure: Some(ScriptFailure {
                message: "The script ended with a failure status.".to_owned(),
                file: script.file.clone(),
                line: None,
            }),
            stopped: false,
        },
        (_, None) if stopped => Ending {
            value: None,
            failure: Some(ScriptFailure {
                message: "The script was stopped.".to_owned(),
                file: script.file.clone(),
                line: None,
            }),
            stopped: true,
        },
        (Ok(value), None) => Ending {
            value,
            failure: None,
            stopped: false,
        },
        (Err(ScriptError::Lua(error)), None) => Ending {
            value: None,
            failure: Some(failure_of(&error, &script.file)),
            stopped: false,
        },
        (Err(error), None) => Ending {
            value: None,
            failure: Some(ScriptFailure {
                message: error.to_string(),
                file: script.file.clone(),
                line: None,
            }),
            stopped: false,
        },
    };
    if let Some(failure) = &ending.failure {
        streams.line(ScriptLineKind::Error, &failure.message);
    }
    ending
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_place_from_a_message_or_its_traceback() {
        assert_eq!(
            location("scripts/a.lua:12: boom"),
            Some(("scripts/a.lua".to_owned(), 12))
        );
        assert_eq!(
            location("boom\nstack traceback:\n\t[C]: in ?\n\tscripts/b.lua:3: in main chunk"),
            Some(("scripts/b.lua".to_owned(), 3))
        );
        assert_eq!(location("opsis:4: nothing here"), None);
    }
}
