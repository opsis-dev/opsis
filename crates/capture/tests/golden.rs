use opsis_capture::drag_path::{
    bounds_of, bulged_curve, drag_duration, glide_plan, nudge_toward, offset_path, polyline_at,
    thin_path,
};
use opsis_capture::ease::camera_ease;
use opsis_capture::keyboard::{key_layout, pressed_chord};
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
use opsis_types::actions::{Point, Viewport};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

fn typed<T: DeserializeOwned>(value: &Value) -> T {
    serde_json::from_value(value.clone()).unwrap_or_else(|e| panic!("{e}: {value}"))
}

fn number(value: &Value) -> f64 {
    value
        .as_f64()
        .unwrap_or_else(|| panic!("not a number: {value}"))
}

fn text(value: &Value) -> &str {
    value
        .as_str()
        .unwrap_or_else(|| panic!("not a string: {value}"))
}

fn cases(value: &Value, record: impl Fn(&Value) -> Value) -> Value {
    Value::Array(
        value
            .as_array()
            .unwrap_or_else(|| panic!("not a list: {value}"))
            .iter()
            .map(record)
            .collect(),
    )
}

fn with(case: &Value, outputs: Value) -> Value {
    let mut case = case.clone();
    for (key, value) in outputs.as_object().expect("outputs") {
        case[key] = value.clone();
    }
    case
}

fn sampled(at: impl Fn(f64) -> Point) -> Value {
    json!(
        (0..=20)
            .map(|i| at(f64::from(i) / 20.0))
            .collect::<Vec<_>>()
    )
}

#[test]
fn camera_easing_is_as_recorded() {
    let made = cases(&opsis_golden::read("capture/ease.json"), |case| {
        with(case, json!({ "eased": camera_ease(number(&case["t"])) }))
    });
    opsis_golden::check("capture/ease.json", &made);
}

#[test]
fn drag_paths_are_as_recorded() {
    let f = opsis_golden::read("capture/drag.json");
    let glides = cases(&f["glides"], |g| {
        let viewport: Option<Viewport> = typed(&g["viewport"]);
        let travel: Option<f64> = typed(&g["travelMs"]);
        let plan = glide_plan(
            typed(&g["from"]),
            typed(&g["to"]),
            number(&g["scale"]),
            viewport,
            travel,
        );
        let outputs = json!({
            "durationMs": plan.duration_ms,
            "aim": plan.aim,
            "overshooting": plan.overshooting,
            "samples": sampled(&plan.at),
        });
        with(g, outputs)
    });
    let curves = cases(&f["curves"], |c| {
        let to: Point = typed(&c["to"]);
        let at = bulged_curve(typed(&c["from"]), to, to);
        with(c, json!({ "samples": sampled(at) }))
    });
    let polylines = cases(&f["polylines"], |c| {
        with(
            c,
            json!({ "samples": sampled(polyline_at(typed(&c["points"]))) }),
        )
    });
    let durations = cases(&f["durations"], |c| {
        let points: Vec<Point> = typed(&c["points"]);
        let ms = drag_duration(&points, number(&c["scale"]), number(&c["dragMs"]));
        with(c, json!({ "durationMs": ms }))
    });
    let nudges = cases(&f["nudges"], |c| {
        let point = nudge_toward(typed(&c["from"]), typed(&c["to"]), number(&c["px"]));
        with(c, json!({ "point": point }))
    });
    let offsets = cases(&f["offsets"], |c| {
        let offsets: Vec<Point> = typed(&c["offsets"]);
        with(
            c,
            json!({ "points": offset_path(typed(&c["start"]), &offsets) }),
        )
    });
    let thins = cases(&f["thins"], |c| {
        let points: Vec<Point> = typed(&c["points"]);
        with(c, json!({ "thinned": thin_path(&points) }))
    });
    let bounds = cases(&f["bounds"], |c| {
        let points: Vec<Point> = typed(&c["points"]);
        with(c, json!({ "rect": bounds_of(&points) }))
    });
    let made = json!({
        "glides": glides,
        "curves": curves,
        "polylines": polylines,
        "durations": durations,
        "nudges": nudges,
        "offsets": offsets,
        "thins": thins,
        "bounds": bounds,
    });
    opsis_golden::check("capture/drag.json", &made);
}

