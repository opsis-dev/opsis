use std::path::Path;

use opsis_capture::drag_path::{
    bounds_of, bulged_curve, drag_duration, glide_plan, nudge_toward, offset_path, polyline_at,
    thin_path,
};
use opsis_capture::ease::camera_ease;
use opsis_capture::keyboard::{KeyDescription, key_layout, pressed_chord};
use opsis_capture::locators::{
    TextPattern, has_text, label_selector, placeholder_selector, role_selector, test_id_selector,
    text_selector, text_string_selector,
};
use opsis_capture::page_scripts::{CursorUnder, before_first_load, cursor_keyword, hiding_css};
use opsis_capture::still_layout::{
    Area, Size, StillLook, capture_canvas, capture_scale, padded_region, scaled_viewport,
    tallest_shot,
};
use opsis_capture::url_match::{glob_to_regex_pattern, url_matches};
use opsis_types::actions::{Point, Rect, Viewport};
use serde::de::DeserializeOwned;
use serde_json::Value;

const CLOSE: f64 = 1e-9;

fn fixture(name: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/capture")
        .join(name);
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn typed<T: DeserializeOwned>(value: &Value) -> T {
    serde_json::from_value(value.clone()).unwrap()
}

fn list(value: &Value) -> &Vec<Value> {
    value.as_array().unwrap()
}

fn close(a: f64, b: f64, what: &str) {
    assert!(
        (a - b).abs() <= CLOSE * b.abs().max(1.0),
        "{what}: {a} vs {b}"
    );
}

fn close_point(a: Point, b: &Value, what: &str) {
    let b: Point = typed(b);
    close(a.x, b.x, what);
    close(a.y, b.y, what);
}

#[test]
fn camera_easing_matches() {
    for case in list(&fixture("ease.json")) {
        close(
            camera_ease(case["t"].as_f64().unwrap()),
            case["eased"].as_f64().unwrap(),
            &case["t"].to_string(),
        );
    }
}

#[test]
fn drag_paths_match() {
    let f = fixture("drag.json");
    let steps: Vec<f64> = (0..=20).map(|i| f64::from(i) / 20.0).collect();
    for g in list(&f["glides"]) {
        let viewport: Option<Viewport> = typed(&g["viewport"]);
        let travel: Option<f64> = typed(&g["travelMs"]);
        let plan = glide_plan(
            typed(&g["from"]),
            typed(&g["to"]),
            g["scale"].as_f64().unwrap(),
            viewport,
            travel,
        );
        close(
            plan.duration_ms,
            g["durationMs"].as_f64().unwrap(),
            "glide duration",
        );
        close_point(plan.aim, &g["aim"], "glide aim");
        assert_eq!(plan.overshooting, g["overshooting"].as_bool().unwrap());
        for (p, expected) in steps.iter().zip(list(&g["samples"])) {
            close_point((plan.at)(*p), expected, "glide sample");
        }
    }
    for c in list(&f["curves"]) {
        let to: Point = typed(&c["to"]);
        let at = bulged_curve(typed(&c["from"]), to, to);
        for (p, expected) in steps.iter().zip(list(&c["samples"])) {
            close_point(at(*p), expected, "curve sample");
        }
    }
    for c in list(&f["polylines"]) {
        let at = polyline_at(typed(&c["points"]));
        for (p, expected) in steps.iter().zip(list(&c["samples"])) {
            close_point(at(*p), expected, "polyline sample");
        }
    }
    for c in list(&f["durations"]) {
        let points: Vec<Point> = typed(&c["points"]);
        close(
            drag_duration(
                &points,
                c["scale"].as_f64().unwrap(),
                c["dragMs"].as_f64().unwrap(),
            ),
            c["durationMs"].as_f64().unwrap(),
            "drag duration",
        );
    }
    for c in list(&f["nudges"]) {
        close_point(
            nudge_toward(
                typed(&c["from"]),
                typed(&c["to"]),
                c["px"].as_f64().unwrap(),
            ),
            &c["point"],
            "nudge",
        );
    }
    for c in list(&f["offsets"]) {
        let offsets: Vec<Point> = typed(&c["offsets"]);
        assert_eq!(
            offset_path(typed(&c["start"]), &offsets),
            typed::<Vec<Point>>(&c["points"])
        );
    }
    for c in list(&f["thins"]) {
        let points: Vec<Point> = typed(&c["points"]);
        assert_eq!(thin_path(&points), typed::<Vec<Point>>(&c["thinned"]));
    }
    for c in list(&f["bounds"]) {
        let points: Vec<Point> = typed(&c["points"]);
        let rect = bounds_of(&points);
        let expected: Rect = typed(&c["rect"]);
        for (a, b) in [
            (rect.x, expected.x),
            (rect.y, expected.y),
            (rect.w, expected.w),
            (rect.h, expected.h),
        ] {
            close(a, b, "bounds");
        }
    }
}

#[test]
fn still_layout_matches() {
    let f = fixture("still.json");
    for c in list(&f["scales"]) {
        let look: StillLook = typed(&c["look"]);
        let canvas: Size = typed(&c["canvas"]);
        assert_eq!(capture_canvas(typed(&c["project"])), canvas);
        let body: Option<Area> = typed(&c["body"]);
        close(
            capture_scale(
                look,
                typed(&c["region"]),
                canvas,
                body,
                0.86,
                c["least"].as_f64().unwrap(),
            ),
            c["scale"].as_f64().unwrap(),
            &c.to_string(),
        );
    }
    for c in list(&f["tallest"]) {
        let look: StillLook = typed(&c["look"]);
        assert_eq!(
            tallest_shot(
                look,
                c["width"].as_f64().unwrap(),
                capture_canvas(typed(&c["project"]))
            ),
            c["tallest"].as_f64().unwrap()
        );
    }
    for c in list(&f["padded"]) {
        let (region, body) = padded_region(
            typed(&c["box"]),
            c["scroll"]["x"].as_f64().unwrap(),
            c["scroll"]["y"].as_f64().unwrap(),
            c["padding"].as_f64().unwrap(),
        );
        assert_eq!((region, body), (typed(&c["region"]), typed(&c["body"])));
    }
    for c in list(&f["viewports"]) {
        assert_eq!(
            scaled_viewport(typed(&c["logical"]), c["scale"].as_f64().unwrap()),
            typed::<Size>(&c["physical"])
        );
    }
}

#[test]
fn page_helpers_match() {
    let f = fixture("page.json");
    for c in list(&f["cursors"]) {
        let under: CursorUnder = typed(&c["under"]);
        assert_eq!(
            cursor_keyword(&under),
            c["keyword"].as_str().unwrap(),
            "{c}"
        );
    }
    for c in list(&f["chords"]) {
        assert_eq!(
            pressed_chord(c["chord"].as_str().unwrap()),
            c["pressed"].as_str().unwrap()
        );
    }
    for c in list(&f["hiding"]) {
        assert_eq!(
            hiding_css(&typed::<Vec<String>>(&c["selectors"])),
            c["css"].as_str().unwrap()
        );
    }
    for c in list(&f["scripts"]) {
        assert_eq!(
            before_first_load(c["id"].as_str().unwrap(), c["script"].as_str().unwrap()),
            c["text"].as_str().unwrap()
        );
    }
}

#[test]
fn playwright_key_map_selectors_and_globs_match() {
    let f = fixture("playwright.json");
    let layout = key_layout();
    let expected: Vec<(String, KeyDescription)> = list(&f["keyMap"])
        .iter()
        .map(|e| {
            (
                e["name"].as_str().unwrap().to_owned(),
                typed(&e["description"]),
            )
        })
        .collect();
    let actual: Vec<(String, KeyDescription)> =
        layout.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    assert_eq!(actual, expected);
    for c in list(&f["selectors"]) {
        let name = c["name"].as_str().unwrap();
        let (exact, loose) = (TextPattern::exact(name), TextPattern::loose(name));
        let roles: Vec<String> = ["button", "link", "textbox", "combobox", "option", "tab"]
            .iter()
            .map(|r| role_selector(r, &exact))
            .collect();
        assert_eq!(roles, typed::<Vec<String>>(&c["roles"]), "{name}");
        assert_eq!(label_selector(&exact), c["label"].as_str().unwrap());
        assert_eq!(
            placeholder_selector(&loose),
            c["placeholder"].as_str().unwrap()
        );
        assert_eq!(test_id_selector(name), c["testId"].as_str().unwrap());
        assert_eq!(text_selector(&loose), c["text"].as_str().unwrap());
        assert_eq!(has_text(&loose), c["hasText"].as_str().unwrap());
        assert_eq!(
            text_string_selector(name, true),
            c["exactText"].as_str().unwrap()
        );
        assert_eq!(
            text_string_selector(name, false),
            c["looseText"].as_str().unwrap()
        );
    }
    for c in list(&f["globs"]) {
        let glob = c["glob"].as_str().unwrap();
        assert_eq!(
            glob_to_regex_pattern(glob).unwrap(),
            c["pattern"].as_str().unwrap(),
            "{glob}"
        );
        assert_eq!(
            url_matches(c["url"].as_str().unwrap(), glob).unwrap(),
            c["matches"].as_bool().unwrap(),
            "{c}"
        );
    }
}
