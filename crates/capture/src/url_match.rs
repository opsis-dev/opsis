use indexmap::IndexMap;
use regex::Regex;
use url::Url;

const ESCAPED: [char; 14] = [
    '$', '^', '+', '.', '*', '(', ')', '|', '\\', '?', '{', '}', '[', ']',
];
const OPAQUE_SCHEMES: [&str; 5] = ["about:", "data:", "chrome:", "edge:", "file:"];

#[derive(Debug, thiserror::Error)]
pub enum GlobError {
    #[error("Invalid glob pattern {0}: nested '{{' is not supported")]
    Nested(String),
    #[error("Invalid glob pattern {0}: unmatched '}}'")]
    UnmatchedClose(String),
    #[error("Invalid glob pattern {0}: unmatched '{{'")]
    UnmatchedOpen(String),
    #[error(transparent)]
    Regex(#[from] regex::Error),
}

fn quoted(glob: &str) -> String {
    serde_json::to_string(glob).expect("strings serialize")
}

fn escaped(c: char) -> String {
    if ESCAPED.contains(&c) {
        format!("\\{c}")
    } else {
        c.to_string()
    }
}

pub fn glob_to_regex_pattern(glob: &str) -> Result<String, GlobError> {
    let chars: Vec<char> = glob.chars().collect();
    let mut tokens = vec!["^".to_owned()];
    let mut in_group = false;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '\\' && i + 1 < chars.len() {
            i += 1;
            tokens.push(escaped(chars[i]));
            i += 1;
            continue;
        }
        if c == '*' {
            let before = i.checked_sub(1).map(|b| chars[b]);
            let mut stars = 1;
            while chars.get(i + 1) == Some(&'*') {
                stars += 1;
                i += 1;
            }
            if stars > 1 {
                if chars.get(i + 1) == Some(&'/') {
                    tokens.push(
                        if before == Some('/') {
                            "((.+/)|)"
                        } else {
                            "(.*/)"
                        }
                        .to_owned(),
                    );
                    i += 1;
                } else {
                    tokens.push("(.*)".to_owned());
                }
            } else {
                tokens.push("([^/]*)".to_owned());
            }
            i += 1;
            continue;
        }
        match c {
            '{' => {
                if in_group {
                    return Err(GlobError::Nested(quoted(glob)));
                }
                in_group = true;
                tokens.push("(".to_owned());
            }
            '}' => {
                if !in_group {
                    return Err(GlobError::UnmatchedClose(quoted(glob)));
                }
                in_group = false;
                tokens.push(")".to_owned());
            }
            ',' => tokens.push(if in_group {
                "|".to_owned()
            } else {
                "\\,".to_owned()
            }),
            other => tokens.push(escaped(other)),
        }
        i += 1;
    }
    if in_group {
        return Err(GlobError::UnmatchedOpen(quoted(glob)));
    }
    tokens.push("$".to_owned());
    Ok(tokens.concat())
}

fn has_glob_chars(token: &str) -> bool {
    token
        .chars()
        .any(|c| matches!(c, '*' | '?' | '{' | '}' | '\\'))
}

fn resolve_glob_base(glob: &str) -> String {
    if glob.starts_with('*') {
        return glob.to_owned();
    }
    let glob = glob.replace("\\\\?", "?");
    if OPAQUE_SCHEMES.iter().any(|s| glob.starts_with(s)) {
        return glob;
    }
    let mut tokens: IndexMap<String, String> = IndexMap::new();
    let mut map_token = |original: &str, replacement: String| -> String {
        if original.is_empty() {
            return String::new();
        }
        tokens.insert(replacement.clone(), original.to_owned());
        replacement
    };
    let relative: Vec<String> = glob
        .split('/')
        .enumerate()
        .map(|(index, token)| {
            if matches!(token, "." | ".." | "") {
                return token.to_owned();
            }
            if index == 0 && token.ends_with(':') {
                return if token.contains('*') || token.contains('{') {
                    map_token(token, "http:".to_owned())
                } else {
                    token.to_owned()
                };
            }
            if !has_glob_chars(token) {
                return token.to_owned();
            }
            match token.find('?') {
                None => map_token(token, format!("$_{index}_$")),
                Some(q) => {
                    let prefix = map_token(&token[..q], format!("$_{index}_$"));
                    let suffix = map_token(&token[q..], format!("?$_{index}_$"));
                    prefix + &suffix
                }
            }
        })
        .collect();
    let relative = relative.join("/");
    let (mut resolved, origin) = match Url::parse(&relative) {
        Ok(url) => (url.to_string(), Some(url.origin().ascii_serialization())),
        Err(_) => (relative, None),
    };
    for (token, original) in tokens {
        let lower = origin.as_deref().is_some_and(|o| o.contains(&token));
        let replacement = if lower {
            original.to_lowercase()
        } else {
            original
        };
        resolved = resolved.replacen(&token, &replacement, 1);
    }
    resolved
}

pub fn url_matches(url: &str, glob: &str) -> Result<bool, GlobError> {
    if glob.is_empty() {
        return Ok(true);
    }
    let pattern = glob_to_regex_pattern(&resolve_glob_base(glob))?;
    let regex = Regex::new(&pattern)?;
    Ok(regex.is_match(url))
}
