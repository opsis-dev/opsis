use opsis_golden::{differences, number, read, text};
use opsis_scene::easing::Easing;
use opsis_scene::interpolate::{Extrapolate, Interpolation, Output, interpolate};
use opsis_scene::random::{Seed, random};
use opsis_scene::spring::{
    REST_THRESHOLD, Spring, SpringConfig, measure_spring, spring, spring_progress,
};
use serde_json::{Value, json};

const FIXTURE: &str = "scene/runtime-maths.json";

fn list(value: &Value) -> &[Value] {
    value.as_array().map_or(&[], Vec::as_slice)
}

fn numbers(value: &Value) -> Vec<f64> {
    list(value).iter().map(number).collect()
}

fn seed(value: &Value) -> Seed {
    value
        .as_str()
        .map_or_else(|| Seed::Number(number(value)), |s| Seed::Text(s.to_owned()))
}

fn randoms(recorded: &Value) -> Value {
    Value::Array(
        list(recorded)
            .iter()
            .map(|r| json!({ "seed": r["seed"], "value": random(seed(&r["seed"])) }))
            .collect(),
    )
}

const SECONDS: [f64; 8] = [0.0, 0.01, 0.1, 0.25, 0.5, 1.0, 2.0, 5.0];

fn springs(recorded: &Value) -> Value {
    Value::Array(
        list(recorded)
            .iter()
            .map(|r| {
                let config: SpringConfig = serde_json::from_value(r["config"].clone()).expect("a spring");
                let stretched: Vec<f64> = [0.0, 3.0, 10.0, 17.0, 25.0, 40.0]
                    .iter()
                    .map(|&frame| {
                        spring(&Spring {
                            config,
                            duration_in_frames: Some(24.0),
                            from: -5.0,
                            to: 5.0,
                            ..Spring::at(frame, 30.0)
                        })
                        .expect("a spring value")
                    })
                    .collect();
                let delayed: Vec<f64> = [0.0, 5.0, 12.0, 30.0]
                    .iter()
                    .map(|&frame| {
                        spring(&Spring {
                            config,
                            delay: 6.0,
                            ..Spring::at(frame, 60.0)
                        })
                        .expect("a spring value")
                    })
                    .collect();
                json!({
                    "config": r["config"],
                    "progress": SECONDS.iter().map(|&s| spring_progress(&config, s)).collect::<Vec<_>>(),
                    "rest30": measure_spring(30.0, &config, REST_THRESHOLD).expect("a rest frame"),
                    "rest60": measure_spring(60.0, &config, REST_THRESHOLD).expect("a rest frame"),
                    "stretched": stretched,
                    "delayed": delayed,
                })
            })
            .collect(),
    )
}

fn named(name: &str, params: &[f64]) -> Easing {
    let param = |i: usize| params.get(i).copied().unwrap_or(0.0);
    match name {
        "linear" => Easing::Linear,
        "ease" => Easing::EASE,
        "quad" => Easing::Quad,
        "cubic" => Easing::Cubic,
        "poly" => Easing::Poly(param(0)),
        "sin" => Easing::Sin,
        "circle" => Easing::Circle,
        "exp" => Easing::Exp,
        "elastic" => Easing::Elastic(param(0)),
        "back" => Easing::Back(param(0)),
        "bounce" => Easing::Bounce,
        "bezier" => Easing::Bezier(param(0), param(1), param(2), param(3)),
        "spring" => Easing::spring(SpringConfig {
            mass: param(0),
            damping: param(1),
            stiffness: param(2),
            overshoot_clamping: false,
        })
        .expect("a spring easing"),
        other => panic!("no easing named {other}"),
    }
}

fn easing_of(recorded: &Value) -> Easing {
    let params = numbers(&recorded["params"]);
    let inner = recorded["inner"].as_str().map(|n| named(n, &[]));
    match (text(&recorded["name"]), inner) {
        ("out", Some(inner)) => Easing::ease_out(inner),
        ("inOut", Some(inner)) => Easing::ease_in_out(inner),
        (name, _) => named(name, &params),
    }
}

const STEPS: [f64; 11] = [0.0, 0.05, 0.1, 0.25, 0.4, 0.5, 0.6, 0.75, 0.9, 0.95, 1.0];

fn easings(recorded: &Value) -> Value {
    Value::Array(
        list(recorded)
            .iter()
            .map(|r| {
                let easing = easing_of(r);
                let mut out = r.clone();
                out["values"] = json!(STEPS.iter().map(|&t| easing.at(t)).collect::<Vec<_>>());
                out
            })
            .collect(),
    )
}

fn extrapolate(value: &Value) -> Extrapolate {
    match value.as_str() {
        Some("clamp") => Extrapolate::Clamp,
        Some("identity") => Extrapolate::Identity,
        Some("wrap") => Extrapolate::Wrap,
        _ => Extrapolate::Extend,
    }
}

fn interpolations(recorded: &Value) -> Value {
    Value::Array(
        list(recorded)
            .iter()
            .map(|r| {
                let options = &r["options"];
                let how = Interpolation {
                    easing: options["easingName"].as_str().map(|n| named(n, &[])),
                    left: extrapolate(&options["extrapolateLeft"]),
                    right: extrapolate(&options["extrapolateRight"]),
                    output: if options["output"].as_str() == Some("perceptual-scale") {
                        Output::PerceptualScale
                    } else {
                        Output::Linear
                    },
                };
                let (input, output) = (numbers(&r["input"]), numbers(&r["output"]));
                let values: Vec<f64> = numbers(&r["at"])
                    .iter()
                    .map(|&x| interpolate(x, &input, &output, &how).expect("an interpolation"))
                    .collect();
                let mut out = r.clone();
                out["values"] = json!(values);
                out
            })
            .collect(),
    )
}

#[test]
fn the_ports_give_the_recorded_values() {
    let recorded = read(FIXTURE);
    let made = json!({
        "randoms": randoms(&recorded["randoms"]),
        "springs": springs(&recorded["springs"]),
        "easings": easings(&recorded["easings"]),
        "interpolations": interpolations(&recorded["interpolations"]),
    });
    let found = differences(&recorded, &made);
    assert!(found.is_empty(), "{}", found.join("\n"));
}
