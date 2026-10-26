use std::path::Path;

use opsis_prepare::typeface::typeface;
use serde_json::{Map, Value, json};

const TYPEFACES: &str = "prepare/typefaces.json";

fn made(file: &str, weight: f64) -> Value {
    let bundled = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../public/fonts/bundled");
    let bytes = std::fs::read(bundled.join(file)).unwrap_or_else(|e| panic!("{file}: {e}"));
    typeface(&bytes, weight).unwrap_or_else(|e| panic!("{file}@{weight}: {e}"))
}

fn commands(outline: &str) -> String {
    let tally = |c: &str| outline.split(' ').filter(|t| *t == c).count();
    format!(
        "m{} l{} q{} b{}",
        tally("m"),
        tally("l"),
        tally("q"),
        tally("b")
    )
}

fn face(stored: &Value) -> Value {
    let file = stored["file"].as_str().expect("a font file");
    let weight = stored["weight"].as_f64().expect("a weight");
    let mut font = made(file, weight).as_object().expect("a typeface").clone();
    let glyphs: Map<String, Value> = font
        .remove("glyphs")
        .and_then(|g| g.as_object().cloned())
        .expect("glyphs")
        .into_iter()
        .map(|(ch, g)| {
            let outline = g["o"].as_str().unwrap_or_default();
            let summary = json!([g["ha"], g["x_min"], g["x_max"], commands(outline)]);
            (ch, summary)
        })
        .collect();
    json!({ "file": file, "weight": weight, "font": font, "glyphs": glyphs })
}

fn outlines(key: &str) -> Value {
    let (file, weight) = key.split_once('@').expect("file@weight");
    let typeface = made(file, weight.parse().expect("a weight"));
    let glyphs: Map<String, Value> = typeface["glyphs"]
        .as_object()
        .expect("glyphs")
        .iter()
        .map(|(ch, g)| (ch.clone(), json!({ "o": g["o"] })))
        .collect();
    json!({ "glyphs": glyphs })
}

#[test]
fn typefaces_are_drawn_as_recorded() {
    let stored = opsis_golden::read(TYPEFACES);
    let faces: Vec<Value> = stored["faces"]
        .as_array()
        .expect("faces")
        .iter()
        .map(face)
        .collect();
    let full: Map<String, Value> = stored["full"]
        .as_object()
        .expect("full outlines")
        .keys()
        .map(|key| (key.clone(), outlines(key)))
        .collect();
    opsis_golden::check(TYPEFACES, &json!({ "faces": faces, "full": full }));
}
