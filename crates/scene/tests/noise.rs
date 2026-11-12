use opsis_golden::{differences, number, read, text};
use opsis_scene::noise::NoiseField;
use serde_json::{Value, json};

const FIXTURE: &str = "scene/noise-field.json";

fn list(value: &Value) -> &[Value] {
    value.as_array().map_or(&[], Vec::as_slice)
}

#[test]
fn the_noise_field_gives_what_the_film_page_gives() {
    let recorded = read(FIXTURE);
    let made = Value::Array(
        list(&recorded)
            .iter()
            .map(|r| {
                let field = NoiseField::new(text(&r["seed"]));
                let values: Vec<f64> = list(&r["points"])
                    .iter()
                    .map(|p| {
                        let [x, y, z] = [0, 1, 2].map(|i| number(&p[i]));
                        field.at(x, y, z)
                    })
                    .collect();
                json!({ "seed": r["seed"], "points": r["points"], "values": values })
            })
            .collect(),
    );
    let found = differences(&recorded, &made);
    assert!(found.is_empty(), "{}", found.join("\n"));
}
