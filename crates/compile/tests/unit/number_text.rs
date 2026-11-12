use opsis_compile::elements::number_text::{format_axis_value, format_number, group_digits};
use opsis_compile::elements::shared::{GroupSeparator, NumberFormat};
use opsis_golden::{differences, number, read, text};
use serde_json::{Value, json};

const FIXTURE: &str = "scene/number-text.json";

fn list(value: &Value) -> &[Value] {
    value.as_array().map_or(&[], Vec::as_slice)
}

#[test]
fn number_text_gives_what_the_film_page_gives() {
    let recorded = read(FIXTURE);
    let values: Vec<f64> = list(&recorded["values"]).iter().map(number).collect();
    let groups: Vec<&str> = list(&recorded["groups"]).iter().map(text).collect();
    let made = json!({
        "values": recorded["values"],
        "compact": values.iter().map(|v| format_number(*v, NumberFormat::Compact)).collect::<Vec<_>>(),
        "full": values.iter().map(|v| format_number(*v, NumberFormat::Full)).collect::<Vec<_>>(),
        "axis": values.iter().map(|v| format_axis_value(*v)).collect::<Vec<_>>(),
        "groups": recorded["groups"],
        "grouped": groups.iter().map(|g| group_digits(g, GroupSeparator::Comma)).collect::<Vec<_>>(),
    });
    let found = differences(&recorded, &made);
    assert!(found.is_empty(), "{}", found.join("\n"));
}
