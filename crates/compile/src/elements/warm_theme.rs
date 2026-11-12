use indexmap::IndexMap;

use crate::elements::theme::{Theme, ThemeMode};

const LIGHT: [(&str, &str); 19] = [
    ("background", "#faf9f6"),
    ("foreground", "#141414"),
    ("card", "#ffffff"),
    ("cardForeground", "#141414"),
    ("popover", "#ffffff"),
    ("popoverForeground", "#141414"),
    ("primary", "#3072db"),
    ("primaryForeground", "#ffffff"),
    ("secondary", "#f2f0eb"),
    ("secondaryForeground", "#141414"),
    ("muted", "#f2f0eb"),
    ("mutedForeground", "#6e6a63"),
    ("accent", "#f2f0eb"),
    ("accentForeground", "#141414"),
    ("destructive", "#d92d20"),
    ("destructiveForeground", "#ffffff"),
    ("border", "#d9d9d9"),
    ("input", "#d9d9d9"),
    ("ring", "#3072db"),
];

const DARK: [(&str, &str); 19] = [
    ("background", "#0a0a0b"),
    ("foreground", "#fafafa"),
    ("card", "#141417"),
    ("cardForeground", "#fafafa"),
    ("popover", "#141417"),
    ("popoverForeground", "#fafafa"),
    ("primary", "#3072db"),
    ("primaryForeground", "#ffffff"),
    ("secondary", "#1d1d21"),
    ("secondaryForeground", "#fafafa"),
    ("muted", "#1b1b1f"),
    ("mutedForeground", "#a1a1aa"),
    ("accent", "#1d1d21"),
    ("accentForeground", "#fafafa"),
    ("destructive", "#f97066"),
    ("destructiveForeground", "#fafafa"),
    ("border", "#26272b"),
    ("input", "#26272b"),
    ("ring", "#3072db"),
];

pub const DEFAULT_RADIUS: f64 = 10.0;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WarmTheme {
    colors: IndexMap<String, String>,
}

impl WarmTheme {
    pub fn get(&self, key: &str) -> &str {
        self.colors.get(key).map_or("", String::as_str)
    }

    pub fn font_family(&self) -> Option<&str> {
        self.colors
            .get("fontFamily")
            .map(String::as_str)
            .filter(|f| !f.is_empty())
    }

    pub fn radius(&self) -> f64 {
        self.colors
            .get("radius")
            .and_then(|r| r.trim().parse().ok())
            .unwrap_or(DEFAULT_RADIUS)
    }
}

pub fn warm_theme(theme: &Theme) -> WarmTheme {
    let base = match theme.mode {
        ThemeMode::Light => LIGHT,
        ThemeMode::Dark => DARK,
    };
    let mut colors: IndexMap<String, String> = base
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect();
    for (key, value) in &theme.colors {
        if !value.trim().is_empty() {
            colors.insert(key.clone(), value.clone());
        }
    }
    WarmTheme { colors }
}

pub fn resolve_font(name: &str) -> Option<String> {
    if name.is_empty() || name == "Default" {
        return None;
    }
    Some(match name {
        "Inter" | "Geist" | "Space Grotesk" | "Outfit" | "Montserrat" | "Instrument Serif" => {
            format!("\"{name}\"")
        }
        other => other.to_owned(),
    })
}
