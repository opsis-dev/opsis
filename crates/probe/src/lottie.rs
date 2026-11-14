use std::io::{Cursor, Read};

use base64::Engine;
use serde_json::Value;

const ZIP_MAGIC: &[u8; 2] = b"PK";

fn has_extension(name: &str, wanted: &str) -> bool {
    std::path::Path::new(name)
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case(wanted))
}

fn mime_of(name: &str) -> &'static str {
    if has_extension(name, "png") {
        "image/png"
    } else if has_extension(name, "webp") {
        "image/webp"
    } else if has_extension(name, "svg") {
        "image/svg+xml"
    } else {
        "image/jpeg"
    }
}

fn read_entry(archive: &mut zip::ZipArchive<Cursor<&[u8]>>, name: &str) -> Option<Vec<u8>> {
    let mut entry = archive.by_name(name).ok()?;
    let mut out = Vec::new();
    entry.read_to_end(&mut out).ok()?;
    Some(out)
}

fn unpacked(bytes: &[u8]) -> Option<Vec<u8>> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).ok()?;
    let names: Vec<String> = archive
        .file_names()
        .filter_map(Result::ok)
        .map(std::borrow::Cow::into_owned)
        .collect();
    let animation = names
        .iter()
        .find(|n| n.starts_with("animations/") && has_extension(n, "json"))
        .or_else(|| {
            names
                .iter()
                .find(|n| has_extension(n, "json") && !n.ends_with("manifest.json"))
        })?
        .clone();
    let mut json: Value = serde_json::from_slice(&read_entry(&mut archive, &animation)?).ok()?;
    if let Some(assets) = json.get_mut("assets").and_then(Value::as_array_mut) {
        for asset in assets {
            let Some(file) = asset.get("p").and_then(Value::as_str).map(str::to_owned) else {
                continue;
            };
            if file.starts_with("data:") {
                continue;
            }
            let folder = asset
                .get("u")
                .and_then(Value::as_str)
                .unwrap_or("")
                .trim_start_matches('/')
                .to_owned();
            let candidates = [
                format!("images/{file}"),
                format!("{folder}{file}"),
                file.clone(),
            ];
            let Some(data) = candidates
                .iter()
                .find_map(|name| read_entry(&mut archive, name))
            else {
                continue;
            };
            let encoded = base64::engine::general_purpose::STANDARD.encode(data);
            asset["p"] = Value::String(format!("data:{};base64,{encoded}", mime_of(&file)));
            asset["u"] = Value::String(String::new());
            asset["e"] = Value::from(1);
        }
    }
    serde_json::to_vec(&json).ok()
}

pub fn animation_json(bytes: &[u8]) -> Option<Vec<u8>> {
    if bytes.starts_with(ZIP_MAGIC) {
        unpacked(bytes)
    } else {
        Some(bytes.to_vec())
    }
}

fn has_expression(value: &Value) -> bool {
    match value {
        Value::Object(map) => {
            map.get("x").is_some_and(Value::is_string) && map.contains_key("k")
                || map.values().any(has_expression)
        }
        Value::Array(items) => items.iter().any(has_expression),
        _ => false,
    }
}

pub fn uses_expressions(bytes: &[u8]) -> bool {
    animation_json(bytes)
        .and_then(|json| serde_json::from_slice::<Value>(&json).ok())
        .is_some_and(|value| has_expression(&value))
}
