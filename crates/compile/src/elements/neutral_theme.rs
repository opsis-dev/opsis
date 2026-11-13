use indexmap::IndexMap;
use serde_json::{Map, Value};

use crate::elements::theme::{Theme, ThemeMode};

pub const LIGHT: [(&str, &str); 20] = [
    ("background", "oklch(1 0 0)"),
    ("foreground", "oklch(0.145 0 0)"),
    ("card", "oklch(1 0 0)"),
    ("cardForeground", "oklch(0.145 0 0)"),
    ("popover", "oklch(1 0 0)"),
    ("popoverForeground", "oklch(0.145 0 0)"),
    ("primary", "oklch(0.205 0 0)"),
    ("primaryForeground", "oklch(0.985 0 0)"),
    ("secondary", "oklch(0.97 0 0)"),
    ("secondaryForeground", "oklch(0.205 0 0)"),
    ("muted", "oklch(0.97 0 0)"),
    ("mutedForeground", "oklch(0.556 0 0)"),
    ("accent", "oklch(0.97 0 0)"),
    ("accentForeground", "oklch(0.205 0 0)"),
    ("destructive", "oklch(0.577 0.245 27.325)"),
    ("destructiveForeground", "oklch(0.985 0 0)"),
    ("border", "oklch(0.922 0 0)"),
    ("input", "oklch(0.922 0 0)"),
    ("ring", "oklch(0.708 0 0)"),
    ("success", "oklch(0.6 0.17 150)"),
];

pub const DARK: [(&str, &str); 20] = [
    ("background", "oklch(0.145 0 0)"),
    ("foreground", "oklch(0.985 0 0)"),
    ("card", "oklch(0.205 0 0)"),
    ("cardForeground", "oklch(0.985 0 0)"),
    ("popover", "oklch(0.205 0 0)"),
    ("popoverForeground", "oklch(0.985 0 0)"),
    ("primary", "oklch(0.922 0 0)"),
    ("primaryForeground", "oklch(0.205 0 0)"),
    ("secondary", "oklch(0.269 0 0)"),
    ("secondaryForeground", "oklch(0.985 0 0)"),
    ("muted", "oklch(0.269 0 0)"),
    ("mutedForeground", "oklch(0.708 0 0)"),
    ("accent", "oklch(0.269 0 0)"),
    ("accentForeground", "oklch(0.985 0 0)"),
    ("destructive", "oklch(0.704 0.191 22.216)"),
    ("destructiveForeground", "oklch(0.985 0 0)"),
    ("border", "oklch(1 0 0 / 10%)"),
    ("input", "oklch(1 0 0 / 15%)"),
    ("ring", "oklch(0.556 0 0)"),
    ("success", "oklch(0.6 0.17 150)"),
];

const DEFAULT_RADIUS: f64 = 10.0;

#[derive(Debug, Clone, PartialEq)]
pub struct NeutralTheme {
    colors: IndexMap<String, String>,
    pub radius: f64,
}

impl NeutralTheme {
    pub fn get(&self, key: &str) -> &str {
        self.colors.get(key).map_or("", String::as_str)
    }

    #[must_use]
    pub const fn with_radius(mut self, radius: f64) -> Self {
        self.radius = radius;
        self
    }
}

pub fn neutral_theme(theme: &Theme) -> NeutralTheme {
    let mut colors: IndexMap<String, String> = defaults(theme.mode)
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect();
    for (key, value) in &theme.colors {
        if !value.trim().is_empty() {
            colors.insert(key.clone(), value.clone());
        }
    }
    NeutralTheme {
        colors,
        radius: DEFAULT_RADIUS,
    }
}

const fn defaults(mode: ThemeMode) -> [(&'static str, &'static str); 20] {
    match mode {
        ThemeMode::Light => LIGHT,
        ThemeMode::Dark => DARK,
    }
}

pub fn schema_colors(mode: ThemeMode) -> Value {
    Value::Object(
        defaults(mode)
            .iter()
            .map(|(k, v)| ((*k).to_owned(), Value::String((*v).to_owned())))
            .collect::<Map<_, _>>(),
    )
}
