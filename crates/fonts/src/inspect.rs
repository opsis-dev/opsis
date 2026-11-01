use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::sync::LazyLock;

use opsis_types::fonts::{FontCategory, FontFormat, FontStyle};
use regex::Regex;
use serde::{Deserialize, Serialize};
use skrifa::raw::types::Tag;
use skrifa::string::StringId;
use skrifa::{FontRef, MetadataProvider};
use specta::Type;

const WEIGHT_AXIS: Tag = Tag::new(b"wght");
const ITALIC_AXIS: Tag = Tag::new(b"ital");
const OPENTYPE_CFF_TAG: &[u8] = b"OTTO";

static STYLE_WORDS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\s+(variable|vf|thin|hairline|extra ?light|ultra ?light|light|regular|book|normal|medium|semi ?bold|demi ?bold|bold|extra ?bold|ultra ?bold|black|heavy|italic|oblique)$")
        .expect("pattern")
});
static VARIABLE_SUFFIX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\s+(variable|vf)$").expect("pattern"));
static SLANTED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)italic|oblique").expect("pattern"));
static LICENSE_NAME: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(^|/)(licen[cs]e|ofl|ffl|copying)[^/]*$").expect("pattern"));
static MONOSPACED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)mono|code").expect("pattern"));
static SERIF: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)serif").expect("pattern"));
static SANS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)sans").expect("pattern"));

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FontCandidate {
    pub name: String,
    pub family: String,
    pub weight: String,
    pub style: FontStyle,
    pub variable: bool,
    pub format: FontFormat,
    pub size: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct PlannedFamily<C = FontCandidate> {
    pub family: String,
    pub faces: Vec<C>,
}

#[derive(Debug)]
pub struct FontBytes {
    pub name: String,
    pub bytes: Vec<u8>,
}

pub fn format_of(name: &str) -> Option<FontFormat> {
    match opsis_compile::asset_ref::extension_of(name).as_str() {
        "woff2" => Some(FontFormat::Woff2),
        "woff" => Some(FontFormat::Woff),
        "ttf" => Some(FontFormat::Truetype),
        "otf" => Some(FontFormat::Opentype),
        _ => None,
    }
}

pub fn is_license_name(name: &str) -> bool {
    LICENSE_NAME.is_match(name)
}

pub fn installed_file_name(postscript_name: &str, bytes: &[u8]) -> String {
    let ext = if bytes.starts_with(OPENTYPE_CFF_TAG) {
        "otf"
    } else {
        "ttf"
    };
    format!("{postscript_name}.{ext}")
}

fn strip_style_words(name: &str) -> &str {
    match STYLE_WORDS.find(name) {
        Some(found) => strip_style_words(&name[..found.start()]),
        None => name,
    }
}

pub fn family_name_from(preferred: Option<&str>, legacy: &str) -> String {
    if let Some(preferred) = preferred.filter(|p| !p.is_empty()) {
        return VARIABLE_SUFFIX.replace(preferred.trim(), "").into_owned();
    }
    match strip_style_words(legacy.trim()) {
        "" => legacy.to_owned(),
        name => name.to_owned(),
    }
}

pub fn guess_category(family: &str) -> FontCategory {
    if MONOSPACED.is_match(family) {
        FontCategory::Monospace
    } else if SERIF.is_match(family) && !SANS.is_match(family) {
        FontCategory::Serif
    } else {
        FontCategory::SansSerif
    }
}

fn sfnt(format: FontFormat, bytes: &[u8]) -> Option<Cow<'_, [u8]>> {
    match format {
        FontFormat::Woff2 => wuff::decompress_woff2(bytes).ok().map(Cow::Owned),
        FontFormat::Woff => wuff::decompress_woff1(bytes).ok().map(Cow::Owned),
        FontFormat::Truetype | FontFormat::Opentype => Some(Cow::Borrowed(bytes)),
    }
}

fn english(font: &FontRef<'_>, id: StringId) -> Option<String> {
    font.localized_strings(id)
        .english_or_first()
        .map(|s| s.to_string())
}

pub fn read_candidate(name: &str, bytes: &[u8]) -> Option<FontCandidate> {
    let format = format_of(name)?;
    let data = sfnt(format, bytes)?;
    let font = FontRef::new(&data).ok()?;
    let legacy = english(&font, StringId::FAMILY_NAME)?;
    let family = family_name_from(
        english(&font, StringId::TYPOGRAPHIC_FAMILY_NAME).as_deref(),
        &legacy,
    );
    let axes = font.axes();
    let wght = axes.iter().find(|a| a.tag() == WEIGHT_AXIS);
    let subfamily = english(&font, StringId::SUBFAMILY_NAME).unwrap_or_default();
    let attributes = font.attributes();
    let italic = attributes.style != skrifa::attribute::Style::Normal
        || SLANTED.is_match(&subfamily)
        || axes.iter().any(|a| a.tag() == ITALIC_AXIS);
    let weight = match &wght {
        Some(weight_axis) => format!(
            "{} {}",
            weight_axis.min_value().round(),
            weight_axis.max_value().round()
        ),
        None => attributes.weight.value().round().to_string(),
    };
    Some(FontCandidate {
        name: name.to_owned(),
        family,
        weight,
        style: if italic {
            FontStyle::Italic
        } else {
            FontStyle::Normal
        },
        variable: wght.is_some(),
        format,
        size: u32::try_from(bytes.len()).unwrap_or(u32::MAX),
    })
}

