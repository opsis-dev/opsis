use opsis_compile::asset_ref::{
    RefTarget, base_name, extension_of, file_version_key, join_url, kind_from_path, project_ref,
    project_rel_path, ref_target,
};
use opsis_compile::catalog::{catalog_entry, node_catalog};
use opsis_compile::color::{parse_color, project_swatches, same_color};
use opsis_compile::registry::registry;
use opsis_compile::text::{
    is_locale, is_time_zone, is_wall_time, non_blank_lines, parse_number_text, project_folder_name,
    wall_time_in,
};
use opsis_compile::vars::{Token, Vars, interpolate_text, rename_variable, strings_in, tokenize};
use opsis_types::secrets::{secret_file_name, secret_name};
use opsis_types::values::{LastRun, MediaKind, NodeResult};
use opsis_types::workflow::Workflow;
use serde_json::json;

#[test]
fn joins_a_base_and_an_encoded_path_with_or_without_the_slash() {
    assert_eq!(
        join_url("opsis-file://project/", "assets/my clip#1.mp4"),
        "opsis-file://project/assets/my%20clip%231.mp4"
    );
    assert_eq!(
        join_url("http://127.0.0.1:5000", "a b.png"),
        "http://127.0.0.1:5000/a%20b.png"
    );
}

#[test]
fn sorts_refs_into_project_app_web_and_absolute_files() {
    assert_eq!(
        project_rel_path("project:/assets/a.mp4").as_deref(),
        Some("assets/a.mp4")
    );
    assert_eq!(
        project_rel_path("assets/a.mp4").as_deref(),
        Some("assets/a.mp4")
    );
    assert_eq!(project_rel_path("app:sfx/a.wav"), None);
    assert_eq!(
        ref_target("project:assets/a.png"),
        RefTarget::Project("assets/a.png".into())
    );
    assert_eq!(
        ref_target("assets/a.png"),
        RefTarget::Project("assets/a.png".into())
    );
    assert_eq!(
        ref_target("app:backdrops/a.jpg"),
        RefTarget::App("backdrops/a.jpg".into())
    );
    assert_eq!(
        ref_target("https://x.test/a.png"),
        RefTarget::External("https://x.test/a.png".into())
    );
    assert_eq!(
        ref_target("/abs/a.png"),
        RefTarget::Absolute("/abs/a.png".into())
    );
    assert_eq!(
        ref_target("C:\\clips\\a.mp4"),
        RefTarget::Absolute("C:\\clips\\a.mp4".into())
    );
    assert_eq!(project_ref("assets/a.png"), "project:assets/a.png");
    assert_eq!(project_ref("project:assets/a.png"), "project:assets/a.png");
    assert_eq!(project_ref("/abs/a.png"), "/abs/a.png");
    assert_eq!(project_ref("app:sfx/a.wav"), "app:sfx/a.wav");
    assert_eq!(
        [
            base_name("project:assets/a.mp4"),
            base_name("exports/site/"),
            base_name("clip.mp4")
        ],
        ["a.mp4", "site", "clip.mp4"]
    );
}

#[test]
fn reads_the_last_segments_extension_in_lowercase() {
    assert_eq!(
        [
            extension_of("assets/Clip.MP4"),
            extension_of("README"),
            extension_of("dir.v2/clip"),
            extension_of(".gitignore")
        ],
        ["mp4", "", "", ""]
    );
}

#[test]
fn keys_a_project_file_the_same_however_its_path_is_written() {
    assert_eq!(
        file_version_key("project:assets/./a.png").as_deref(),
        Some("assets/a.png")
    );
    assert_eq!(
        file_version_key("project:/assets//a.png").as_deref(),
        Some("assets/a.png")
    );
    assert_eq!(
        file_version_key("assets\\a.png").as_deref(),
        Some("assets/a.png")
    );
    assert_eq!(file_version_key("app:backdrops/a.png"), None);
}

#[test]
fn reads_sound_picture_and_video_files_by_extension() {
    for ext in [
        "mp3", "wav", "opus", "aiff", "aif", "weba", "oga", "caf", "wma", "mka",
    ] {
        assert_eq!(
            kind_from_path(&format!("assets/take.{}", ext.to_uppercase())),
            MediaKind::Audio,
            "{ext}"
        );
    }
    assert_eq!(
        [
            kind_from_path("a/b.png"),
            kind_from_path("a/b.mkv"),
            kind_from_path("a/b.webm")
        ],
        [MediaKind::Image, MediaKind::Video, MediaKind::Video]
    );
}

