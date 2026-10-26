use std::sync::LazyLock;

use indexmap::IndexMap;
use regex::{Captures, Regex};
use serde_json::{Map, Value};

use crate::text::project_folder_name;

pub const VAR_PATTERN: &str = r"\{\{\s*([^{}]+?)\s*\}\}";

static VAR: LazyLock<Regex> = LazyLock::new(|| Regex::new(VAR_PATTERN).expect("pattern"));

pub const PAUSE_MARK: char = '¦';
pub const SITE_ADDRESS_VAR: &str = "Site address";
pub const SITE_ADDRESS: &str = "{{Site address}}";
pub const PROJECT_NAME_VAR: &str = "Project name";

pub type Vars = IndexMap<String, String>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Text(String),
    Var(String),
    Pause,
}

fn plain(text: &str, pauses: bool, out: &mut Vec<Token>) {
    if !pauses {
        if !text.is_empty() {
            out.push(Token::Text(text.to_owned()));
        }
        return;
    }
    for (i, part) in text.split(PAUSE_MARK).enumerate() {
        if i > 0 {
            out.push(Token::Pause);
        }
        if !part.is_empty() {
            out.push(Token::Text(part.to_owned()));
        }
    }
}

pub fn tokenize(value: &str, pauses: bool) -> Vec<Token> {
    let mut out = Vec::new();
    let mut last = 0;
    for caps in VAR.captures_iter(value) {
        let whole = caps.get(0).expect("match");
        plain(&value[last..whole.start()], pauses, &mut out);
        out.push(Token::Var(caps[1].to_owned()));
        last = whole.end();
    }
    plain(&value[last..], pauses, &mut out);
    out
}

pub fn has_variable(text: &str) -> bool {
    VAR.is_match(text)
}

pub fn var_token(name: &str) -> String {
    format!("{{{{{name}}}}}")
}

pub fn strings_in<'a>(value: &'a Value, test: &dyn Fn(&str) -> bool, out: &mut Vec<&'a str>) {
    match value {
        Value::String(s) if test(s) => out.push(s),
        Value::Array(items) => items.iter().for_each(|v| strings_in(v, test, out)),
        Value::Object(map) => map.values().for_each(|v| strings_in(v, test, out)),
        _ => {}
    }
}

pub fn map_strings(value: &Value, f: &dyn Fn(&str) -> String) -> Value {
    match value {
        Value::String(s) => Value::String(f(s)),
        Value::Array(items) => Value::Array(items.iter().map(|v| map_strings(v, f)).collect()),
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(k, v)| (k.clone(), map_strings(v, f)))
                .collect(),
        ),
        other => other.clone(),
    }
}

pub fn interpolate_text(text: &str, vars: &Vars) -> String {
    VAR.replace_all(text, |caps: &Captures| {
        vars.get(&caps[1])
            .cloned()
            .unwrap_or_else(|| caps[0].to_owned())
    })
    .into_owned()
}

pub fn interpolate(value: &Value, vars: &Vars) -> Value {
    map_strings(value, &|text| interpolate_text(text, vars))
}

pub fn interpolate_map(map: &Map<String, Value>, vars: &Vars) -> Map<String, Value> {
    map.iter()
        .map(|(k, v)| (k.clone(), interpolate(v, vars)))
        .collect()
}

pub fn rename_variable(value: &Value, from: &str, to: &str) -> Value {
    map_strings(value, &|text| {
        VAR.replace_all(text, |caps: &Captures| {
            if &caps[1] == from {
                var_token(to)
            } else {
                caps[0].to_owned()
            }
        })
        .into_owned()
    })
}

pub fn names_for_unknown(value: &Value) -> Value {
    map_strings(value, &|text| VAR.replace_all(text, "$1").into_owned())
}

pub fn unknown_variables(value: &Value, vars: &Vars) -> Vec<String> {
    let mut texts = Vec::new();
    strings_in(value, &|_| true, &mut texts);
    let mut out: Vec<String> = Vec::new();
    for text in texts {
        for token in tokenize(text, false) {
            if let Token::Var(name) = token
                && !vars.contains_key(&name)
                && !out.contains(&name)
            {
                out.push(name);
            }
        }
    }
    out
}

pub fn workflow_vars(name: &str, variables: &IndexMap<String, String>) -> Vars {
    let mut vars = Vars::new();
    vars.insert(PROJECT_NAME_VAR.to_owned(), project_folder_name(name));
    for (k, v) in variables {
        vars.insert(k.clone(), v.clone());
    }
    vars
}
