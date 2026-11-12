use std::collections::BTreeSet;
use std::sync::LazyLock;

use opsis_types::fonts::{
    DEFAULT_CODE_FONT, DEFAULT_MAIN_FONT, FilmFonts, FontChoice, FontChoices, FontFamilyDef,
    FontRole, ProjectFonts,
};
use opsis_types::recording::ScreenProps;
use opsis_types::timeline::{TimelineItem, TimelineTrack};
use serde_json::{Map, Value};

use crate::clip::walk_items;
use crate::elements::element_meta;
use crate::terminal::TERMINAL_FONTS;

static BUNDLED: LazyLock<Vec<FontFamilyDef>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("bundled.generated.json")).expect("bundled font list parses")
});

pub fn bundled_fonts() -> &'static [FontFamilyDef] {
    &BUNDLED
}

pub fn same_family(a: &str, b: &str) -> bool {
    a.to_lowercase() == b.to_lowercase()
}

const FAMILY_ALIASES: [(&str, &str); 1] = [("Google Sans", "Google Sans Flex")];

const ROLE_NAMES: [(&str, FontRole); 5] = [
    ("df-main", FontRole::Main),
    ("df-heading", FontRole::Heading),
    ("df-code", FontRole::Code),
    (DEFAULT_MAIN_FONT, FontRole::Main),
    (DEFAULT_CODE_FONT, FontRole::Code),
];

fn family_alias(name: &str) -> &str {
    FAMILY_ALIASES
        .iter()
        .find(|(alias, _)| same_family(alias, name))
        .map_or(name, |(_, family)| family)
}

pub fn role_of(name: &str) -> Option<FontRole> {
    ROLE_NAMES
        .iter()
        .find(|(role_name, _)| same_family(role_name, name))
        .map(|(_, role)| *role)
}

pub fn family_for<'a>(name: &str, fonts: &'a ProjectFonts) -> Option<&'a FontFamilyDef> {
    role_of(name)
        .and_then(|role| find_family(role_family_name(role, fonts), &fonts.families))
        .or_else(|| find_family(name, &fonts.families))
}

pub fn find_family<'a>(name: &str, families: &'a [FontFamilyDef]) -> Option<&'a FontFamilyDef> {
    let wanted = family_alias(name);
    families
        .iter()
        .chain(bundled_fonts())
        .find(|f| same_family(&f.family, wanted))
}

pub fn font_stack(name: &str) -> String {
    format!("\"{}\"", name.replace('"', ""))
}

pub fn font_prop(value: &str) -> Option<String> {
    let name = value.trim();
    (!name.is_empty()).then(|| font_stack(name))
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

fn font_family_of(props: &Map<String, Value>) -> Option<String> {
    let name = first_family(props.get("fontFamily")?.as_str()?.trim());
    (!name.is_empty()).then_some(name)
}

fn used_font_names(tracks: &[TimelineTrack]) -> BTreeSet<String> {
    let mut used = BTreeSet::new();
    walk_items(tracks, &mut |item| {
        let (component, props) = match item {
            TimelineItem::Element {
                component, props, ..
            } => (Some(component), Some(props)),
            TimelineItem::Group { props, .. } => props
                .wrapper
                .as_ref()
                .map_or((None, None), |w| (Some(&w.component), Some(&w.props))),
            TimelineItem::Recording { props, .. } => {
                if matches!(props.screen, ScreenProps::Terminal { .. }) {
                    used.extend(TERMINAL_FONTS.iter().map(|f| (*f).to_owned()));
                }
                (None, None)
            }
        };
        used.extend(props.and_then(font_family_of));
        if let Some(meta) = component.and_then(|c| element_meta(c)) {
            used.extend(meta.fonts.iter().cloned());
        }
    });
    used
}

pub fn film_fonts(fonts: &ProjectFonts, tracks: &[TimelineTrack]) -> FilmFonts {
    let used = used_font_names(tracks);
    let names = ROLE_NAMES
        .iter()
        .map(|(name, _)| *name)
        .chain(FAMILY_ALIASES.iter().map(|(alias, _)| *alias))
        .chain(fonts.families.iter().map(|f| f.family.as_str()))
        .chain(
            bundled_fonts()
                .iter()
                .filter(|f| !f.hidden || used.contains(&f.family))
                .map(|f| f.family.as_str()),
        );
    let mut families = indexmap::IndexMap::new();
    for name in names {
        if let Some(def) = family_for(name, fonts) {
            families
                .entry(name.to_lowercase())
                .or_insert_with(|| def.clone());
        }
    }
    let mut preload: Vec<String> = ROLE_NAMES.iter().map(|(n, _)| (*n).to_owned()).collect();
    for name in used {
        if !preload.contains(&name) {
            preload.push(name);
        }
    }
    FilmFonts { families, preload }
}

pub fn preview_name(def: &FontFamilyDef) -> String {
    let source = serde_json::to_value(def.source)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_default();
    format!("df-preview-{source}-{}", def.id)
}

pub fn font_choice(def: &FontFamilyDef, value: &str) -> FontChoice {
    let preview = preview_name(def);
    FontChoice {
        stack: font_stack(&preview),
        preview,
        selected: same_family(&def.family, value),
        def: def.clone(),
    }
}

fn has_family(defs: &[FontFamilyDef], family: &str) -> bool {
    defs.iter().any(|d| same_family(&d.family, family))
}

pub fn font_choices(
    value: &str,
    project: &[FontFamilyDef],
    library: &[FontFamilyDef],
) -> FontChoices {
    let choices = |defs: &mut dyn Iterator<Item = &FontFamilyDef>| -> Vec<FontChoice> {
        defs.map(|d| font_choice(d, value)).collect()
    };
    let current = project
        .iter()
        .chain(bundled_fonts())
        .chain(library)
        .find(|f| same_family(&f.family, value))
        .map(|d| font_choice(d, value));
    FontChoices {
        project: choices(&mut project.iter()),
        built_in: choices(
            &mut bundled_fonts()
                .iter()
                .filter(|d| !d.hidden && !has_family(project, &d.family)),
        ),
        library: choices(&mut library.iter().filter(|d| {
            !has_family(project, &d.family) && !has_family(bundled_fonts(), &d.family)
        })),
        current,
    }
}

pub fn is_known_family(family: &str, library: &[FontFamilyDef]) -> bool {
    has_family(library, family) || has_family(bundled_fonts(), family)
}