#[test]
fn keeps_the_alpha_of_colours_and_refuses_variables_and_non_colours() {
    let alpha = |c: &str| parse_color(c).map(|h| h.a);
    assert!((alpha("rgba(10,10,12,0.72)").unwrap() - 0.72).abs() < 0.01);
    assert!((alpha("#ffffff80").unwrap() - 0.5).abs() < 0.01);
    assert!((alpha("oklch(1 0 0 / 10%)").unwrap() - 0.1).abs() < 0.01);
    assert_eq!(alpha("transparent"), Some(0.0));
    for text in ["var(--accent)", "", "linear-gradient(red, blue)"] {
        assert!(parse_color(text).is_none(), "{text}");
    }
    assert!(same_color("#ff0000", "rgb(255, 0, 0)"));
}

#[test]
fn lists_colours_used_in_settings_most_used_first_once_each() {
    let a = json!({ "color": "#ff0000", "text": "Hello #1" });
    let b = json!({ "colors": ["#FF0000", "rgb(0, 0, 255)"], "nested": { "ink": "#ff0000" } });
    assert_eq!(
        project_swatches(&[&a, &b], 24),
        ["#ff0000", "rgb(0, 0, 255)"]
    );
}

#[test]
fn reads_numbers_and_lines_typed_into_text_fields() {
    assert_eq!(parse_number_text("1,5"), Some(1.5));
    assert_eq!(parse_number_text(" 2 "), Some(2.0));
    assert_eq!(
        [
            parse_number_text(""),
            parse_number_text("   "),
            parse_number_text("12 dB")
        ],
        [None, None, None]
    );
    assert_eq!(
        opsis_compile::nodes::timeline::parse_number_list("100, x,250"),
        [100.0, 0.0, 250.0]
    );
    assert_eq!(opsis_compile::nodes::timeline::parse_number_list(""), [0.0]);
    assert_eq!(non_blank_lines(" a \n\n  \nb"), ["a", "b"]);
}

fn vars(pairs: &[(&str, &str)]) -> Vars {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect()
}

#[test]
fn fills_variables_with_readable_names_and_leaves_other_names_alone() {
    assert_eq!(
        interpolate_text(
            "{{Site address}}/login",
            &vars(&[("Site address", "https://x.test")])
        ),
        "https://x.test/login"
    );
    assert_eq!(
        interpolate_text("{{constructor}} {{x}}", &vars(&[("x", "1")])),
        "{{constructor}} 1"
    );
}

#[test]
fn splits_text_into_words_variable_chips_and_pauses() {
    assert_eq!(
        tokenize("Hi {{Product name}}¦ welcome", true),
        [
            Token::Text("Hi ".into()),
            Token::Var("Product name".into()),
            Token::Pause,
            Token::Text(" welcome".into())
        ]
    );
    assert_eq!(tokenize("a¦b", false), [Token::Text("a¦b".into())]);
}

#[test]
fn renames_variables_everywhere_and_collects_nested_strings() {
    let renamed = rename_variable(
        &json!({ "url": "{{Site}}/x", "list": ["{{Site}}", "{{Other}}"] }),
        "Site",
        "Site address",
    );
    assert_eq!(
        renamed,
        json!({ "url": "{{Site address}}/x", "list": ["{{Site address}}", "{{Other}}"] })
    );
    let value =
        json!({ "a": "project:a.mp4", "b": ["x", { "c": "project:b.png" }], "d": 3, "e": null });
    let mut found = Vec::new();
    strings_in(&value, &|t| t.starts_with("project:"), &mut found);
    assert_eq!(found, ["project:a.mp4", "project:b.png"]);
}

#[test]
fn names_project_folders_with_slug() {
    assert_eq!(project_folder_name("My launch video"), "my-launch-video");
    assert_eq!(project_folder_name("  Q3 / Release!  "), "q3-release");
    assert_eq!(project_folder_name("v1.2_beta"), "v1-2-beta");
    assert_eq!(project_folder_name("   "), "demo");
}

#[test]
fn accepts_language_tags_and_time_zones_only() {
    assert!(is_locale("de-DE") && is_locale("ja"));
    assert!(!is_locale("xx_INVALID!!") && !is_locale("en-US, de"));
    assert!(is_time_zone("Asia/Tokyo") && is_time_zone("UTC"));
    assert!(!is_time_zone("Mars/Olympus_Mons"));
}

#[test]
fn trims_sign_in_names_and_gives_both_sides_the_same_file_name() {
    assert_eq!(
        [secret_name(" work "), secret_name("")],
        ["work", "default"]
    );
    assert_eq!(secret_file_name(" work "), secret_file_name("work"));
    assert_eq!(
        [
            secret_file_name(""),
            secret_file_name("my account"),
            secret_file_name(" Login! "),
            secret_file_name("!!!")
        ],
        ["default", "my-account", "login", "default"]
    );
}

