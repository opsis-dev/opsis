#![allow(
    dead_code,
    reason = "each test binary uses a different part of the shared helpers"
)]

use opsis_cases::{HEIGHT, WIDTH, oracle_dir};
use opsis_layout::fonts::Fonts;
use opsis_layout::output::{Laid, PlacedKind};
use opsis_layout::{Images, Viewport, lay_out};
use opsis_scene::node::Node;
use serde_json::Value;

fn read_json(path: &std::path::Path) -> Value {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

pub fn case_fonts() -> Fonts {
    let mut fonts = Fonts::new();
    for (name, path) in opsis_cases::case_font_files() {
        let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        fonts
            .register(&name, &bytes)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
    }
    for (subset, path) in opsis_cases::fallback_font_files() {
        let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        fonts
            .register_fallback(&subset, &bytes)
            .unwrap_or_else(|e| panic!("{subset}: {e}"));
    }
    fonts
}

#[derive(Debug, Clone, Copy)]
pub struct Tolerance {
    pub boxes: f64,
    pub text: f64,
}

pub const LAYOUT: Tolerance = Tolerance {
    boxes: 0.5,
    text: 1.5,
};

fn rect(value: &Value) -> [f64; 4] {
    ["x", "y", "width", "height"].map(|k| value[k].as_f64().unwrap_or(0.0))
}

fn path_of(value: &Value) -> Vec<usize> {
    value["path"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_u64().and_then(|n| usize::try_from(n).ok()))
        .collect()
}

pub fn differences(
    id: &str,
    laid: &Laid,
    tolerance: Tolerance,
    skipped: &[&[usize]],
) -> Vec<String> {
    let expected = read_json(&oracle_dir().join("cases").join(format!("{id}.boxes.json")));
    let mut found = Vec::new();
    for entry in expected.as_array().into_iter().flatten() {
        let path = path_of(entry);
        if skipped.iter().any(|prefix| path.starts_with(prefix)) {
            continue;
        }
        let is_text = entry["kind"].as_str() == Some("text");
        let want = rect(&entry["rect"]);
        let Some(placed) = laid.at(&path) else {
            if want.iter().any(|v| v.abs() > f64::EPSILON) {
                found.push(format!("{path:?} {}: missing", entry["tag"]));
            }
            continue;
        };
        let got = [
            placed.client.x,
            placed.client.y,
            placed.client.width,
            placed.client.height,
        ];
        let slack = if is_text || matches!(placed.kind, PlacedKind::Text { .. }) {
            tolerance.text
        } else {
            tolerance.boxes
        };
        let apart =
            |a: &[f64; 4], b: &[f64; 4]| a.iter().zip(b).any(|(x, y)| (x - y).abs() > slack);
        let lines = match &placed.kind {
            PlacedKind::Text { lines, .. } => Some((
                merged_lines(entry),
                lines
                    .iter()
                    .map(|r| {
                        let b = placed.matrix.bounds(*r);
                        [b.x, b.y, b.width, b.height]
                    })
                    .collect::<Vec<_>>(),
            )),
            PlacedKind::Element(_) => None,
        };
        let lines_apart = lines.as_ref().is_some_and(|(wanted, laid)| {
            wanted.len() != laid.len() || wanted.iter().zip(laid).any(|(a, b)| apart(a, b))
        });
        if apart(&want, &got) || lines_apart {
            found.push(format!(
                "{path:?} {}: want {want:?} got {got:?}",
                entry["tag"].as_str().unwrap_or_default()
            ));
            if let Some((wanted, laid)) = lines {
                found.push(format!("  lines want {wanted:?}\n  lines got  {laid:?}"));
            }
        }
    }
    found
}

fn merged_lines(entry: &Value) -> Vec<[f64; 4]> {
    let mut lines: Vec<[f64; 4]> = Vec::new();
    for fragment in entry["lines"].as_array().into_iter().flatten().map(rect) {
        if fragment[2] <= 0.0 {
            continue;
        }
        match lines
            .iter_mut()
            .find(|line| (line[1] - fragment[1]).abs() < 0.5)
        {
            Some(line) => {
                let left = line[0].min(fragment[0]);
                let right = (line[0] + line[2]).max(fragment[0] + fragment[2]);
                line[0] = left;
                line[2] = right - left;
            }
            None => lines.push(fragment),
        }
    }
    lines
}

pub fn check(id: &str, root: &Node, tolerance: Tolerance) -> Vec<String> {
    check_skipping(id, root, tolerance, &[])
}

pub fn app_picture_size(source: &str) -> Option<(f64, f64)> {
    let probe = opsis_probe::probe(&opsis_cases::app_file(source)?).ok()?;
    let picture = probe.video?;
    Some((f64::from(picture.width), f64::from(picture.height)))
}

pub fn check_skipping(
    id: &str,
    root: &Node,
    tolerance: Tolerance,
    skipped: &[&[usize]],
) -> Vec<String> {
    let mut fonts = case_fonts();
    let laid = lay_out(
        root,
        Viewport {
            width: WIDTH,
            height: HEIGHT,
        },
        &mut fonts,
        &Images {
            size_of: &app_picture_size,
        },
    );
    differences(id, &laid, tolerance, skipped)
}
