use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use specta::Type;

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
    Type,
    schemars::JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub enum FontCategory {
    #[default]
    SansSerif,
    Serif,
    Monospace,
    Display,
    Handwriting,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type, schemars::JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum FontSource {
    Bundled,
    Online,
    File,
    Installed,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type, schemars::JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum FontStyle {
    Normal,
    Italic,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type, schemars::JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum FontFormat {
    Woff2,
    Woff,
    Truetype,
    Opentype,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, schemars::JsonSchema)]
pub struct FontFaceDef {
    pub file: String,
    pub weight: String,
    pub style: FontStyle,
    pub format: FontFormat,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FontFamilyDef {
    pub id: String,
    pub family: String,
    #[serde(default)]
    pub category: FontCategory,
    pub source: FontSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license_file: Option<String>,
    pub faces: Vec<FontFaceDef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, schemars::JsonSchema)]
pub struct ProjectFonts {
    #[serde(default = "default_main_font")]
    pub main: String,
    #[serde(default)]
    pub heading: String,
    #[serde(default = "default_code_font")]
    pub code: String,
    #[serde(default)]
    pub families: Vec<FontFamilyDef>,
}

pub const DEFAULT_MAIN_FONT: &str = "Inter";

pub const DEFAULT_CODE_FONT: &str = "JetBrains Mono";

fn default_main_font() -> String {
    DEFAULT_MAIN_FONT.to_owned()
}

fn default_code_font() -> String {
    DEFAULT_CODE_FONT.to_owned()
}

impl Default for ProjectFonts {
    fn default() -> Self {
        Self {
            main: default_main_font(),
            heading: String::new(),
            code: default_code_font(),
            families: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum FontRole {
    Main,
    Heading,
    Code,
}

impl FontRole {
    pub fn family(self) -> &'static str {
        match self {
            Self::Main => "df-main",
            Self::Heading => "df-heading",
            Self::Code => "df-code",
        }
    }
}

#[derive(
    Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, Type, schemars::JsonSchema,
)]
pub struct FilmFonts {
    pub families: IndexMap<String, FontFamilyDef>,
    pub preload: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct FontChoice {
    pub def: FontFamilyDef,
    pub preview: String,
    pub stack: String,
    pub selected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FontChoices {
    pub project: Vec<FontChoice>,
    pub built_in: Vec<FontChoice>,
    pub library: Vec<FontChoice>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current: Option<FontChoice>,
}
