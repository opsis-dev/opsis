use opsis_types::fonts::{FontFaceDef, FontFamilyDef, FontRole, FontStyle, ProjectFonts};
use serde_json::Value;

use crate::asset_ref::{PreparedKind, PreparedRef, extension_of};
use crate::elements::core::{Card3d, Logo3d, Model3d, Text3d};
use crate::fonts::{find_family, first_family, role_family_name};
use crate::params::{Values, typed_values};

const HEADING_FAMILY: &str = "df-heading";
const DEFAULT_WEIGHT: f64 = 400.0;

fn role_of(name: &str) -> Option<FontRole> {
    match name {
        "df-main" | "Inter" => Some(FontRole::Main),
        "df-heading" => Some(FontRole::Heading),
        "df-code" | "JetBrains Mono" => Some(FontRole::Code),
        _ => None,
    }
}

fn family_for<'a>(name: &str, fonts: &'a ProjectFonts) -> Option<&'a FontFamilyDef> {
    role_of(name)
        .and_then(|role| find_family(role_family_name(role, fonts), &fonts.families))
        .or_else(|| find_family(name, &fonts.families))
}

fn weight_range(weight: &str) -> (f64, f64) {
    let numbers: Vec<f64> = weight
        .split_whitespace()
        .filter_map(|w| w.parse().ok())
        .filter(|w: &f64| w.is_finite())
        .collect();
    let low = numbers.first().copied().unwrap_or(DEFAULT_WEIGHT);
    (low, numbers.get(1).copied().unwrap_or(low))
}

fn weight_distance(face: &FontFaceDef, wanted: f64) -> f64 {
    let (low, high) = weight_range(&face.weight);
    if wanted < low {
        low - wanted
    } else if wanted > high {
        wanted - high
    } else {
        0.0
    }
}

pub fn face_for(def: &FontFamilyDef, weight: f64, italic: bool) -> Option<&FontFaceDef> {
    let styled: Vec<&FontFaceDef> = def
        .faces
        .iter()
        .filter(|f| (f.style == FontStyle::Italic) == italic)
        .collect();
    let faces = if styled.is_empty() {
        def.faces.iter().collect()
    } else {
        styled
    };
    faces
        .into_iter()
        .min_by(|a, b| weight_distance(a, weight).total_cmp(&weight_distance(b, weight)))
}

fn prepare_src(props: &mut Values, src: &str, kind: PreparedKind) {
    if src.is_empty() {
        return;
    }
    if let Some(prepared) = PreparedRef::new(kind, src) {
        props.insert("src".into(), Value::from(prepared.to_ref()));
    }
}

fn typeface(props: &mut Values, text: &Text3d, fonts: &ProjectFonts) {
    let family = Some(first_family(&text.font_family))
        .filter(|f| !f.is_empty())
        .unwrap_or_else(|| HEADING_FAMILY.to_owned());
    let weight = text.font_weight;
    let Some(face) = family_for(&family, fonts).and_then(|def| face_for(def, weight, text.italic))
    else {
        return;
    };
    if let Some(prepared) = PreparedRef::new(PreparedKind::Typeface { weight }, &face.file) {
        props.insert("typeface".into(), Value::from(prepared.to_ref()));
        props.insert(
            "typefaceItalic".into(),
            Value::from(face.style == FontStyle::Italic),
        );
    }
}

fn logo_kind(src: &str) -> PreparedKind {
    if extension_of(src) == "svg" {
        PreparedKind::Svg
    } else {
        PreparedKind::Texture
    }
}

pub fn prepare_props(key: &str, props: &mut Values, fonts: &ProjectFonts) {
    match key {
        "core/text-3d" => {
            if let Ok(text) = typed_values::<Text3d>(props) {
                typeface(props, &text, fonts);
            }
        }
        "core/model-3d" => {
            if let Ok(model) = typed_values::<Model3d>(props) {
                prepare_src(props, &model.src, PreparedKind::Model);
            }
        }
        "core/card-3d" => {
            if let Ok(card) = typed_values::<Card3d>(props) {
                prepare_src(props, &card.src, PreparedKind::Texture);
            }
        }
        "core/logo-3d" => {
            if let Ok(logo) = typed_values::<Logo3d>(props) {
                prepare_src(props, &logo.src, logo_kind(&logo.src));
            }
        }
        _ => {}
    }
}
