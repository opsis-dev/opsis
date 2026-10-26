use opsis_compile::elements::lengths::animation_length;
use opsis_compile::elements::{ElementMeta, element_meta, elements};
use serde_json::{Map, Value, json};

fn meta(key: &str) -> &'static ElementMeta {
    element_meta(key).unwrap_or_else(|| panic!("{key}"))
}

fn with(meta: &ElementMeta, changes: Value) -> Map<String, Value> {
    let mut settings = meta.default_props.clone();
    if let Value::Object(changes) = changes {
        settings.extend(changes);
    }
    settings
}

#[test]
fn keeps_the_usual_length_for_the_usual_settings_unless_that_cuts_the_animation_off() {
    let changes: Vec<(&str, i64)> = elements()
        .iter()
        .map(|e| {
            (
                e.key.as_str(),
                i64::from(animation_length(e, &e.default_props, 30))
                    - i64::from(e.default_duration_in_frames),
            )
        })
        .collect();
    assert_eq!(
        changes.iter().filter(|(_, c)| *c < 0).count(),
        0,
        "{changes:?}"
    );
    let longer: Vec<&str> = changes
        .iter()
        .filter(|(_, c)| *c > 0)
        .map(|(k, _)| *k)
        .collect();
    assert_eq!(
        longer,
        [
            "remocn/terminal-simulator",
            "remocn/signup-flow",
            "remocn/release-teaser",
            "remocn/launch-anything",
            "remotion-ui/notification-stack"
        ]
    );
}

#[test]
fn measures_timing_in_seconds_at_the_projects_frame_rate() {
    let element = meta("remotion-ui/changelog-entry");
    assert_eq!(
        animation_length(element, &element.default_props, 60),
        2 * element.default_duration_in_frames
    );
}

#[test]
fn keeps_elements_made_for_30_frames_a_second_as_long_in_seconds_at_60() {
    for key in [
        "snapcn/status-cycle",
        "snapcn/laptop-frame",
        "remocn/soft-blur-in",
    ] {
        let element = meta(key);
        assert_eq!(
            animation_length(element, &element.default_props, 60),
            2 * element.default_duration_in_frames,
            "{key}"
        );
    }
}

#[test]
fn runs_an_element_with_no_timing_of_its_own_for_its_usual_length_at_its_speed() {
    let element = meta("remocn/strikethrough-replace");
    assert_eq!(
        animation_length(element, &with(element, json!({ "speed": 0.5 })), 30),
        2 * element.default_duration_in_frames
    );
}

#[test]
fn plays_faster_at_a_higher_speed() {
    let element = meta("remocn/soft-blur-in");
    let usual = animation_length(element, &element.default_props, 30);
    assert!(animation_length(element, &with(element, json!({ "speed": 2 })), 30) < usual);
}
