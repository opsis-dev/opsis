use std::path::{Path, PathBuf};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use globset::GlobBuilder;
use grep_regex::RegexMatcher;
use grep_searcher::Searcher;
use grep_searcher::sinks::UTF8;
use ignore::WalkBuilder;
use opsis_folders::slashed;
use rig_core::completion::ToolDefinition;
use rig_core::message::{DocumentSourceKind, Image, ImageMediaType, ToolName, ToolResultContent};
use serde_json::{Map, Value, json};

pub const READ: &str = "read";
pub const WRITE: &str = "write";
pub const EDIT: &str = "edit";
pub const GLOB: &str = "glob";
pub const GREP: &str = "grep";
pub const LISTED_AT_MOST: usize = 200;
const READ_LINES: usize = 2000;

#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("{0}")]
    Refused(String),
    #[error("{path}: {error}")]
    Io { path: String, error: std::io::Error },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    Reads,
    Changes,
}

pub fn effect(name: &str) -> Effect {
    if [READ, GLOB, GREP].contains(&name) {
        Effect::Reads
    } else {
        Effect::Changes
    }
}

fn definition(name: &str, description: &str, parameters: Value) -> Option<ToolDefinition> {
    Some(ToolDefinition {
        name: ToolName::new(name).ok()?,
        description: description.to_owned(),
        parameters,
        annotations: rig_core::completion::ToolAnnotations::default(),
    })
}

pub fn definitions() -> Vec<ToolDefinition> {
    [
        definition(
            READ,
            "Read a file. Text comes back with line numbers; an image comes back as a picture. Paths are relative to the session's folder.",
            json!({ "type": "object", "properties": {
                "path": { "type": "string" },
                "offset": { "type": "integer", "minimum": 1, "description": "The first line to read." },
                "limit": { "type": "integer", "minimum": 1, "description": "How many lines to read." }
            }, "required": ["path"] }),
        ),
        definition(
            WRITE,
            "Write a whole file, making its folders. Never write the project file project.opsis.json; change the project through run_lua.",
            json!({ "type": "object", "properties": {
                "path": { "type": "string" },
                "content": { "type": "string" }
            }, "required": ["path", "content"] }),
        ),
        definition(
            EDIT,
            "Replace exact text in a file. The old text must appear once, unless all is true.",
            json!({ "type": "object", "properties": {
                "path": { "type": "string" },
                "old": { "type": "string" },
                "new": { "type": "string" },
                "all": { "type": "boolean" }
            }, "required": ["path", "old", "new"] }),
        ),
        definition(
            GLOB,
            "List files matching a glob such as **/*.lua, skipping what .gitignore leaves out.",
            json!({ "type": "object", "properties": {
                "pattern": { "type": "string" },
                "path": { "type": "string", "description": "The folder to search; the session's folder by default." }
            }, "required": ["pattern"] }),
        ),
        definition(
            GREP,
            "Search file contents with a regular expression and list file:line:text for each match.",
            json!({ "type": "object", "properties": {
                "pattern": { "type": "string" },
                "path": { "type": "string" },
                "glob": { "type": "string", "description": "Only files matching this glob." }
            }, "required": ["pattern"] }),
        ),
    ]
    .into_iter()
    .flatten()
    .collect()
}

pub fn is_local(name: &str) -> bool {
    [READ, WRITE, EDIT, GLOB, GREP].contains(&name)
}

fn text<'a>(arguments: &'a Map<String, Value>, name: &str) -> Result<&'a str, ToolError> {
    arguments
        .get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| ToolError::Refused(format!("{name} is needed, as a string.")))
}

pub fn resolve(folder: &Path, path: &str) -> PathBuf {
    let path = Path::new(path);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        folder.join(path)
    }
}

fn io_error(path: &Path) -> impl FnOnce(std::io::Error) -> ToolError + '_ {
    move |error| ToolError::Io {
        path: slashed(path),
        error,
    }
}

fn image_type(path: &Path) -> Option<(ImageMediaType, &'static str)> {
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    match extension.as_str() {
        "png" => Some((ImageMediaType::PNG, "image/png")),
        "jpg" | "jpeg" => Some((ImageMediaType::JPEG, "image/jpeg")),
        "gif" => Some((ImageMediaType::GIF, "image/gif")),
        "webp" => Some((ImageMediaType::WEBP, "image/webp")),
        _ => None,
    }
}

pub fn numbered(text: &str, offset: usize, limit: usize) -> String {
    text.lines()
        .enumerate()
        .skip(offset.saturating_sub(1))
        .take(limit)
        .map(|(index, line)| format!("{:>6}\t{line}", index + 1))
        .collect::<Vec<_>>()
        .join("\n")
}

