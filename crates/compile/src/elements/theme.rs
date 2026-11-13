use indexmap::IndexMap;
use opsis_macros::settings;
use schemars::{JsonSchema, Schema};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::elements::neutral_theme::schema_colors;

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

#[settings(export, eq)]
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
            "light": schema_colors(ThemeMode::Light),
            "dark": schema_colors(ThemeMode::Dark),
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

pub fn keystroke_themes(schema: &mut Schema) {
    with_themes(
        schema,
        json!({
            "light": {
                "cap": "#fcfcfb",
                "edge": "#dfdfda",
                "wall": "#cfcfca",
                "ink": "#1d1d1f",
                "separator": "#8e8e89",
                "shadow": "rgba(20, 20, 16, 0.18)"
            },
            "dark": {
                "cap": "#38393e",
                "edge": "#48494f",
                "wall": "#1f2023",
                "ink": "#f5f5f7",
                "separator": "#9d9da4",
                "shadow": "rgba(0, 0, 0, 0.5)"
            }
        }),
    );
}

pub fn notification_themes(schema: &mut Schema) {
    with_themes(
        schema,
        json!({
            "light": {
                "window": "#FFFFFF",
                "border": "rgba(0,0,0,0.10)",
                "highlight": "rgba(255,255,255,0.9)",
                "shadow": "rgba(15,18,25,0.18)",
                "fg": "#22252B",
                "dim": "#6B7280",
                "faint": "#A0A6B0",
                "success": "#7FD1A0",
                "error": "#F97362",
                "info": "#7DD3E8"
            },
            "dark": {
                "window": "#0B0C11",
                "border": "rgba(255,255,255,0.09)",
                "highlight": "rgba(255,255,255,0.14)",
                "shadow": "rgba(0,0,0,0.55)",
                "fg": "#D8DCE4",
                "dim": "#7A828F",
                "faint": "#4A5160",
                "success": "#7FD1A0",
                "error": "#F97362",
                "info": "#7DD3E8"
            }
        }),
    );
}

pub fn stat_card_themes(schema: &mut Schema) {
    with_themes(
        schema,
        json!({
            "light": {
                "page": "#F4F4F2",
                "fg": "#22252B",
                "faint": "#A0A6B0",
                "band": "rgba(15,18,25,0.045)"
            },
            "dark": {
                "page": "#07070B",
                "fg": "#D8DCE4",
                "faint": "#4A5160",
                "band": "rgba(255,255,255,0.05)"
            }
        }),
    );
}

pub fn metric_ticker_themes(schema: &mut Schema) {
    with_themes(
        schema,
        json!({
            "light": {
                "bg": "#F4F4F2",
                "ink": "#22252B",
                "label": "rgba(34,37,43,0.62)",
                "eyebrow": "rgba(34,37,43,0.48)",
                "card": "#FFFFFF",
                "border": "rgba(0,0,0,0.10)",
                "accent": "#b45309",
                "up": "#0f766e",
                "down": "#dc2626"
            },
            "dark": {
                "bg": "#080810",
                "ink": "#fafafa",
                "label": "rgba(250,250,250,0.58)",
                "eyebrow": "rgba(250,250,250,0.44)",
                "card": "rgba(250,250,250,0.035)",
                "border": "rgba(250,250,250,0.10)",
                "accent": "#e8b86d",
                "up": "#2dd4bf",
                "down": "#f87171"
            }
        }),
    );
}

