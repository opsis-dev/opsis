use std::path::{Path, PathBuf};

use opsis_prepare::typeface::typeface;
use serde_json::Value;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixture() -> Value {
    serde_json::from_str(
        &std::fs::read_to_string(root().join("fixtures/prepare/typefaces.json")).unwrap(),
    )
    .unwrap()
}

const FIXED_POINT_COORDINATES: f64 = 0.02;

fn same_advance_with_fixed_point_coordinates(got: &Value, want: &Value) -> bool {
    match (got.as_f64(), want.as_f64()) {
        (Some(a), Some(b)) => (a - b).abs() <= FIXED_POINT_COORDINATES,
        _ => got == want,
    }
}

fn without_bounds_three_never_reads(glyph: &Value) -> Value {
    glyph[3].clone()
}

fn counts(o: &str) -> String {
    let tally = |c: &str| o.split(' ').filter(|t| *t == c).count();
    format!(
        "m{} l{} q{} b{}",
        tally("m"),
        tally("l"),
        tally("q"),
        tally("b")
    )
}

#[test]
fn matches_the_frozen_typefaces_glyph_by_glyph() {
    let fixture = fixture();
    let mut differences = Vec::new();
    for face in fixture["faces"].as_array().unwrap() {
        let file = face["file"].as_str().unwrap();
        let weight = face["weight"].as_f64().unwrap();
        let bytes = std::fs::read(root().join("public/fonts/bundled").join(file)).unwrap();
        let made = typeface(&bytes, weight).unwrap();
        let mut font = made.as_object().unwrap().clone();
        let glyphs = font.remove("glyphs").unwrap();
        if Value::Object(font) != face["font"] {
            differences.push(format!("{file}@{weight}: font fields"));
        }
        let expected = face["glyphs"].as_object().unwrap();
        let glyphs = glyphs.as_object().unwrap();
        if expected.len() != glyphs.len() {
            differences.push(format!(
                "{file}@{weight}: {} glyphs, expected {}",
                glyphs.len(),
                expected.len()
            ));
        }
        for (ch, want) in expected {
            let Some(got) = glyphs.get(ch) else {
                differences.push(format!("{file}@{weight} {ch:?}: missing"));
                continue;
            };
            let got = serde_json::json!([
                got["ha"],
                got["x_min"],
                got["x_max"],
                counts(got["o"].as_str().unwrap())
            ]);
            let same = same_advance_with_fixed_point_coordinates(&got[0], &want[0])
                && without_bounds_three_never_reads(&got) == without_bounds_three_never_reads(want);
            if !same {
                differences.push(format!("{file}@{weight} {ch:?}: {got} expected {want}"));
            }
        }
    }
    assert!(
        differences.is_empty(),
        "{} differences:\n{}",
        differences.len(),
        differences
            .iter()
            .take(40)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn draws_the_same_outlines_as_before() {
    let fixture = fixture();
    for (key, want) in fixture["full"].as_object().unwrap() {
        let (file, weight) = key.split_once('@').unwrap();
        let bytes = std::fs::read(root().join("public/fonts/bundled").join(file)).unwrap();
        let made = typeface(&bytes, weight.parse().unwrap()).unwrap();
        let mut differences = Vec::new();
        for (ch, glyph) in want["glyphs"].as_object().unwrap() {
            if made["glyphs"][ch]["o"] != glyph["o"] {
                differences.push(format!(
                    "{ch:?}\n  got  {}\n  want {}",
                    made["glyphs"][ch]["o"], glyph["o"]
                ));
            }
        }
        assert!(
            differences.is_empty(),
            "{key}: {} glyphs differ:\n{}",
            differences.len(),
            differences
                .iter()
                .take(5)
                .cloned()
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}