fn read(
    folder: &Path,
    arguments: &Map<String, Value>,
) -> Result<Vec<ToolResultContent>, ToolError> {
    let path = resolve(folder, text(arguments, "path")?);
    let bytes = std::fs::read(&path).map_err(io_error(&path))?;
    if let Some((media_type, _)) = image_type(&path) {
        return Ok(vec![ToolResultContent::Image(Image {
            data: DocumentSourceKind::Base64(STANDARD.encode(bytes)),
            media_type: Some(media_type),
            ..Image::default()
        })]);
    }
    let text = String::from_utf8(bytes)
        .map_err(|_| ToolError::Refused(format!("{} isn't text or a picture.", slashed(&path))))?;
    let offset = arguments
        .get("offset")
        .and_then(Value::as_u64)
        .map_or(1, |v| usize::try_from(v).unwrap_or(1));
    let limit = arguments
        .get("limit")
        .and_then(Value::as_u64)
        .map_or(READ_LINES, |v| usize::try_from(v).unwrap_or(READ_LINES));
    Ok(vec![ToolResultContent::text(numbered(
        &text, offset, limit,
    ))])
}

fn write(folder: &Path, arguments: &Map<String, Value>) -> Result<String, ToolError> {
    let path = resolve(folder, text(arguments, "path")?);
    let content = text(arguments, "content")?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(io_error(parent))?;
    }
    std::fs::write(&path, content).map_err(io_error(&path))?;
    Ok(format!(
        "Wrote {} ({} bytes).",
        slashed(&path),
        content.len()
    ))
}

pub fn replaced(original: &str, old: &str, new: &str, all: bool) -> Result<String, String> {
    if old.is_empty() {
        return Err("old can't be empty.".to_owned());
    }
    match original.matches(old).count() {
        0 => Err("old doesn't appear in the file.".to_owned()),
        1 => Ok(original.replacen(old, new, 1)),
        count if all => {
            let _ = count;
            Ok(original.replace(old, new))
        }
        count => Err(format!(
            "old appears {count} times; make it unique or set all."
        )),
    }
}

fn edit(folder: &Path, arguments: &Map<String, Value>) -> Result<String, ToolError> {
    let path = resolve(folder, text(arguments, "path")?);
    let original = std::fs::read_to_string(&path).map_err(io_error(&path))?;
    let all = arguments
        .get("all")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let changed = replaced(
        &original,
        text(arguments, "old")?,
        text(arguments, "new")?,
        all,
    )
    .map_err(ToolError::Refused)?;
    std::fs::write(&path, changed).map_err(io_error(&path))?;
    Ok(format!("Edited {}.", slashed(&path)))
}

fn search_root(folder: &Path, arguments: &Map<String, Value>) -> PathBuf {
    arguments
        .get("path")
        .and_then(Value::as_str)
        .map_or_else(|| folder.to_path_buf(), |path| resolve(folder, path))
}

fn glob_matcher(pattern: &str) -> Result<globset::GlobMatcher, ToolError> {
    GlobBuilder::new(pattern)
        .literal_separator(true)
        .build()
        .map(|glob| glob.compile_matcher())
        .map_err(|e| ToolError::Refused(format!("{pattern} isn't a glob: {e}")))
}

fn walk(root: &Path) -> impl Iterator<Item = PathBuf> {
    WalkBuilder::new(root)
        .build()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_some_and(|t| t.is_file()))
        .map(ignore::DirEntry::into_path)
}

fn shown(root: &Path, path: &Path) -> String {
    slashed(path.strip_prefix(root).unwrap_or(path))
}

fn glob(folder: &Path, arguments: &Map<String, Value>) -> Result<String, ToolError> {
    let root = search_root(folder, arguments);
    let matcher = glob_matcher(text(arguments, "pattern")?)?;
    let mut found: Vec<String> = walk(&root)
        .filter(|path| matcher.is_match(path.strip_prefix(&root).unwrap_or(path)))
        .map(|path| shown(&root, &path))
        .collect();
    found.sort();
    Ok(listed(found))
}

fn listed(mut lines: Vec<String>) -> String {
    if lines.is_empty() {
        return "Nothing matched.".to_owned();
    }
    let more = lines.len().saturating_sub(LISTED_AT_MOST);
    lines.truncate(LISTED_AT_MOST);
    if more > 0 {
        lines.push(format!("… and {more} more."));
    }
    lines.join("\n")
}

