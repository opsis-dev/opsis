use indexmap::IndexMap;
use opsis_macros::settings;
use schemars::{JsonSchema, Schema};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Default,
    Serialize,
    Deserialize,
    JsonSchema,
    specta::Type,
)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    #[default]
    Light,
    Dark,
}

#[settings]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema, specta::Type)]
pub struct Theme {
    pub mode: ThemeMode,
    #[setting("propertyNames" = { "type": "string" })]
    pub colors: IndexMap<String, String>,
}

impl Theme {
    pub fn of(mode: ThemeMode) -> Self {
        Self {
            mode,
            colors: IndexMap::new(),
        }
    }
}

pub fn with_themes(schema: &mut Schema, themes: Value) {
    if let Some(object) = schema.as_object_mut() {
        object.insert("x-themes".to_owned(), themes);
    }
}

pub fn neutral_themes(schema: &mut Schema) {
    with_themes(
        schema,
        json!({
            "light": {
                "background": "oklch(1 0 0)",
                "foreground": "oklch(0.145 0 0)",
                "card": "oklch(1 0 0)",
                "cardForeground": "oklch(0.145 0 0)",
                "popover": "oklch(1 0 0)",
                "popoverForeground": "oklch(0.145 0 0)",
                "primary": "oklch(0.205 0 0)",
                "primaryForeground": "oklch(0.985 0 0)",
                "secondary": "oklch(0.97 0 0)",
                "secondaryForeground": "oklch(0.205 0 0)",
                "muted": "oklch(0.97 0 0)",
                "mutedForeground": "oklch(0.556 0 0)",
                "accent": "oklch(0.97 0 0)",
                "accentForeground": "oklch(0.205 0 0)",
                "destructive": "oklch(0.577 0.245 27.325)",
                "destructiveForeground": "oklch(0.985 0 0)",
                "border": "oklch(0.922 0 0)",
                "input": "oklch(0.922 0 0)",
                "ring": "oklch(0.708 0 0)"
            },
            "dark": {
                "background": "oklch(0.145 0 0)",
                "foreground": "oklch(0.985 0 0)",
                "card": "oklch(0.205 0 0)",
                "cardForeground": "oklch(0.985 0 0)",
                "popover": "oklch(0.205 0 0)",
                "popoverForeground": "oklch(0.985 0 0)",
                "primary": "oklch(0.922 0 0)",
                "primaryForeground": "oklch(0.205 0 0)",
                "secondary": "oklch(0.269 0 0)",
                "secondaryForeground": "oklch(0.985 0 0)",
                "muted": "oklch(0.269 0 0)",
                "mutedForeground": "oklch(0.708 0 0)",
                "accent": "oklch(0.269 0 0)",
                "accentForeground": "oklch(0.985 0 0)",
                "destructive": "oklch(0.704 0.191 22.216)",
                "destructiveForeground": "oklch(0.985 0 0)",
                "border": "oklch(1 0 0 / 10%)",
                "input": "oklch(1 0 0 / 15%)",
                "ring": "oklch(0.556 0 0)"
            }
        }),
    );
}

pub fn warm_themes(schema: &mut Schema) {
    with_themes(
        schema,
        json!({
            "light": {
                "background": "#faf9f6",
                "foreground": "#141414",
                "card": "#ffffff",
                "cardForeground": "#141414",
                "popover": "#ffffff",
                "popoverForeground": "#141414",
                "primary": "#3072db",
                "primaryForeground": "#ffffff",
                "secondary": "#f2f0eb",
                "secondaryForeground": "#141414",
                "muted": "#f2f0eb",
                "mutedForeground": "#6e6a63",
                "accent": "#f2f0eb",
                "accentForeground": "#141414",
                "destructive": "#d92d20",
                "destructiveForeground": "#ffffff",
                "border": "#d9d9d9",
                "input": "#d9d9d9",
                "ring": "#3072db"
            },
            "dark": {
                "background": "#0a0a0b",
                "foreground": "#fafafa",
                "card": "#141417",
                "cardForeground": "#fafafa",
                "popover": "#141417",
                "popoverForeground": "#fafafa",
                "primary": "#3072db",
                "primaryForeground": "#ffffff",
                "secondary": "#1d1d21",
                "secondaryForeground": "#fafafa",
                "muted": "#1b1b1f",
                "mutedForeground": "#a1a1aa",
                "accent": "#1d1d21",
                "accentForeground": "#fafafa",
                "destructive": "#f97066",
                "destructiveForeground": "#fafafa",
                "border": "#26272b",
                "input": "#26272b",
                "ring": "#3072db"
            }
        }),
    );
}
