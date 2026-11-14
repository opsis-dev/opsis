use opsis_compile::elements::transition::path_morph::{Segment, prepare_morph};
use opsis_compile::elements::transition::transition_morph_shape::MorphShape;
use opsis_golden::{differences, read};
use serde_json::{Value, json};

const FIXTURE: &str = "scene/path-morph.json";

fn segments(list: &[Segment]) -> Value {
    list.iter()
        .map(|segment| {
            let command = if matches!(segment, Segment::Move(_)) {
                "M"
            } else {
                "C"
            };
            let mut row = vec![json!(command)];
            row.extend(segment.numbers().into_iter().map(Value::from));
            Value::Array(row)
        })
        .collect()
}

fn shape(value: &Value) -> MorphShape {
    serde_json::from_value(value.clone()).unwrap_or_else(|e| panic!("{value}: {e}"))
}

#[test]
fn shape_morphs_match_the_recorded_paths() {
    let recorded = read(FIXTURE);
    let pairs = recorded["pairs"].as_array().map_or(&[][..], Vec::as_slice);
    let made = json!({
        "pairs": pairs.iter().map(|pair| {
            let morph = prepare_morph(shape(&pair["from"]), shape(&pair["to"]));
            json!({
                "from": pair["from"],
                "to": pair["to"],
                "start": segments(&morph.from),
                "end": segments(&morph.to),
                "closed": morph.closed,
            })
        }).collect::<Vec<_>>(),
    });
    let found = differences(&recorded, &made);
    assert!(found.is_empty(), "{}", found.join("\n"));
}
