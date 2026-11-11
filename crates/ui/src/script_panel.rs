use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use notify_debouncer_mini::notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{DebounceEventResult, Debouncer, new_debouncer};
use opsis_qt::{
    BracketView, EventKind, ScriptLineView, ScriptView, SyntaxRangeView, TextChangeView,
};
use opsis_script::editing::{self, TextChange};
use opsis_script::runtime::location;
use opsis_script::syntax::highlight;
use opsis_session::script::{ScriptState, list_scripts, script_path};
use opsis_types::script::{SCRIPT_EXTENSION, SCRIPTS_DIR, ScriptLine};

use crate::ui::plain;

const WATCH_DELAY: Duration = Duration::from_millis(300);
const STARTER: &str = "print(opsis.project.name)\n";
const NEW_NAME: &str = "script";

#[derive(Default)]
pub struct ScriptEditing {
    pub dir: Option<PathBuf>,
    pub open: Option<String>,
    saved: HashMap<String, String>,
    watch: Option<Debouncer<RecommendedWatcher>>,
}

impl std::fmt::Debug for ScriptEditing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScriptEditing")
            .field("dir", &self.dir)
            .field("open", &self.open)
            .finish_non_exhaustive()
    }
}

impl ScriptEditing {
    pub fn follow(&mut self, dir: &Path, open: Option<String>) {
        if self.dir.as_deref() == Some(dir) {
            if self.watch.is_none() {
                self.watch = watch(dir);
            }
            return;
        }
        self.dir = Some(dir.to_path_buf());
        self.open = open;
        self.saved.clear();
        self.watch = watch(dir);
    }

    pub fn remember(&mut self, file: &str, text: &str) {
        self.saved.insert(file.to_owned(), text.to_owned());
    }

    pub fn reload(&mut self, dir: &Path, file: &str, text: &str) -> Option<String> {
        if self.saved.get(file).map(String::as_str) != Some(text) {
            return None;
        }
        let disk = std::fs::read_to_string(dir.join(file)).ok()?;
        if disk == text {
            return None;
        }
        self.saved.insert(file.to_owned(), disk.clone());
        Some(disk)
    }
}

fn watch(dir: &Path) -> Option<Debouncer<RecommendedWatcher>> {
    let scripts = dir.join(SCRIPTS_DIR);
    if !scripts.is_dir() {
        return None;
    }
    let mut debouncer = new_debouncer(WATCH_DELAY, |result: DebounceEventResult| {
        if result.is_ok() {
            opsis_qt::post_event(EventKind::Script);
        }
    })
    .map_err(|error| log::warn!("watching the scripts folder: {error}"))
    .ok()?;
    debouncer
        .watcher()
        .watch(&scripts, RecursiveMode::Recursive)
        .map_err(|error| log::warn!("watching the scripts folder: {error}"))
        .ok()?;
    Some(debouncer)
}

fn line_view(line: &ScriptLine) -> ScriptLineView {
    let (file, at) = location(&line.text).unwrap_or_default();
    ScriptLineView {
        id: line.id,
        kind: plain(&line.kind),
        text: line.text.clone(),
        file,
        line: at,
    }
}

pub fn view(dir: Option<&Path>, open: Option<&str>, state: &ScriptState) -> ScriptView {
    let files = dir.map_or_default(list_scripts);
    let open = open
        .filter(|file| files.iter().any(|f| f == file))
        .or_else(|| files.first().map(String::as_str))
        .unwrap_or_default()
        .to_owned();
    let last = state.last.as_ref().map(|(_, outcome)| outcome);
    let failure = last.and_then(|o| o.failure.as_ref());
    ScriptView {
        has_project: dir.is_some(),
        dir: dir.map_or_default(opsis_engine::tools::shown),
        files,
        open,
        running: state.running.is_some(),
        running_file: state.running.clone().unwrap_or_default(),
        output: state.output.iter().map(line_view).collect(),
        outcome: match last {
            None => String::new(),
            Some(o) if o.ok => "ok".to_owned(),
            Some(o) if o.stopped => "stopped".to_owned(),
            Some(_) => "failed".to_owned(),
        },
        failure: failure.map_or_default(|f| f.message.clone()),
        failure_file: failure.map_or_default(|f| f.file.clone()),
        failure_line: failure.and_then(|f| f.line).unwrap_or(0),
        rolled_back: last.is_some_and(|o| o.rolled_back),
    }
}

pub fn checked(file: &str) -> Result<String, String> {
    script_path(file).ok_or_else(|| {
        format!("Scripts live in the project's {SCRIPTS_DIR} folder as .{SCRIPT_EXTENSION} files.")
    })
}

pub fn create(dir: &Path) -> Result<String, String> {
    let folder = dir.join(SCRIPTS_DIR);
    std::fs::create_dir_all(&folder)
        .map_err(|e| format!("Couldn't make the scripts folder. {e}"))?;
    let name = (1..=u32::MAX)
        .map(|n| {
            if n == 1 {
                format!("{NEW_NAME}.{SCRIPT_EXTENSION}")
            } else {
                format!("{NEW_NAME}-{n}.{SCRIPT_EXTENSION}")
            }
        })
        .find(|name| !folder.join(name).exists())
        .unwrap_or_default();
    std::fs::write(folder.join(&name), STARTER)
        .map_err(|e| format!("Couldn't make the script. {e}"))?;
    Ok(format!("{SCRIPTS_DIR}/{name}"))
}

pub const fn starter() -> &'static str {
    STARTER
}

pub fn ranges(text: &str) -> Vec<SyntaxRangeView> {
    highlight(text)
        .into_iter()
        .map(|range| SyntaxRangeView {
            start: range.start,
            length: range.length,
            kind: range.kind.name().to_owned(),
        })
        .collect()
}

fn change_view(change: Option<TextChange>) -> TextChangeView {
    change.map_or(
        TextChangeView {
            handled: false,
            start: 0,
            end: 0,
            text: String::new(),
            anchor: 0,
            cursor: 0,
        },
        |change| TextChangeView {
            handled: true,
            start: change.start,
            end: change.end,
            text: change.text,
            anchor: change.anchor,
            cursor: change.cursor,
        },
    )
}

pub fn reloaded(text: Option<String>) -> TextChangeView {
    change_view(text.map(|text| TextChange {
        start: 0,
        end: 0,
        text,
        anchor: 0,
        cursor: 0,
    }))
}

pub fn key(text: &str, anchor: u32, cursor: u32, key: &str) -> TextChangeView {
    let selected = anchor != cursor;
    let change = match key {
        "enter" if !selected => Some(editing::enter(text, cursor)),
        "backspace" if !selected => editing::backspace(text, cursor),
        "tab" => Some(editing::indent_lines(text, anchor, cursor, false)),
        "backtab" => Some(editing::indent_lines(text, anchor, cursor, true)),
        "comment" => Some(editing::toggle_comment(text, anchor, cursor)),
        typed if !selected => {
            let mut chars = typed.chars();
            match (chars.next(), chars.next()) {
                (Some(ch), None) => Some(editing::typed(text, cursor, ch)),
                _ => None,
            }
        }
        _ => None,
    };
    change_view(change)
}

pub fn bracket(text: &str, cursor: u32) -> BracketView {
    editing::matching_bracket(text, cursor).map_or(
        BracketView {
            found: false,
            open: 0,
            close: 0,
        },
        |(open, close)| BracketView {
            found: true,
            open,
            close,
        },
    )
}
