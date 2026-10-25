use std::sync::LazyLock;

use opsis_types::fonts::{FontCategory, FontFamilyDef, ProjectFonts};

static BUNDLED: LazyLock<Vec<FontFamilyDef>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("bundled.generated.json")).expect("bundled font list parses")
});

pub fn bundled_fonts() -> &'static [FontFamilyDef] {
    &BUNDLED
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    fn stack(self) -> String {
        let fallback = if self == Self::Code {
            "ui-monospace, monospace"
        } else {
            "system-ui, sans-serif"
        };
        format!("\"{}\", {fallback}", self.family())
    }
}

pub fn same_family(a: &str, b: &str) -> bool {
    a.to_lowercase() == b.to_lowercase()
}

fn family_alias(name: &str) -> &str {
    match name {
        "Google Sans" => "Google Sans Flex",
        other => other,
    }
}

pub fn find_family<'a>(name: &str, families: &'a [FontFamilyDef]) -> Option<&'a FontFamilyDef> {
    let wanted = family_alias(name);
    families
        .iter()
        .chain(bundled_fonts())
        .find(|f| same_family(&f.family, wanted))
}

fn generic(category: FontCategory) -> &'static str {
    match category {
        FontCategory::SansSerif | FontCategory::Display => "sans-serif",
        FontCategory::Serif => "serif",
        FontCategory::Monospace => "monospace",
        FontCategory::Handwriting => "cursive",
    }
}

pub fn font_stack(name: &str, families: &[FontFamilyDef]) -> String {
    let category = find_family(name, families).map_or(FontCategory::SansSerif, |f| f.category);
    format!("\"{}\", {}", name.replace('"', ""), generic(category))
}

pub fn font_prop(value: &str) -> Option<String> {
    let name = value.trim();
    (!name.is_empty()).then(|| font_stack(name, &[]))
}

pub fn font_or(family: Option<&str>, role: FontRole) -> String {
    family
        .filter(|f| !f.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| role.stack())
}

pub fn main_font() -> String {
    FontRole::Main.stack()
}

pub fn code_font() -> String {
    FontRole::Code.stack()
}

pub fn role_family_name(role: FontRole, fonts: &ProjectFonts) -> &str {
    match role {
        FontRole::Heading if !fonts.heading.is_empty() => &fonts.heading,
        FontRole::Heading | FontRole::Main => &fonts.main,
        FontRole::Code => &fonts.code,
    }
}

pub fn first_family(stack: &str) -> String {
    let first = stack.split(',').next().unwrap_or("").trim();
    let first = first.strip_prefix(['"', '\'']).unwrap_or(first);
    first.strip_suffix(['"', '\'']).unwrap_or(first).to_owned()
}
