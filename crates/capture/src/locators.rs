use std::sync::LazyLock;

use regex::Regex;

use crate::page_scripts::interactive_selector;

const NAMED_ROLES: [&str; 6] = ["button", "link", "textbox", "combobox", "option", "tab"];
const ANCESTOR_ROLES: [&str; 6] = ["button", "link", "menuitem", "tab", "option", "combobox"];
const ANCESTOR_TAGS: [&str; 2] = ["button", "a"];
const REGEX_SPECIALS: &str = ".*+?^${}()|[]\\";
pub const TEST_ID_ATTRIBUTE: &str = "data-testid";

static UNESCAPED_QUOTE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(^|[^\\])(\\\\)*(["'`])"#).expect("pattern"));

#[derive(Debug)]
pub struct TextPattern {
    source: String,
}

impl TextPattern {
    pub fn exact(text: &str) -> Self {
        Self {
            source: format!("^{}$", escape_regexp(text)),
        }
    }

    pub fn loose(text: &str) -> Self {
        Self {
            source: escape_regexp(text),
        }
    }

    fn literal(&self) -> String {
        format!("/{}/i", regexp_source(&self.source))
    }
}

pub fn escape_regexp(text: &str) -> String {
    text.chars().fold(String::new(), |mut out, c| {
        if REGEX_SPECIALS.contains(c) {
            out.push('\\');
        }
        out.push(c);
        out
    })
}

fn regexp_source(pattern: &str) -> String {
    let mut out = String::new();
    let mut chars = pattern.chars();
    let mut in_class = false;
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                out.push('\\');
                if let Some(next) = chars.next() {
                    out.push(next);
                }
            }
            '[' => {
                in_class = true;
                out.push(c);
            }
            ']' => {
                in_class = false;
                out.push(c);
            }
            '/' if !in_class => out.push_str("\\/"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            other => out.push(other),
        }
    }
    out
}

fn escape_regex_for_selector(pattern: &TextPattern) -> String {
    UNESCAPED_QUOTE
        .replace_all(&pattern.literal(), r"${1}${2}\${3}")
        .replace(">>", "\\>\\>")
}

fn json_text(text: &str) -> String {
    serde_json::to_string(text).expect("strings serialize")
}

pub fn text_string_selector(text: &str, exact: bool) -> String {
    format!(
        "internal:text={}{}",
        json_text(text),
        if exact { 's' } else { 'i' }
    )
}

fn attribute_value(value: &str, exact: bool) -> String {
    format!(
        "\"{}\"{}",
        value.replace('\\', "\\\\").replace('"', "\\\""),
        if exact { 's' } else { 'i' }
    )
}

pub fn role_selector(role: &str, name: &TextPattern) -> String {
    format!(
        "internal:role={role}[name={}]",
        escape_regex_for_selector(name)
    )
}

pub fn label_selector(text: &TextPattern) -> String {
    format!("internal:label={}", escape_regex_for_selector(text))
}

pub fn placeholder_selector(text: &TextPattern) -> String {
    format!(
        "internal:attr=[placeholder={}]",
        escape_regex_for_selector(text)
    )
}

pub fn test_id_selector(test_id: &str) -> String {
    format!(
        "internal:testid=[{TEST_ID_ATTRIBUTE}={}]",
        attribute_value(test_id, true)
    )
}

pub fn text_selector(text: &TextPattern) -> String {
    format!("internal:text={}", escape_regex_for_selector(text))
}

pub fn has_text(text: &TextPattern) -> String {
    escape_regex_for_selector(text)
}

fn interactive_ancestor() -> String {
    let tests: Vec<String> = ANCESTOR_ROLES
        .iter()
        .map(|r| format!("@role=\"{r}\""))
        .chain(ANCESTOR_TAGS.iter().map(|t| format!("self::{t}")))
        .collect();
    format!("xpath=ancestor-or-self::*[{}][1]", tests.join(" or "))
}

pub fn first(selector: &str) -> String {
    format!("{selector} >> nth=0")
}

#[derive(Debug)]
pub struct Rung {
    pub by: String,
    pub selector: String,
}

pub fn auto_candidates(name: &str) -> Vec<Rung> {
    let exact = TextPattern::exact(name);
    let loose = TextPattern::loose(name);
    let mut rungs: Vec<Rung> = NAMED_ROLES
        .iter()
        .map(|role| Rung {
            by: format!("role={role} name=\"{name}\""),
            selector: role_selector(role, &exact),
        })
        .collect();
    rungs.push(Rung {
        by: format!("label=\"{name}\""),
        selector: label_selector(&exact),
    });
    rungs.push(Rung {
        by: format!("placeholder~\"{name}\""),
        selector: placeholder_selector(&loose),
    });
    rungs.push(Rung {
        by: format!("testid=\"{name}\""),
        selector: test_id_selector(name),
    });
    rungs.push(Rung {
        by: format!("control containing \"{name}\""),
        selector: format!(
            "{} >> internal:has-text={}",
            interactive_selector(),
            has_text(&loose)
        ),
    });
    rungs.push(Rung {
        by: format!("text \"{name}\" -> control"),
        selector: format!("{} >> {}", text_selector(&loose), interactive_ancestor()),
    });
    rungs.push(Rung {
        by: format!("text \"{name}\""),
        selector: text_selector(&loose),
    });
    rungs
}