pub fn code_themes(schema: &mut Schema) {
    with_themes(
        schema,
        json!({
            "light": {
                "page": "#F4F4F2",
                "window": "#FFFFFF",
                "header": "rgba(0,0,0,0.025)",
                "border": "rgba(0,0,0,0.10)",
                "highlight": "rgba(255,255,255,0.9)",
                "gutter": "rgba(0,0,0,0.018)",
                "fg": "#22252B",
                "dim": "#6B7280",
                "faint": "#A0A6B0",
                "band": "rgba(15,18,25,0.045)",
                "shadow": "rgba(15,18,25,0.18)",
                "plain": "#22252B",
                "comment": "#8A93A1",
                "string": "#1B7F4B",
                "keyword": "#8B3FBF",
                "number": "#9A5B12",
                "type": "#0E7490",
                "call": "#2563A8",
                "prop": "#9A5B12",
                "punct": "#6B7280",
                "tag": "#0E7490",
                "added": "#1B7F4B",
                "removed": "#DC2626"
            },
            "dark": {
                "page": "#07070B",
                "window": "#0B0C11",
                "header": "rgba(255,255,255,0.035)",
                "border": "rgba(255,255,255,0.09)",
                "highlight": "rgba(255,255,255,0.14)",
                "gutter": "rgba(255,255,255,0.02)",
                "fg": "#D8DCE4",
                "dim": "#7A828F",
                "faint": "#4A5160",
                "band": "rgba(255,255,255,0.05)",
                "shadow": "rgba(0,0,0,0.55)",
                "plain": "#D8DCE4",
                "comment": "#5C6472",
                "string": "#9BD4A0",
                "keyword": "#C99BE8",
                "number": "#E8B86D",
                "type": "#7DD3E8",
                "call": "#8FB8F0",
                "prop": "#E8B86D",
                "punct": "#7A828F",
                "tag": "#7DD3E8",
                "added": "#9BD4A0",
                "removed": "#F87171"
            }
        }),
    );
}

pub fn code_morph_themes(schema: &mut Schema) {
    with_themes(
        schema,
        json!({
            "light": {
                "panel": "#ffffff",
                "chrome": "#f7f7f8",
                "border": "#e4e4e7",
                "dot": "#d4d4d8",
                "label": "#27272a",
                "gutter": "#b4b4bb",
                "keyword": "#71717a",
                "function": "#09090b",
                "type": "#09090b",
                "text": "#3f3f46",
                "punct": "#909098",
                "comment": "#a1a1aa",
                "mix": "#000000"
            },
            "dark": {
                "panel": "#111113",
                "chrome": "#0c0c0e",
                "border": "#27272a",
                "dot": "#3f3f46",
                "label": "#e4e4e7",
                "gutter": "#52525b",
                "keyword": "#a1a1aa",
                "function": "#fafafa",
                "type": "#fafafa",
                "text": "#d4d4d8",
                "punct": "#71717a",
                "comment": "#5b5b63",
                "mix": "#ffffff"
            }
        }),
    );
}

pub fn agent_run_themes(schema: &mut Schema) {
    with_themes(
        schema,
        json!({
            "light": {
                "panel": "#ffffff",
                "border": "#e4e4e0",
                "divider": "#ededea",
                "fg": "#1c1c1a",
                "muted": "#63635e",
                "subtle": "#9a9a94",
                "surface": "#f3f3f1",
                "surfaceBorder": "#e7e7e3",
                "code": "#f8f8f6",
                "track": "#e3e3df"
            },
            "dark": {
                "panel": "#161616",
                "border": "#2b2b2a",
                "divider": "#232322",
                "fg": "#ececea",
                "muted": "#a2a29d",
                "subtle": "#6d6d68",
                "surface": "#212120",
                "surfaceBorder": "#2e2e2d",
                "code": "#1c1c1b",
                "track": "#343433"
            }
        }),
    );
}

pub fn steps_timeline_themes(schema: &mut Schema) {
    with_themes(
        schema,
        json!({
            "light": {
                "page": "#F4F4F2",
                "window": "#FFFFFF",
                "border": "rgba(0,0,0,0.10)",
                "fg": "#22252B",
                "dim": "#6B7280",
                "faint": "#A0A6B0"
            },
            "dark": {
                "page": "#0c0c14",
                "window": "#0B0C11",
                "border": "rgba(255,255,255,0.09)",
                "fg": "#D8DCE4",
                "dim": "#7A828F",
                "faint": "#4A5160"
            }
        }),
    );
}