fn format_rank(format: FontFormat) -> u8 {
    match format {
        FontFormat::Woff2 => 0,
        FontFormat::Woff => 1,
        FontFormat::Truetype => 2,
        FontFormat::Opentype => 3,
    }
}

fn lowest_weight(weight: &str) -> i32 {
    weight
        .split(' ')
        .next()
        .and_then(|w| w.parse().ok())
        .unwrap_or(0)
}

fn pick_faces<C: AsRef<FontCandidate> + Clone>(list: &[C], style: FontStyle) -> Vec<C> {
    let of_style: Vec<&C> = list.iter().filter(|c| c.as_ref().style == style).collect();
    let variable: Vec<&C> = of_style
        .iter()
        .copied()
        .filter(|c| c.as_ref().variable)
        .collect();
    let mut by_weight: BTreeMap<&str, &C> = BTreeMap::new();
    for c in if variable.is_empty() {
        of_style
    } else {
        variable
    } {
        let better = by_weight
            .get(c.as_ref().weight.as_str())
            .is_none_or(|best| format_rank(c.as_ref().format) < format_rank(best.as_ref().format));
        if better {
            by_weight.insert(&c.as_ref().weight, c);
        }
    }
    let mut faces: Vec<C> = by_weight.into_values().cloned().collect();
    faces.sort_by_key(|c| lowest_weight(&c.as_ref().weight));
    faces
}

fn by_family(a: &str, b: &str) -> Ordering {
    a.to_lowercase()
        .cmp(&b.to_lowercase())
        .then_with(|| a.cmp(b))
}

pub fn plan_font_files<C: AsRef<FontCandidate> + Clone>(candidates: &[C]) -> Vec<PlannedFamily<C>> {
    let mut groups: BTreeMap<&str, Vec<C>> = BTreeMap::new();
    for c in candidates {
        groups
            .entry(&c.as_ref().family)
            .or_default()
            .push(c.clone());
    }
    let mut plan: Vec<PlannedFamily<C>> = groups
        .into_iter()
        .map(|(family, list)| {
            let mut faces = pick_faces(&list, FontStyle::Normal);
            faces.extend(pick_faces(&list, FontStyle::Italic));
            PlannedFamily {
                family: family.to_owned(),
                faces,
            }
        })
        .filter(|f| !f.faces.is_empty())
        .collect();
    plan.sort_by(|a, b| by_family(&a.family, &b.family));
    plan
}

impl AsRef<FontCandidate> for FontCandidate {
    fn as_ref(&self) -> &FontCandidate {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(
        family: &str,
        weight: &str,
        style: FontStyle,
        variable: bool,
        format: FontFormat,
    ) -> FontCandidate {
        FontCandidate {
            name: format!("{family}-{weight}"),
            family: family.into(),
            weight: weight.into(),
            style,
            variable,
            format,
            size: 1,
        }
    }

    #[test]
    fn names_the_family_without_its_style_words() {
        assert_eq!(
            family_name_from(None, "Acme Sans Semi Bold Italic"),
            "Acme Sans"
        );
        assert_eq!(
            family_name_from(Some("Acme Sans Variable"), "Acme Sans Bold"),
            "Acme Sans"
        );
        assert_eq!(family_name_from(None, "Bold"), "Bold");
    }

    #[test]
    fn keeps_one_file_per_style_preferring_variable_files_and_woff2() {
        let list = [
            candidate("B", "700", FontStyle::Normal, false, FontFormat::Truetype),
            candidate("B", "400", FontStyle::Normal, false, FontFormat::Truetype),
            candidate("B", "400", FontStyle::Normal, false, FontFormat::Woff2),
            candidate(
                "a",
                "100 900",
                FontStyle::Italic,
                true,
                FontFormat::Truetype,
            ),
            candidate("a", "400", FontStyle::Italic, false, FontFormat::Woff2),
        ];
        let plan = plan_font_files(&list);
        let summary: Vec<(String, Vec<(String, FontFormat)>)> = plan
            .into_iter()
            .map(|f| {
                (
                    f.family,
                    f.faces.into_iter().map(|c| (c.weight, c.format)).collect(),
                )
            })
            .collect();
        assert_eq!(
            summary,
            [
                (
                    "a".to_owned(),
                    vec![("100 900".to_owned(), FontFormat::Truetype)]
                ),
                (
                    "B".to_owned(),
                    vec![
                        ("400".to_owned(), FontFormat::Woff2),
                        ("700".to_owned(), FontFormat::Truetype)
                    ]
                ),
            ]
        );
    }

    #[test]
    fn guesses_the_category_from_the_name() {
        assert_eq!(guess_category("Fira Code"), FontCategory::Monospace);
        assert_eq!(guess_category("Noto Serif"), FontCategory::Serif);
        assert_eq!(guess_category("Noto Sans Serif"), FontCategory::SansSerif);
    }

    #[test]
    fn reads_the_family_weight_range_and_style_of_a_compressed_variable_face() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../public/fonts/bundled/dm-sans");
        let read =
            |file: &str| read_candidate(file, &std::fs::read(dir.join(file)).unwrap()).unwrap();
        let normal = read("dm-sans-latin-wght-normal.woff2");
        let italic = read("dm-sans-latin-wght-italic.woff2");
        assert_eq!(
            (
                normal.family.as_str(),
                normal.weight.as_str(),
                normal.variable
            ),
            ("DM Sans 9pt", "100 1000", true)
        );
        assert_eq!(
            (normal.style, italic.style),
            (FontStyle::Normal, FontStyle::Italic)
        );
        assert_eq!(read_candidate("broken.ttf", &[1, 2, 3, 4]), None);
    }
}
