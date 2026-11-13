use opsis_compile::elements::camera::click_log::{ClickEvent, ClickLog, to_click_log};
use opsis_compile::elements::camera::track::{
    CameraKey, CameraPose, Tuning, ZoomState, build_camera_track, log_time_s, pose_at, zoom_at,
};
use opsis_golden::{differences, read};
use opsis_types::actions::ActionLog;
use opsis_types::recording::CameraTuning;
use serde_json::{Map, Value, json};

const FIXTURE: &str = "scene/camera-track.json";
const FPS: f64 = 30.0;

fn put(map: &mut Map<String, Value>, key: &str, value: Option<Value>) {
    if let Some(value) = value {
        map.insert(key.to_owned(), value);
    }
}

fn click(c: &ClickEvent) -> Value {
    let mut map = Map::new();
    let number = |v: Option<f64>| v.map(Value::from);
    put(&mut map, "label", c.label.clone().map(Value::from));
    map.insert("tMs".into(), json!(c.t_ms));
    put(&mut map, "tDepartMs", number(c.t_depart_ms));
    put(&mut map, "tDownMs", number(c.t_down_ms));
    put(&mut map, "tSecondDownMs", number(c.t_second_down_ms));
    put(&mut map, "button", c.button.map(|b| json!(b)));
    put(&mut map, "typeEndMs", number(c.type_end_ms));
    map.insert("x".into(), json!(c.x));
    map.insert("y".into(), json!(c.y));
    put(&mut map, "rect", c.rect.map(|r| json!(r)));
    put(&mut map, "cluster", c.cluster.clone().map(Value::from));
    put(&mut map, "zoom", c.zoom.map(Value::from));
    put(&mut map, "zoomScale", number(c.zoom_scale));
    put(&mut map, "tUpMs", number(c.t_up_ms));
    put(&mut map, "path", c.path.as_ref().map(|p| json!(p)));
    put(&mut map, "drop", c.drop.map(|r| json!(r)));
    Value::Object(map)
}

fn clicks(log: &ClickLog) -> Value {
    let mut map = Map::new();
    map.insert("name".into(), json!(log.name));
    map.insert("viewport".into(), json!(log.viewport));
    map.insert("durationMs".into(), json!(log.duration_ms));
    put(
        &mut map,
        "cursorTrack",
        log.cursor_track.as_ref().map(|t| json!(t)),
    );
    map.insert("clicks".into(), log.clicks.iter().map(click).collect());
    map.insert(
        "keys".into(),
        log.keys
            .iter()
            .map(|k| {
                let mut key = Map::new();
                key.insert("tMs".into(), json!(k.t_ms));
                key.insert("chord".into(), json!(k.chord));
                put(&mut key, "label", k.label.clone().map(Value::from));
                Value::Object(key)
            })
            .collect(),
    );
    Value::Object(map)
}

fn pose(p: CameraPose) -> Value {
    json!({ "scale": p.scale, "cx": p.cx, "cy": p.cy })
}

fn zoom(z: ZoomState) -> Value {
    json!({ "scale": z.scale, "translateX": z.translate_x, "translateY": z.translate_y })
}

fn keys(list: &[CameraKey]) -> Value {
    list.iter()
        .map(|k| json!({ "t": k.t, "pose": pose(k.pose) }))
        .collect()
}

#[test]
fn camera_tracks_match_what_the_film_page_makes() {
    let recorded = read(FIXTURE);
    let cases = recorded["cases"].as_array().map_or(&[][..], Vec::as_slice);
    let made = json!({
        "cases": cases.iter().map(|case| {
            let log: ActionLog = serde_json::from_value(case["log"].clone()).expect("an action log");
            let camera: CameraTuning = serde_json::from_value(case["tuning"].clone()).expect("a tuning");
            let tuning = Tuning { camera, narrow: case["tuning"]["narrow"].as_f64() };
            let speed = case["speed"].as_f64().expect("a speed");
            let zoom_actions = case["zoomActions"].as_bool().expect("zoom actions");
            let name = case["clicks"]["name"].as_str().expect("a name");
            let click_log = to_click_log(&log, name, &[], zoom_actions);
            let track = build_camera_track(&click_log, speed, &tuning);
            let samples: Vec<Value> = case["samples"].as_array().map_or(&[][..], Vec::as_slice).iter().map(|s| {
                let frame = s["frame"].as_f64().expect("a frame");
                json!({
                    "frame": frame,
                    "pose": pose(pose_at(&track, frame, FPS, speed, &tuning)),
                    "zoom": zoom(zoom_at(&click_log, &track, log_time_s(frame, FPS, speed), &tuning, 0.04, 0.9)),
                })
            }).collect();
            json!({
                "log": case["log"],
                "tuning": case["tuning"],
                "speed": speed,
                "zoomActions": zoom_actions,
                "clicks": clicks(&click_log),
                "keys": keys(&track),
                "samples": samples,
            })
        }).collect::<Vec<_>>(),
    });
    let found = differences(&recorded, &made);
    assert!(
        found.is_empty(),
        "{}",
        found
            .iter()
            .take(40)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}
