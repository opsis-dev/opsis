use std::path::Path;

use serde_json::Value;

const NAME_SEPARATOR: &str = "__";
const PATH_FIELDS: [&str; 2] = ["path", "file_path"];
const PATTERN_FIELDS: [&str; 2] = ["pattern", "query"];
const TARGET_FIELDS: [&str; 7] = [
    "path",
    "file_path",
    "pattern",
    "query",
    "command",
    "url",
    "name",
];
const SCRIPT_FIELD: &str = "source";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Read,
    Edit,
    Search,
    Command,
    Script,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Asking,
    Running,
    Done,
    Failed,
}

#[derive(Debug, Clone, Copy)]
struct Verbs {
    action: &'static str,
    running: &'static str,
    done: &'static str,
    category: Category,
}

const fn verbs(
    action: &'static str,
    running: &'static str,
    done: &'static str,
    category: Category,
) -> Verbs {
    Verbs {
        action,
        running,
        done,
        category,
    }
}

const READ: Verbs = verbs("Read", "Reading", "Read", Category::Read);
const EDIT: Verbs = verbs("Edit", "Editing", "Edited", Category::Edit);
const SEARCH: Verbs = verbs("Search", "Searching", "Searched", Category::Search);
const RUN: Verbs = verbs("Run", "Running", "Ran", Category::Command);
const USE: Verbs = verbs("Use", "Using", "Used", Category::Other);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sentence {
    pub text: String,
    pub category: Category,
}

fn first_line(text: &str) -> &str {
    text.lines().next().unwrap_or_default().trim()
}

fn field<'a>(input: Option<&'a Value>, fields: &[&str]) -> Option<&'a str> {
    let input = input?;
    fields
        .iter()
        .find_map(|field| input.get(field).and_then(Value::as_str))
        .map(first_line)
        .filter(|text| !text.is_empty())
}

fn shown_path(path: &str, project: Option<&Path>) -> String {
    project
        .and_then(|project| Path::new(path).strip_prefix(project).ok())
        .filter(|relative| !relative.as_os_str().is_empty())
        .map_or_else(
            || path.to_owned(),
            |relative| relative.to_string_lossy().into_owned(),
        )
}

fn path_target(input: Option<&Value>, project: Option<&Path>) -> String {
    field(input, &PATH_FIELDS).map_or_else(String::new, |path| shown_path(path, project))
}

fn pattern_target(input: Option<&Value>) -> String {
    field(input, &PATTERN_FIELDS).map_or_else(String::new, |pattern| format!("for {pattern}"))
}

fn kind_verbs(kind: &str) -> Verbs {
    match kind {
        "read" => READ,
        "edit" => EDIT,
        "delete" => verbs("Delete", "Deleting", "Deleted", Category::Edit),
        "move" => verbs("Move", "Moving", "Moved", Category::Edit),
        "search" => SEARCH,
        "execute" => RUN,
        "fetch" => verbs("Fetch", "Fetching", "Fetched", Category::Other),
        _ => USE,
    }
}

fn spoken(name: &str) -> String {
    name.replace(['_', '-'], " ")
}

fn named(name: &str, kind: &str, input: Option<&Value>, project: Option<&Path>) -> (Verbs, String) {
    match name {
        "read" => (READ, path_target(input, project)),
        "write" => (
            verbs("Write", "Writing", "Wrote", Category::Edit),
            path_target(input, project),
        ),
        "edit" => (EDIT, path_target(input, project)),
        "glob" | "grep" => (SEARCH, pattern_target(input)),
        "shell" => (
            RUN,
            field(input, &["command"]).unwrap_or_default().to_owned(),
        ),
        "agent" => (
            verbs("Hand off", "Handing off", "Handed off", Category::Other),
            "a task".to_owned(),
        ),
        "run_lua" => (
            verbs("Run", "Running", "Ran", Category::Script),
            "Lua".to_owned(),
        ),
        "undo" => (
            verbs("Undo", "Undoing", "Undid", Category::Edit),
            "the last change".to_owned(),
        ),
        "redo" => (
            verbs("Redo", "Redoing", "Redid", Category::Edit),
            "the last change".to_owned(),
        ),
        "wait_run" => (
            verbs("Wait", "Waiting", "Waited", Category::Other),
            "for the run".to_owned(),
        ),
        _ => {
            let verbs = kind_verbs(kind);
            let target = if verbs.category == Category::Other && kind != "fetch" {
                spoken(name)
            } else if verbs.category == Category::Search {
                pattern_target(input)
            } else {
                field(input, &TARGET_FIELDS)
                    .map_or_else(String::new, |target| shown_path(target, project))
            };
            (verbs, target)
        }
    }
}

