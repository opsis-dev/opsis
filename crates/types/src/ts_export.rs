use regex::Regex;
use specta::Types;
use specta_typescript::Typescript;

use crate::events::RunEvent;
use crate::timeline::TimelineSpec;
use crate::values::{CompileValue, LastRun};
use crate::workflow::Workflow;

pub const BINDINGS_FILE: &str = "apps/ui/src/generated/bindings.ts";

fn tidy(text: &str) -> String {
    let optional_null = Regex::new(r"(\w\?: [^;,{}]+?) \| null(\s*[;,}])").expect("pattern");
    let lines: Vec<String> = text
        .lines()
        .filter(|line| !line.starts_with("//"))
        .map(|line| {
            let line = line.replace("number | null", "number");
            match line.strip_suffix(" | null,") {
                Some(field) if field.starts_with('\t') && field.contains("?: ") => {
                    format!("{field},")
                }
                _ => optional_null.replace_all(&line, "$1$2").into_owned(),
            }
        })
        .collect();
    format!("{}\n", lines.join("\n").trim())
}

fn serialize_side(text: &str) -> String {
    let code: Vec<&str> = text
        .lines()
        .filter(|line| !line.starts_with("//"))
        .collect();
    code.join("\n")
        .split("\n\n")
        .filter(|block| {
            let name = block
                .trim_start()
                .strip_prefix("export type ")
                .and_then(|rest| rest.split(' ').next())
                .unwrap_or("");
            let union = format!("export type {name} = {name}_Serialize | {name}_Deserialize;");
            !name.ends_with("_Deserialize") && block.trim() != union
        })
        .map(str::to_owned)
        .collect::<Vec<_>>()
        .join("\n\n")
        .replace("_Serialize", "")
}

const NUMBER_LITERALS: [(&str, &str, &str); 3] = [
    ("Workflow", "version", "1"),
    ("WorkflowSettings", "fps", "30 | 60"),
    ("LinkedParam", "sign", "1 | -1"),
];

fn number_literals(text: &str) -> String {
    let mut out = text.to_owned();
    for (ty, field, literal) in NUMBER_LITERALS {
        let start = out
            .find(&format!("export type {ty} = {{"))
            .expect("type is exported");
        let end = start + out[start..].find("\n};").expect("type ends");
        let block = out[start..end].replace(
            &format!("\t{field}: number,"),
            &format!("\t{field}: {literal},"),
        );
        out.replace_range(start..end, &block);
    }
    out
}

pub fn boundary_types() -> Types {
    Types::default()
        .register::<Workflow>()
        .register::<TimelineSpec>()
        .register::<CompileValue>()
        .register::<LastRun>()
        .register::<RunEvent>()
        .register::<crate::cards::CardRow>()
}

pub fn render(types: &Types) -> String {
    let ts = Typescript::default()
        .export(types, specta_serde::PhasesFormat)
        .expect("types export");
    number_literals(&tidy(&serialize_side(&ts)))
}
