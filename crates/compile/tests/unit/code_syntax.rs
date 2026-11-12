use opsis_compile::elements::code_syntax::tokenize_code;
use opsis_golden::{differences, read, text};
use serde_json::{Value, json};

const FIXTURE: &str = "scene/code-tokens.json";

#[test]
fn code_tokens_match_what_the_film_page_makes() {
    let recorded = read(FIXTURE);
    let samples: Vec<&str> = recorded["samples"]
        .as_array()
        .map_or(&[][..], Vec::as_slice)
        .iter()
        .map(text)
        .collect();
    let made = json!({
        "samples": recorded["samples"],
        "tokens": samples.iter().map(|sample| {
            let lines: Vec<&str> = sample.split('\n').collect();
            tokenize_code(&lines)
                .iter()
                .map(|line| line.iter().map(|t| json!({ "text": t.text, "kind": t.kind.id() })).collect::<Vec<Value>>())
                .collect::<Vec<_>>()
        }).collect::<Vec<_>>(),
    });
    let found = differences(&recorded, &made);
    assert!(found.is_empty(), "{}", found.join("\n"));
}