fn grep(folder: &Path, arguments: &Map<String, Value>) -> Result<String, ToolError> {
    let root = search_root(folder, arguments);
    let pattern = text(arguments, "pattern")?;
    let matcher = RegexMatcher::new(pattern)
        .map_err(|e| ToolError::Refused(format!("{pattern} isn't a regular expression: {e}")))?;
    let only = arguments
        .get("glob")
        .and_then(Value::as_str)
        .map(glob_matcher)
        .transpose()?;
    let mut lines = Vec::new();
    for path in walk(&root) {
        let relative = path.strip_prefix(&root).unwrap_or(&path).to_path_buf();
        if only.as_ref().is_some_and(|m| !m.is_match(&relative)) {
            continue;
        }
        let name = slashed(&relative);
        let searched = Searcher::new().search_path(
            &matcher,
            &path,
            UTF8(|line, text| {
                lines.push(format!("{name}:{line}:{}", text.trim_end()));
                Ok(lines.len() <= LISTED_AT_MOST)
            }),
        );
        if searched.is_err() {
            continue;
        }
        if lines.len() > LISTED_AT_MOST {
            break;
        }
    }
    Ok(listed(lines))
}

pub fn run(
    folder: &Path,
    name: &str,
    arguments: &Map<String, Value>,
) -> Result<Vec<ToolResultContent>, ToolError> {
    let text_result =
        |result: Result<String, ToolError>| result.map(|t| vec![ToolResultContent::text(t)]);
    match name {
        READ => read(folder, arguments),
        WRITE => text_result(write(folder, arguments)),
        EDIT => text_result(edit(folder, arguments)),
        GLOB => text_result(glob(folder, arguments)),
        GREP => text_result(grep(folder, arguments)),
        _ => Err(ToolError::Refused(format!("There is no tool {name}."))),
    }
}

pub const fn image_mime(content: &ToolResultContent) -> Option<(&str, &'static str)> {
    match content {
        ToolResultContent::Image(Image {
            data: DocumentSourceKind::Base64(data),
            media_type,
            ..
        }) => {
            let mime = match media_type {
                Some(ImageMediaType::JPEG) => "image/jpeg",
                Some(ImageMediaType::GIF) => "image/gif",
                Some(ImageMediaType::WEBP) => "image/webp",
                _ => "image/png",
            };
            Some((data.as_str(), mime))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        #[test]
        fn an_edit_of_unique_text_changes_only_it(before in "[a-z ]{0,20}", old in "[A-Z]{3,6}", after in "[a-z ]{0,20}", new in "[0-9]{0,6}") {
            let original = format!("{before}{old}{after}");
            prop_assume!(original.matches(old.as_str()).count() == 1);
            prop_assert_eq!(replaced(&original, &old, &new, false).unwrap(), format!("{before}{new}{after}"));
        }

        #[test]
        fn repeated_text_is_refused_unless_all(old in "[A-Z]{2,4}", times in 2_usize..5) {
            let original = old.repeat(times);
            prop_assert!(replaced(&original, &old, "x", false).is_err());
            prop_assert_eq!(replaced(&original, &old, "x", true).unwrap(), "x".repeat(times));
        }

        #[test]
        fn numbering_keeps_each_line_and_counts_from_the_offset(lines in proptest::collection::vec("[a-z]{0,6}", 1..20), offset in 1_usize..10) {
            let text = lines.join("\n");
            let shown = numbered(&text, offset, 1000);
            let expected = text.lines().count().saturating_sub(offset - 1);
            prop_assert_eq!(shown.lines().count(), expected);
            if let Some(first) = shown.lines().next() {
                prop_assert!(first.trim_start().starts_with(&offset.to_string()));
            }
        }
    }

    #[test]
    fn reads_writes_finds_and_searches_in_the_session_folder() {
        let dir = tempfile::tempdir().unwrap();
        let args = |value: Value| match value {
            Value::Object(map) => map,
            _ => Map::new(),
        };
        run(
            dir.path(),
            WRITE,
            &args(json!({ "path": "scripts/a.lua", "content": "return 1\nlocal two = 2\n" })),
        )
        .unwrap();
        run(
            dir.path(),
            EDIT,
            &args(json!({ "path": "scripts/a.lua", "old": "two", "new": "three" })),
        )
        .unwrap();
        let read = run(dir.path(), READ, &args(json!({ "path": "scripts/a.lua" }))).unwrap();
        assert!(read[0].as_text().unwrap().contains("local three = 2"));
        let globbed = run(dir.path(), GLOB, &args(json!({ "pattern": "**/*.lua" }))).unwrap();
        assert_eq!(globbed[0].as_text(), Some("scripts/a.lua"));
        let found = run(dir.path(), GREP, &args(json!({ "pattern": "three" }))).unwrap();
        assert_eq!(found[0].as_text(), Some("scripts/a.lua:2:local three = 2"));
        std::fs::write(dir.path().join("shot.png"), [137, 80, 78, 71]).unwrap();
        let picture = run(dir.path(), READ, &args(json!({ "path": "shot.png" }))).unwrap();
        assert!(image_mime(&picture[0]).is_some());
    }
}
