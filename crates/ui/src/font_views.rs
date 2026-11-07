use opsis_compile::text::{plural, plural_with};
use opsis_fonts::online::{CatalogCategory, FontCatalogEntry};

const WEIGHT_NAMES: [(&str, &str); 9] = [
    ("100", "Thin"),
    ("200", "Extra light"),
    ("300", "Light"),
    ("400", "Regular"),
    ("500", "Medium"),
    ("600", "Semibold"),
    ("700", "Bold"),
    ("800", "Extra bold"),
    ("900", "Black"),
];

fn weight_name(weight: &str) -> String {
    if let Some((low, high)) = weight.split_once(' ') {
        return format!("All weights ({low}–{high})");
    }
    WEIGHT_NAMES
        .iter()
        .find(|(number, _)| *number == weight)
        .map_or_else(|| weight.to_owned(), |(_, name)| (*name).to_owned())
}

pub fn describe_faces<'a>(faces: impl IntoIterator<Item = (&'a str, bool)>) -> String {
    faces
        .into_iter()
        .map(|(weight, italic)| {
            if italic {
                format!("{} italic", weight_name(weight))
            } else {
                weight_name(weight)
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

pub const fn category_label(category: CatalogCategory) -> &'static str {
    match category {
        CatalogCategory::SansSerif => "Sans serif",
        CatalogCategory::Serif => "Serif",
        CatalogCategory::Monospace => "Monospace",
        CatalogCategory::Display => "Display",
        CatalogCategory::Handwriting => "Handwriting",
        CatalogCategory::Icons => "Icons",
        CatalogCategory::Other => "Other",
    }
}

pub fn catalog_detail(entry: &FontCatalogEntry) -> String {
    let weights = opsis_types::numbers::float(entry.weights.len());
    let italics = if entry.styles.iter().any(|s| s == "italic") {
        " · italics"
    } else {
        ""
    };
    format!(
        "{} · {}{italics}",
        category_label(entry.category),
        plural(weights, "weight")
    )
}

pub fn licence_note(license: Option<&str>, installed: bool) -> String {
    if let Some(license) = license {
        return format!("Licence file found: {license}. It's kept with the fonts.");
    }
    if installed {
        "Fonts installed on a computer often have licences that limit use in videos. Check yours before you publish.".to_owned()
    } else {
        "No licence file found. Check that you're allowed to use these fonts in videos.".to_owned()
    }
}

pub fn found_text(families: usize, skipped: u32, unreadable: u32) -> String {
    let found = plural_with(
        opsis_types::numbers::float(families),
        "font family",
        "font families",
    );
    let skipped = if skipped > 0 {
        format!(" {skipped} duplicate files in other formats were skipped.")
    } else {
        String::new()
    };
    let unreadable = if unreadable > 0 {
        format!(
            " {} couldn't be read as a font.",
            plural(f64::from(unreadable), "file")
        )
    } else {
        String::new()
    };
    format!("Found {found}.{skipped}{unreadable}")
}

pub fn added_text(families: &[String]) -> String {
    match families {
        [one] => format!("Added {one}"),
        many => format!(
            "Added {}",
            plural(opsis_types::numbers::float(many.len()), "font")
        ),
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::describe_faces;

    proptest! {
        #[test]
        fn each_face_reads_as_one_part(faces in proptest::collection::vec((1u32..10, any::<bool>()), 1..12)) {
            let weights: Vec<String> = faces.iter().map(|(w, _)| (w * 100).to_string()).collect();
            let text = describe_faces(weights.iter().map(String::as_str).zip(faces.iter().map(|f| f.1)));
            let parts: Vec<&str> = text.split(", ").collect();
            prop_assert_eq!(parts.len(), faces.len());
            for (part, (_, italic)) in parts.iter().zip(&faces) {
                prop_assert_eq!(part.ends_with(" italic"), *italic);
            }
        }
    }
}
