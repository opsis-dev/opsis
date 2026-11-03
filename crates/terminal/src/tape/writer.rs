use opsis_types::terminal::LookIn;

use super::model::{Command, Tape};

const QUOTES: [char; 3] = ['"', '\'', '`'];

fn is_bare(text: &str) -> bool {
    !text.is_empty()
        && text.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
        && text
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '/' | '%'))
}

fn quoted(text: &str) -> Vec<String> {
    if let Some(q) = QUOTES.iter().find(|q| !text.contains(**q)) {
        return vec![format!("{q}{text}{q}")];
    }
    let mut parts = Vec::new();
    let mut part = String::new();
    for c in text.chars() {
        let next = format!("{part}{c}");
        if QUOTES.iter().all(|q| next.contains(*q)) {
            parts.extend(quoted(&part));
            part = c.to_string();
        } else {
            part = next;
        }
    }
    parts.extend(quoted(&part));
    parts
}

fn time(ms: u32) -> String {
    format!("{ms}ms")
}

fn speed(speed_ms: Option<u32>) -> String {
    speed_ms.map_or_else(String::new, |ms| format!("@{}", time(ms)))
}

fn value(text: &str) -> String {
    if text.starts_with('{') || is_bare(text) || text.parse::<f64>().is_ok() {
        return text.to_owned();
    }
    let ends_with_unit = text
        .strip_suffix("ms")
        .or_else(|| text.strip_suffix('%'))
        .is_some_and(|n| n.parse::<f64>().is_ok());
    if ends_with_unit {
        return text.to_owned();
    }
    quoted(text).join(" ")
}

pub(super) fn line(command: &Command) -> Vec<String> {
    match command {
        Command::Type { speed_ms, text } => quoted(text)
            .into_iter()
            .map(|q| format!("Type{} {q}", speed(*speed_ms)))
            .collect(),
        Command::Key {
            key,
            speed_ms,
            repeat,
        } => {
            let count = if *repeat == 1 {
                String::new()
            } else {
                format!(" {repeat}")
            };
            vec![format!("{}{}{count}", key.word(), speed(*speed_ms))]
        }
        Command::Ctrl { keys } => vec![format!("Ctrl+{}", keys.join("+"))],
        Command::Alt { key } => vec![format!("Alt+{key}")],
        Command::Shift { key } => vec![format!("Shift+{key}")],
        Command::Sleep { ms } => vec![format!("Sleep {}", time(*ms))],
        Command::Hide => vec!["Hide".to_owned()],
        Command::Show => vec!["Show".to_owned()],
        Command::Wait {
            scope,
            timeout_ms,
            pattern,
        } => {
            let scope = if *scope == LookIn::Screen {
                "+Screen"
            } else {
                ""
            };
            let pattern = pattern
                .as_ref()
                .map_or_else(String::new, |p| format!(" /{p}/"));
            vec![format!("Wait{scope}{}{pattern}", speed(*timeout_ms))]
        }
        Command::Require { program } => vec![format!("Require {}", value(program))],
        Command::Screenshot { path } => vec![format!("Screenshot {}", quoted(path).join(" "))],
        Command::Copy { text } => vec![format!("Copy {}", quoted(text).join(" "))],
        Command::Paste => vec!["Paste".to_owned()],
        Command::Env { name, value: v } => vec![format!("Env {name} {}", quoted(v).join(" "))],
        Command::Source { path } => vec![format!("Source {}", quoted(path).join(" "))],
        Command::Output { path } => vec![format!("Output {}", quoted(path).join(" "))],
        Command::Set { name, value: v } => vec![format!("Set {name} {}", value(v))],
    }
}

pub fn write(tape: &Tape) -> String {
    let mut out = String::new();
    for command in &tape.commands {
        for text in line(command) {
            out.push_str(&text);
            out.push('\n');
        }
    }
    out
}