fn joined(verb: &str, target: &str) -> String {
    if target.is_empty() {
        verb.to_owned()
    } else {
        format!("{verb} {target}")
    }
}

pub fn sentence(
    kind: &str,
    title: &str,
    input: Option<&Value>,
    stage: Stage,
    project: Option<&Path>,
) -> Sentence {
    let title = title.trim();
    if title.contains(char::is_whitespace) {
        let text = project.map_or_else(
            || title.to_owned(),
            |project| {
                let prefix = format!("{}/", project.to_string_lossy());
                title.replace(&prefix, "")
            },
        );
        return Sentence {
            text,
            category: kind_verbs(kind).category,
        };
    }
    let name = title.rsplit(NAME_SEPARATOR).next().unwrap_or(title);
    let (verbs, target) = named(name, kind, input, project);
    let text = match stage {
        Stage::Asking => joined(verbs.action, &target),
        Stage::Running => joined(verbs.running, &target),
        Stage::Done => joined(verbs.done, &target),
        Stage::Failed => joined(
            &format!("Failed to {}", verbs.action.to_lowercase()),
            &target,
        ),
    };
    Sentence {
        text,
        category: verbs.category,
    }
}

pub fn script(input: Option<&Value>) -> String {
    input
        .and_then(|input| input.get(SCRIPT_FIELD))
        .and_then(Value::as_str)
        .map_or_else(String::new, |source| source.trim_end().to_owned())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use proptest::prelude::*;
    use serde_json::json;

    use super::*;

    const NAMES: [&str; 12] = [
        "read",
        "write",
        "edit",
        "glob",
        "grep",
        "shell",
        "run_lua",
        "undo",
        "wait_run",
        "opsis__run_lua",
        "list_things",
        "agent",
    ];
    const KINDS: [&str; 6] = ["read", "edit", "search", "execute", "fetch", "other"];
    const STAGES: [Stage; 4] = [Stage::Asking, Stage::Running, Stage::Done, Stage::Failed];

    proptest! {
        #[test]
        fn a_sentence_is_one_capitalised_line(
            name in 0..NAMES.len(),
            kind in 0..KINDS.len(),
            stage in 0..STAGES.len(),
            target in "[a-z/._ ]{0,30}",
        ) {
            let input = json!({ "path": target, "command": target, "pattern": target });
            let said = sentence(KINDS[kind], NAMES[name], Some(&input), STAGES[stage], None);
            prop_assert!(said.text.chars().next().is_some_and(char::is_uppercase), "{}", said.text);
            prop_assert!(!said.text.contains('\n'));
            prop_assert_eq!(said.text.starts_with("Failed to "), STAGES[stage] == Stage::Failed);
        }

        #[test]
        fn a_path_in_the_project_is_shown_from_the_project(
            parts in proptest::collection::vec("[a-z]{1,8}", 1..4),
        ) {
            let project = PathBuf::from("/work/project");
            let relative = parts.join("/");
            let input = json!({ "path": project.join(&relative).to_string_lossy() });
            let said = sentence("read", "read", Some(&input), Stage::Done, Some(&project));
            prop_assert_eq!(said.text, format!("Read {relative}"));
        }
    }
}
