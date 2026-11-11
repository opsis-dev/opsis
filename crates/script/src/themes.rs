use std::collections::HashMap;
use std::sync::LazyLock;

use opsis_types::settings::DEFAULT_SCRIPT_THEME;
use serde::Deserialize;

const THEMES: &str = include_str!("themes.generated.json");

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Style {
    pub fg: String,
    #[serde(default)]
    pub bold: bool,
    #[serde(default)]
    pub italic: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Theme {
    pub name: String,
    pub label: String,
    pub background: String,
    pub text: String,
    pub selection: String,
    pub bracket: String,
    pub error: String,
    pub warning: String,
    pub styles: HashMap<String, Style>,
}

static ALL: LazyLock<Vec<Theme>> =
    LazyLock::new(|| serde_json::from_str(THEMES).expect("the generated script themes"));

pub fn themes() -> &'static [Theme] {
    &ALL
}

pub fn theme(name: &str) -> &'static Theme {
    ALL.iter()
        .find(|theme| theme.name == name)
        .or_else(|| ALL.iter().find(|theme| theme.name == DEFAULT_SCRIPT_THEME))
        .unwrap_or(&ALL[0])
}

impl Theme {
    pub fn style(&self, capture: &str) -> Option<&Style> {
        let mut scope = capture;
        loop {
            if let Some(style) = self.styles.get(scope) {
                return Some(style);
            }
            scope = &scope[..scope.rfind('.')?];
        }
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    fn is_hex(colour: &str) -> bool {
        colour.len() == 7
            && colour.starts_with('#')
            && colour[1..].chars().all(|c| c.is_ascii_hexdigit())
    }

    #[test]
    fn every_theme_has_readable_colours_for_every_lua_capture() {
        assert_eq!(themes()[0].name, DEFAULT_SCRIPT_THEME);
        for theme in themes() {
            for colour in [
                &theme.background,
                &theme.text,
                &theme.selection,
                &theme.bracket,
                &theme.error,
                &theme.warning,
            ] {
                assert!(is_hex(colour), "{}: {colour}", theme.name);
            }
            assert_ne!(theme.background, theme.text, "{}", theme.name);
            for capture in crate::syntax::captures() {
                if let Some(style) = theme.style(capture) {
                    assert!(is_hex(&style.fg), "{} {capture}: {}", theme.name, style.fg);
                }
            }
        }
    }

    #[test]
    fn an_unknown_theme_falls_back_to_the_default() {
        assert_eq!(theme("no such theme").name, DEFAULT_SCRIPT_THEME);
    }

    proptest! {
        #[test]
        fn the_longest_styled_scope_wins(parts in proptest::collection::vec("[a-z]{1,4}", 1..5), styled in 0usize..5) {
            let capture = parts.join(".");
            let depth = styled.min(parts.len() - 1);
            let key = parts[..=depth].join(".");
            let mut styles = HashMap::new();
            styles.insert(parts[0].clone(), Style { fg: "#000000".into(), bold: false, italic: false });
            styles.insert(key, Style { fg: "#ffffff".into(), bold: false, italic: false });
            let theme = Theme {
                name: "t".into(),
                label: "T".into(),
                background: "#000000".into(),
                text: "#ffffff".into(),
                selection: "#333333".into(),
                bracket: "#444444".into(),
                error: "#ff0000".into(),
                warning: "#ffff00".into(),
                styles,
            };
            prop_assert_eq!(theme.style(&capture).map(|s| s.fg.as_str()), Some("#ffffff"));
            prop_assert_eq!(theme.style(&format!("{capture}.extra")).map(|s| s.fg.as_str()), Some("#ffffff"));
        }
    }
}