#[test]
fn still_layouts_are_as_recorded() {
    let f = opsis_golden::read("capture/still.json");
    let scales = cases(&f["scales"], |c| {
        let look: StillLook = typed(&c["look"]);
        let canvas: Size = capture_canvas(typed(&c["project"]));
        let body: Option<Area> = typed(&c["body"]);
        let scale = capture_scale(
            look,
            typed(&c["region"]),
            canvas,
            body,
            0.86,
            number(&c["least"]),
        );
        with(c, json!({ "canvas": canvas, "scale": scale }))
    });
    let tallest = cases(&f["tallest"], |c| {
        let look: StillLook = typed(&c["look"]);
        let shot = tallest_shot(
            look,
            number(&c["width"]),
            capture_canvas(typed(&c["project"])),
        );
        with(c, json!({ "tallest": shot }))
    });
    let padded = cases(&f["padded"], |c| {
        let (region, body) = padded_region(
            typed(&c["box"]),
            number(&c["scroll"]["x"]),
            number(&c["scroll"]["y"]),
            number(&c["padding"]),
        );
        with(c, json!({ "region": region, "body": body }))
    });
    let viewports = cases(&f["viewports"], |c| {
        let physical = scaled_viewport(typed(&c["logical"]), number(&c["scale"]));
        with(c, json!({ "physical": physical }))
    });
    let made =
        json!({ "scales": scales, "tallest": tallest, "padded": padded, "viewports": viewports });
    opsis_golden::check("capture/still.json", &made);
}

#[test]
fn page_helpers_are_as_recorded() {
    let f = opsis_golden::read("capture/page.json");
    let cursors = cases(&f["cursors"], |c| {
        let under: CursorUnder = typed(&c["under"]);
        with(c, json!({ "keyword": cursor_keyword(&under) }))
    });
    let chords = cases(&f["chords"], |c| {
        with(c, json!({ "pressed": pressed_chord(text(&c["chord"])) }))
    });
    let hiding = cases(&f["hiding"], |c| {
        let selectors: Vec<String> = typed(&c["selectors"]);
        with(c, json!({ "css": hiding_css(&selectors) }))
    });
    let scripts = cases(&f["scripts"], |c| {
        let made = before_first_load(text(&c["id"]), text(&c["script"]));
        with(c, json!({ "text": made }))
    });
    let made =
        json!({ "cursors": cursors, "chords": chords, "hiding": hiding, "scripts": scripts });
    opsis_golden::check("capture/page.json", &made);
}

fn selectors(name: &str) -> Value {
    let (exact, loose) = (TextPattern::exact(name), TextPattern::loose(name));
    let roles: Vec<String> = ["button", "link", "textbox", "combobox", "option", "tab"]
        .iter()
        .map(|r| role_selector(r, &exact))
        .collect();
    json!({
        "roles": roles,
        "label": label_selector(&exact),
        "placeholder": placeholder_selector(&loose),
        "testId": test_id_selector(name),
        "text": text_selector(&loose),
        "hasText": has_text(&loose),
        "exactText": text_string_selector(name, true),
        "looseText": text_string_selector(name, false),
    })
}

#[test]
fn keys_selectors_and_address_patterns_are_as_recorded() {
    let f = opsis_golden::read("capture/locators.json");
    let key_map: Vec<Value> = key_layout()
        .iter()
        .map(|(name, description)| json!({ "name": name, "description": description }))
        .collect();
    let selectors = cases(&f["selectors"], |c| with(c, selectors(text(&c["name"]))));
    let globs = cases(&f["globs"], |c| {
        let glob = text(&c["glob"]);
        let outputs = json!({
            "pattern": glob_to_regex_pattern(glob).expect("a pattern"),
            "matches": url_matches(text(&c["url"]), glob).expect("a match"),
        });
        with(c, outputs)
    });
    let made = json!({ "keyMap": key_map, "selectors": selectors, "globs": globs });
    opsis_golden::check("capture/locators.json", &made);
}
