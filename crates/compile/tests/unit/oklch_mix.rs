use opsis_compile::elements::oklch_mix::{interpolate_colors, mix_oklch};
use opsis_golden::{differences, number, read, text};
use serde_json::{Value, json};

const FIXTURE: &str = "scene/oklch-mix.json";
const STEPS: [f64; 7] = [0.0, 0.1, 0.25, 0.5, 0.75, 0.9, 1.0];

fn list(value: &Value) -> &[Value] {
    value.as_array().map_or(&[], Vec::as_slice)
}

#[test]
fn colour_mixing_gives_what_the_film_page_gives() {
    let recorded = read(FIXTURE);
    let mixes: Vec<Value> = list(&recorded["mixes"])
        .iter()
        .map(|m| {
            let (a, b) = (text(&m["a"]), text(&m["b"]));
            let values: Vec<String> = STEPS.iter().map(|&t| mix_oklch(a, b, t)).collect();
            json!({ "a": a, "b": b, "values": values })
        })
        .collect();
    let ramps: Vec<Value> = list(&recorded["ramps"])
        .iter()
        .map(|r| {
            let range: Vec<f64> = list(&r["range"]).iter().map(number).collect();
            let colors: Vec<&str> = list(&r["colors"]).iter().map(text).collect();
            let values: Vec<String> = list(&r["at"])
                .iter()
                .map(|x| interpolate_colors(number(x), &range, &colors).expect("a colour"))
                .collect();
            json!({ "range": r["range"], "colors": r["colors"], "at": r["at"], "values": values })
        })
        .collect();
    let found = differences(&recorded, &json!({ "mixes": mixes, "ramps": ramps }));
    assert!(found.is_empty(), "{}", found.join("\n"));
}
