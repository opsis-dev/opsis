use indexmap::IndexMap;

use crate::elements::theme::ThemeMode;

const DARK: [(&str, &str); 11] = [
    ("page", "#07070B"),
    ("window", "#0B0C11"),
    ("header", "rgba(255,255,255,0.035)"),
    ("border", "rgba(255,255,255,0.09)"),
    ("highlight", "rgba(255,255,255,0.14)"),
    ("gutter", "rgba(255,255,255,0.02)"),
    ("fg", "#D8DCE4"),
    ("dim", "#7A828F"),
    ("faint", "#4A5160"),
    ("band", "rgba(255,255,255,0.05)"),
    ("shadow", "rgba(0,0,0,0.55)"),
];

const LIGHT: [(&str, &str); 11] = [
    ("page", "#F4F4F2"),
    ("window", "#FFFFFF"),
    ("header", "rgba(0,0,0,0.025)"),
    ("border", "rgba(0,0,0,0.10)"),
    ("highlight", "rgba(255,255,255,0.9)"),
    ("gutter", "rgba(0,0,0,0.018)"),
    ("fg", "#22252B"),
    ("dim", "#6B7280"),
    ("faint", "#A0A6B0"),
    ("band", "rgba(15,18,25,0.045)"),
    ("shadow", "rgba(15,18,25,0.18)"),
];

pub const DARK_TOKENS: [(&str, &str); 10] = [
    ("plain", "#D8DCE4"),
    ("comment", "#5C6472"),
    ("string", "#9BD4A0"),
    ("keyword", "#C99BE8"),
    ("number", "#E8B86D"),
    ("type", "#7DD3E8"),
    ("call", "#8FB8F0"),
    ("prop", "#E8B86D"),
    ("punct", "#7A828F"),
    ("tag", "#7DD3E8"),
];

pub const LIGHT_TOKENS: [(&str, &str); 10] = [
    ("plain", "#22252B"),
    ("comment", "#8A93A1"),
    ("string", "#1B7F4B"),
    ("keyword", "#8B3FBF"),
    ("number", "#9A5B12"),
    ("type", "#0E7490"),
    ("call", "#2563A8"),
    ("prop", "#9A5B12"),
    ("punct", "#6B7280"),
    ("tag", "#0E7490"),
];

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Palette(IndexMap<String, String>);

impl Palette {
    pub fn get(&self, key: &str) -> &str {
        self.0.get(key).map_or("", String::as_str)
    }
}

pub fn code_palette(
    mode: ThemeMode,
    extra: &[(&str, &str)],
    overrides: &IndexMap<String, String>,
) -> Palette {
    let base = match mode {
        ThemeMode::Dark => DARK,
        ThemeMode::Light => LIGHT,
    };
    palette_of(&base, extra, overrides)
}

pub fn palette_of(
    base: &[(&str, &str)],
    extra: &[(&str, &str)],
    overrides: &IndexMap<String, String>,
) -> Palette {
    let mut out: IndexMap<String, String> = base
        .iter()
        .chain(extra)
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect();
    for (k, v) in overrides {
        out.insert(k.clone(), v.clone());
    }
    Palette(out)
}