#[test]
fn reads_a_wall_time_as_a_clock_in_that_zone_shows_it() {
    let at = |t: &str, z: &str| wall_time_in(t, z).unwrap().to_string();
    assert_eq!(at("2026-10-01T09:41", "Asia/Tokyo"), "2026-10-01T00:41:00Z");
    assert_eq!(
        at("2026-10-01T09:41:30", "America/New_York"),
        "2026-10-01T13:41:30Z"
    );
    assert_eq!(
        at("2026-01-15T09:41", "America/New_York"),
        "2026-01-15T14:41:00Z"
    );
    assert_eq!(at("2026-10-01T09:41", "UTC"), "2026-10-01T09:41:00Z");
    assert!(is_wall_time("2026-10-01T09:41") && is_wall_time("2026-10-01T09:41:05"));
    for bad in ["not a date", "", "2026-02-30T09:41", "2026-10-01T25:00"] {
        assert!(!is_wall_time(bad), "{bad}");
    }
    assert_eq!(
        wall_time_in("soon", "UTC").unwrap_err(),
        "“soon” isn't a date and time."
    );
}

#[test]
fn prunes_mixer_channels_of_deleted_nodes() {
    let wf: Workflow = serde_json::from_value(json!({
        "version": 1, "id": "x", "name": "x",
        "nodes": [{ "id": "m", "type": "audio.music", "position": { "x": 0, "y": 0 }, "params": {} }],
        "edges": [],
        "mix": { "channels": { "gone": { "fader": -3 }, "m": { "fader": -6 } } }
    }))
    .unwrap();
    assert_eq!(
        wf.normalized()
            .mix
            .unwrap()
            .channels
            .keys()
            .collect::<Vec<_>>(),
        ["m"]
    );
}

fn result(outputs: serde_json::Value) -> serde_json::Value {
    json!({ "hash": "h", "outputs": outputs, "artifacts": [], "finishedAt": 0, "durationMs": 0 })
}

#[test]
fn reads_every_saved_output_kind_and_rejects_mismatched_ones() {
    let outputs = json!({
        "media": { "type": "media", "src": "project:a.mp4", "kind": "video", "durationMs": 1200 },
        "image": { "type": "image", "src": null, "width": 640, "height": 360, "scale": 2 },
        "info": { "type": "file-info", "src": "project:probe.json" },
        "folder": { "type": "folder", "src": "project:video-code" },
        "recording": {
            "type": "recording", "src": "project:r.mp4", "log": null, "durationMs": 5000,
            "viewport": { "width": 1920, "height": 1080 }, "nodeId": "record",
            "style": [{ "nodeId": "zoom", "type": "camera.manualZoom", "params": { "atMs": 500 } }]
        }
    });
    let parsed: NodeResult = serde_json::from_value(result(outputs.clone())).unwrap();
    assert_eq!(
        serde_json::to_value(&parsed.outputs)
            .unwrap()
            .as_object()
            .unwrap()
            .keys()
            .collect::<Vec<_>>(),
        outputs.as_object().unwrap().keys().collect::<Vec<_>>()
    );
    assert!(
        serde_json::from_value::<NodeResult>(result(
            json!({ "media": { "type": "media", "src": 3 } })
        ))
        .is_err()
    );
    assert!(
        serde_json::from_value::<NodeResult>(result(json!({ "x": { "type": "unknown" } })))
            .is_err()
    );
}

#[test]
fn keeps_the_readable_node_results_when_one_entry_is_damaged() {
    let parsed = LastRun::parse(&json!({ "updatedAt": 1, "nodes": { "good": result(json!({})), "bad": result(json!({ "info": { "type": "file-info" } })) } })).unwrap();
    assert_eq!(parsed.nodes.keys().collect::<Vec<_>>(), ["good"]);
}

#[test]
fn lists_visible_node_types_and_describes_one_with_its_ports_and_settings() {
    let listed: Vec<String> = node_catalog().into_iter().map(|n| n.kind).collect();
    let visible: Vec<String> = registry().visible().map(|d| d.kind.to_string()).collect();
    assert_eq!(listed, visible);
    let click = serde_json::to_value(catalog_entry("browser.click").unwrap()).unwrap();
    assert_eq!(click["title"], "Click");
    assert_eq!(click["runs"], true);
    assert_eq!(
        click["inputs"]["session"],
        json!({ "type": "browser-session", "label": "Browser", "optional": false, "many": false })
    );
    assert_eq!(click["params"]["type"], "object");
    assert!(click["params"]["properties"]["target"].is_object());
    for kind in &listed {
        assert_eq!(
            serde_json::to_value(catalog_entry(kind).unwrap()).unwrap()["params"]["type"],
            "object",
            "{kind}"
        );
    }
    assert!(catalog_entry("nope.nothing").is_none());
}
