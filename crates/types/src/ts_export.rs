use regex::Regex;
use specta::Types;
use specta_typescript::Typescript;

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

pub fn render_plain(types: &Types) -> String {
    let ts = Typescript::default()
        .export(types, specta_serde::PhasesFormat)
        .expect("types export");
    tidy(&serialize_side(&ts))
}
