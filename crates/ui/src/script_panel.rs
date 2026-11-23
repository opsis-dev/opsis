use std::collections::HashMap;
use std::path::{Path, PathBuf};

use notify_debouncer_mini::notify::event::ModifyKind;
use notify_debouncer_mini::notify::{
    self, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher, recommended_watcher,
};
use opsis_qt::{
    BracketView, Choice, EventKind as PanelEvent, ScriptLineView, ScriptLookView, ScriptView,
    SyntaxRangeView, TextChangeView,
};
use opsis_script::editing::{self, TextChange};
use opsis_script::runtime::location;
use opsis_script::syntax::highlight;
use opsis_script::themes::{Theme, theme, themes};
use opsis_session::script::{ScriptState, list_scripts, script_path};
use opsis_types::script::{SCRIPT_EXTENSION, SCRIPTS_DIR, ScriptLine};
use opsis_types::settings::{SCRIPT_FONT_SIZE, ScriptSection};

use crate::ui::plain;

const STARTER: &str = "print(opsis.project.name)\n";
const NEW_NAME: &str = "script";

#[derive(Default)]
pub struct ScriptEditing {
    pub dir: Option<PathBuf>,
    pub open: Option<String>,
    saved: HashMap<String, String>,
    watch: Option<RecommendedWatcher>,
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

    pub fn forget(&mut self, file: &str) {
        self.saved.remove(file);
        if self.open.as_deref() == Some(file) {
            self.open = None;
        }
    }

    pub fn moved(&mut self, from: &str, to: &str) {
        if let Some(text) = self.saved.remove(from) {
            self.saved.insert(to.to_owned(), text);
        }
        if self.open.as_deref() == Some(from) {
            self.open = Some(to.to_owned());
        }
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

pub const fn changes_files(kind: EventKind) -> bool {
    matches!(
        kind,
        EventKind::Create(_)
            | EventKind::Remove(_)
            | EventKind::Modify(ModifyKind::Data(_) | ModifyKind::Name(_) | ModifyKind::Any)
    )
}

fn watch(dir: &Path) -> Option<RecommendedWatcher> {
    let scripts = dir.join(SCRIPTS_DIR);
    if !scripts.is_dir() {
        return None;
    }
    let mut watcher = recommended_watcher(|result: notify::Result<Event>| {
        if result.is_ok_and(|event| changes_files(event.kind)) {
            opsis_qt::post_event(PanelEvent::Script);
        }
    })
    .map_err(|error| log::warn!("watching the scripts folder: {error}"))
    .ok()?;
    watcher
        .watch(&scripts, RecursiveMode::Recursive)
        .map_err(|error| log::warn!("watching the scripts folder: {error}"))
        .ok()?;
    Some(watcher)
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

pub fn renamed(name: &str) -> String {
    let name = name.trim().replace('\\', "/");
    let name = name
        .strip_prefix(&format!("{SCRIPTS_DIR}/"))
        .unwrap_or(&name)
        .to_owned();
    if Path::new(&name)
        .extension()
        .is_some_and(|e| e == SCRIPT_EXTENSION)
    {
        format!("{SCRIPTS_DIR}/{name}")
    } else {
        format!("{SCRIPTS_DIR}/{name}.{SCRIPT_EXTENSION}")
    }
}

pub fn rename(dir: &Path, from: &str, to: &str) -> Result<(), String> {
    let target = dir.join(to);
    if target.exists() {
        return Err(format!("There's already a script called {to}."));
    }
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Couldn't rename {from}. {e}"))?;
    }
    std::fs::rename(dir.join(from), &target).map_err(|e| format!("Couldn't rename {from}. {e}"))
}

fn trash_context() -> trash::TrashContext {
    #[cfg_attr(
        not(target_os = "macos"),
        expect(unused_mut, reason = "only macOS chooses how files reach the trash")
    )]
    let mut context = trash::TrashContext::default();
    #[cfg(target_os = "macos")]
    {
        use trash::macos::{DeleteMethod, TrashContextExtMacos};
        context.set_delete_method(DeleteMethod::NsFileManager);
    }
    context
}

pub fn discard(dir: &Path, file: &str) -> Result<(), String> {
    trash_context()
        .delete(dir.join(file))
        .map_err(|e| format!("Couldn't move {file} to the trash. {e}"))
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

pub fn ranges(text: &str, theme: &Theme) -> Vec<SyntaxRangeView> {
    highlight(text)
        .into_iter()
        .filter_map(|range| {
            let style = range.capture.and_then(|capture| theme.style(capture));
            (style.is_some() || range.error).then(|| SyntaxRangeView {
                start: range.start,
                length: range.length,
                color: style.map_or_default(|s| s.fg.clone()),
                bold: style.is_some_and(|s| s.bold),
                italic: style.is_some_and(|s| s.italic),
                error: range.error,
            })
        })
        .collect()
}

pub fn look(settings: &ScriptSection) -> ScriptLookView {
    let theme = theme(&settings.theme);
    ScriptLookView {
        theme: theme.name.clone(),
        background: theme.background.clone(),
        text: theme.text.clone(),
        selection: theme.selection.clone(),
        bracket: theme.bracket.clone(),
        error: theme.error.clone(),
        warning: theme.warning.clone(),
        font_size: settings.font_size,
        font_min: SCRIPT_FONT_SIZE.min,
        font_max: SCRIPT_FONT_SIZE.max,
    }
}

pub fn theme_choices() -> Vec<Choice> {
    themes()
        .iter()
        .map(|theme| Choice {
            value: theme.name.clone(),
            label: theme.label.clone(),
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

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        #[test]
        fn a_new_name_always_lands_in_the_scripts_folder_as_lua(name in "[a-z][a-z0-9-]{0,8}(\\.lua)?") {
            let file = renamed(&name);
            prop_assert!(script_path(&file).is_some(), "{file}");
            prop_assert_eq!(renamed(&file), file);
        }
    }

    #[test]
    fn only_writes_new_files_renames_and_removals_reload_the_list() {
        use notify::event::{
            AccessKind, AccessMode, CreateKind, DataChange, MetadataKind, RemoveKind, RenameMode,
        };
        assert!(!changes_files(EventKind::Access(AccessKind::Open(
            AccessMode::Any
        ))));
        assert!(!changes_files(EventKind::Access(AccessKind::Close(
            AccessMode::Read
        ))));
        assert!(!changes_files(EventKind::Modify(ModifyKind::Metadata(
            MetadataKind::AccessTime
        ))));
        assert!(changes_files(EventKind::Create(CreateKind::File)));
        assert!(changes_files(EventKind::Remove(RemoveKind::File)));
        assert!(changes_files(EventKind::Modify(ModifyKind::Data(
            DataChange::Content
        ))));
        assert!(changes_files(EventKind::Modify(ModifyKind::Name(
            RenameMode::Both
        ))));
    }

    #[test]
    fn renames_a_script_and_refuses_to_overwrite_one() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(SCRIPTS_DIR)).unwrap();
        std::fs::write(dir.path().join("scripts/a.lua"), "print(1)").unwrap();
        std::fs::write(dir.path().join("scripts/b.lua"), "print(2)").unwrap();
        assert!(rename(dir.path(), "scripts/a.lua", "scripts/b.lua").is_err());
        rename(dir.path(), "scripts/a.lua", "scripts/c.lua").unwrap();
        assert_eq!(
            std::fs::read_to_string(dir.path().join("scripts/c.lua")).unwrap(),
            "print(1)"
        );
        assert!(!dir.path().join("scripts/a.lua").exists());
    }
}
