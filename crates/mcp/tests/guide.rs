use std::collections::BTreeSet;

use opsis_mcp::server::GUIDE;

const OPSIS_LUA: &str = include_str!("../../script/src/opsis.lua");

fn named_after(line: &str, prefix: &str) -> Option<String> {
    let rest = line.strip_prefix(prefix)?;
    let end = rest.find(|c: char| !(c.is_alphanumeric() || c == '_' || c == '.'))?;
    Some(rest[..end].to_owned())
}

fn module_names() -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let mut in_run_methods = false;
    for line in OPSIS_LUA.lines() {
        if let Some(name) = named_after(line, "function opsis.") {
            names.insert(format!("opsis.{name}"));
        } else if let Some(name) = named_after(line, "function nodes.") {
            names.insert(format!("opsis.nodes.{name}"));
        } else if let Some(name) = named_after(line, "opsis.")
            && line[6 + name.len()..].trim_start().starts_with('=')
        {
            names.insert(format!("opsis.{name}"));
        }
        if line.starts_with("run_meta.__index = {") {
            in_run_methods = true;
        } else if in_run_methods && line.starts_with('}') {
            in_run_methods = false;
        } else if in_run_methods
            && line.contains("= function")
            && let Some(name) = named_after(line.trim_start(), "")
        {
            names.insert(format!("run:{name}"));
        }
    }
    names
}

#[test]
fn the_guide_names_every_part_of_the_opsis_module() {
    let names = module_names();
    assert!(names.len() > 50, "found only {names:?}");
    let missing: Vec<&String> = names
        .iter()
        .filter(|name| !GUIDE.contains(name.as_str()))
        .collect();
    assert!(missing.is_empty(), "the guide doesn't name {missing:?}");
}
