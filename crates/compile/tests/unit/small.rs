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
use proptest::prelude::*;
use serde_json::json;

#[test]
fn joins_a_base_and_an_encoded_path_with_or_without_the_slash() {
    assert_eq!(
        join_url("http://127.0.0.1:5/project/", "assets/my clip#1.mp4"),
        "http://127.0.0.1:5/project/assets/my%20clip%231.mp4"
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
        wall_time_in("soon", "UTC").unwrap_err().to_string(),
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
    let mut result =
        json!({ "hash": "h", "outputs": null, "artifacts": [], "finishedAt": 0, "durationMs": 0 });
    result["outputs"] = outputs;
    result
}

#[test]
fn reads_every_saved_output_kind_and_rejects_mismatched_ones() {
    let outputs = json!({
        "media": { "type": "media", "src": "project:a.mp4", "kind": "video", "durationMs": 1200 },
        "image": { "type": "image", "src": null, "width": 640, "height": 360, "scale": 2 },
        "captions": { "type": "captions", "captions": [{ "text": "Hi", "startMs": 0, "endMs": 900 }] },
        "recording": {
            "type": "recording", "log": null, "durationMs": 5000, "nodeId": "record",
            "edits": [{ "nodeId": "zoom", "type": "camera.move", "params": { "atMs": 500 } }],
            "source": { "kind": "browser", "video": "project:r.mp4", "viewport": { "width": 1920, "height": 1080 } }
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
    let parsed = LastRun::parse(&json!({ "updatedAt": 1, "nodes": { "good": result(json!({})), "bad": result(json!({ "media": { "type": "media", "src": 3 } })) } })).unwrap();
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

#[test]
fn prepared_refs_name_their_kind_settings_and_source_and_come_back_whole() {
    use opsis_compile::asset_ref::{PreparedKind, PreparedRef};
    let typeface = PreparedRef::new(
        PreparedKind::Typeface { weight: 700.0 },
        "app:fonts/bundled/inter/inter-latin-wght-normal.woff2",
    )
    .unwrap();
    assert_eq!(
        typeface.to_ref(),
        "prepared:typeface/700/app/fonts/bundled/inter/inter-latin-wght-normal.woff2"
    );
    assert_eq!(PreparedRef::parse(&typeface.to_ref()), Some(typeface));
    let model = PreparedRef::new(PreparedKind::Model, "project:assets/models/robot.glb").unwrap();
    assert_eq!(model.path(), "model/project/assets/models/robot.glb");
    assert_eq!(PreparedRef::from_path(&model.path()), Some(model));
    assert_eq!(
        PreparedRef::new(PreparedKind::Svg, "project:../secrets/a.svg"),
        None
    );
    assert_eq!(
        PreparedRef::new(PreparedKind::Svg, "https://example.com/a.svg"),
        None
    );
    assert_eq!(PreparedRef::from_path("model/app/../../etc/passwd"), None);
    assert_eq!(PreparedRef::from_path("video/project/a.mp4"), None);
}

#[test]
fn three_d_elements_name_prepared_files_and_text_picks_the_face_the_page_would() {
    use opsis_compile::elements::prepared::prepare_props;
    use opsis_types::fonts::ProjectFonts;
    use serde_json::{Map, json};
    let fonts: ProjectFonts =
        serde_json::from_value(json!({ "main": "Inter", "heading": "Playfair Display", "code": "JetBrains Mono", "families": [] })).unwrap();
    let props = |value: serde_json::Value| -> Map<String, serde_json::Value> {
        value.as_object().unwrap().clone()
    };
    let mut heading = props(json!({ "fontWeight": 700, "italic": true }));
    prepare_props("text-3d", &mut heading, &fonts);
    assert_eq!(
        heading["typeface"],
        "prepared:typeface/700/app/fonts/bundled/playfair-display/playfair-display-latin-wght-italic.woff2"
    );
    assert_eq!(heading["typefaceItalic"], true);
    let mut serif = props(
        json!({ "fontFamily": "\"Instrument Serif\", serif", "fontWeight": 400, "italic": true }),
    );
    prepare_props("text-3d", &mut serif, &fonts);
    assert_eq!(
        serif["typefaceItalic"],
        serif["typeface"].as_str().unwrap().contains("italic")
    );
    let mut code = props(
        json!({ "fontFamily": "\"JetBrains Mono\", monospace", "fontWeight": 900, "italic": false }),
    );
    prepare_props("text-3d", &mut code, &fonts);
    assert_eq!(
        code["typeface"],
        "prepared:typeface/900/app/fonts/bundled/jetbrains-mono/jetbrains-mono-latin-wght-normal.woff2"
    );
    let mut model = props(json!({ "src": "project:assets/robot.glb" }));
    prepare_props("model-3d", &mut model, &fonts);
    assert_eq!(model["src"], "prepared:model/project/assets/robot.glb");
    let mut logo = props(json!({ "src": "project:brand/logo.svg" }));
    prepare_props("logo-3d", &mut logo, &fonts);
    assert_eq!(logo["src"], "prepared:svg/project/brand/logo.svg");
    let mut photo = props(json!({ "src": "project:brand/photo.jpg" }));
    prepare_props("card-3d", &mut photo, &fonts);
    assert_eq!(photo["src"], "prepared:texture/project/brand/photo.jpg");
    let mut empty = props(json!({ "src": "" }));
    prepare_props("logo-3d", &mut empty, &fonts);
    assert_eq!(empty["src"], "");
}

fn segment() -> impl Strategy<Value = String> {
    "[A-Za-z0-9_é-]([A-Za-z0-9_é -]{0,6}[A-Za-z0-9_é-])?"
}

fn segments() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(segment(), 1..4)
}

fn rebuilt(tokens: &[Token]) -> String {
    tokens
        .iter()
        .map(|t| match t {
            Token::Text(text) => text.clone(),
            Token::Var(name) => format!("{{{{{name}}}}}"),
            Token::Pause => "¦".to_owned(),
        })
        .collect()
}

fn is_slug(text: &str) -> bool {
    !text.is_empty()
        && text.split('-').all(|part| {
            !part.is_empty()
                && part
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        })
}

proptest! {
    #[test]
    fn keys_a_project_file_the_same_however_its_path_is_written(parts in segments(), dots in 0usize..3) {
        let plain = parts.join("/");
        let messy = format!("{}{}", "./".repeat(dots), parts.join("//"));
        for written in [
            format!("project:{plain}"),
            format!("project:/{messy}"),
            parts.join("\\"),
            format!("  project:{messy}  "),
        ] {
            prop_assert_eq!(file_version_key(&written), Some(plain.clone()), "{}", written);
        }
        prop_assert_eq!(file_version_key(&format!("app:{plain}")), None);
    }

    #[test]
    fn reads_the_last_segments_extension_in_lowercase(dirs in prop::collection::vec("[a-z]{1,4}(\\.[a-z]{1,3})?", 0..3), stem in "[A-Za-z0-9]{1,6}", ext in "[A-Za-z0-9]{1,4}") {
        let folder: String = dirs.iter().flat_map(|d| [d.as_str(), "/"]).collect();
        prop_assert_eq!(extension_of(&format!("{folder}{stem}.{ext}")), ext.to_lowercase());
        prop_assert_eq!(extension_of(&format!("{folder}{stem}")), "");
        prop_assert_eq!(extension_of(&format!("{folder}.{stem}")), "");
    }

    #[test]
    fn names_project_files_once_and_finds_them_again(parts in segments()) {
        let rel = parts.join("/");
        let named = project_ref(&rel);
        prop_assert_eq!(&project_ref(&named), &named);
        prop_assert_eq!(project_rel_path(&named), Some(rel));
    }

    #[test]
    fn splits_any_text_into_tokens_that_write_it_again(text in "[a-z {}¦]{0,40}", pauses in any::<bool>()) {
        let tokens = tokenize(&text, pauses);
        prop_assert_eq!(&tokenize(&rebuilt(&tokens), pauses), &tokens);
        prop_assert!(pauses || !tokens.contains(&Token::Pause));
        if !text.contains("{{") {
            prop_assert_eq!(rebuilt(&tokens), text);
        }
    }

    #[test]
    fn renames_every_use_of_a_variable_and_no_other(parts in prop::collection::vec(prop::sample::select(vec!["{{a}}", "{{ a }}", "{{c}}", "x", " ", "¦"]), 0..12)) {
        let text = parts.concat();
        let renamed = rename_variable(&json!({ "t": text, "l": [text] }), "a", "b");
        let tokens = tokenize(renamed["t"].as_str().unwrap(), true);
        let before = tokenize(&text, true);
        prop_assert!(!tokens.contains(&Token::Var("a".into())));
        prop_assert_eq!(tokens.len(), before.len());
        let renamed_before: Vec<Token> = before
            .into_iter()
            .map(|t| if t == Token::Var("a".into()) { Token::Var("b".into()) } else { t })
            .collect();
        prop_assert_eq!(tokens, renamed_before);
        prop_assert_eq!(&renamed["l"][0], &renamed["t"]);
    }

    #[test]
    fn names_folders_and_sign_in_files_with_a_plain_slug(name in "\\PC{0,30}") {
        let folder = project_folder_name(&name);
        prop_assert!(is_slug(&folder), "{}", folder);
        prop_assert_eq!(project_folder_name(&folder), folder);
        let file = secret_file_name(&name);
        prop_assert!(is_slug(&file), "{}", file);
        prop_assert_eq!(secret_file_name(&format!("  {name}  ")), file);
    }

    #[test]
    fn reads_any_number_typed_with_a_point_or_a_comma(value in -1e9..1e9f64, comma in any::<bool>(), pad in "[ ]{0,2}") {
        let typed = format!("{value}");
        let typed = if comma { typed.replacen('.', ",", 1) } else { typed };
        prop_assert_eq!(parse_number_text(&format!("{pad}{typed}{pad}")), Some(value));
    }

    #[test]
    fn reads_the_same_colour_in_any_notation(r in 0u8..=255, g in 0u8..=255, b in 0u8..=255) {
        let hex = format!("#{r:02x}{g:02x}{b:02x}");
        let rgb = format!("rgb({r}, {g}, {b})");
        let opaque = format!("{hex}ff");
        prop_assert!(same_color(&hex, &rgb));
        prop_assert!(same_color(&hex, &hex.to_uppercase()));
        prop_assert!(same_color(&hex, &opaque));
    }

    #[test]
    fn prepared_refs_come_back_whole_and_never_leave_their_folder(
        parts in segments(),
        kind in prop_oneof![
            (100u32..=900).prop_map(|w| opsis_compile::asset_ref::PreparedKind::Typeface { weight: f64::from(w / 100 * 100) }),
            Just(opsis_compile::asset_ref::PreparedKind::Model),
            Just(opsis_compile::asset_ref::PreparedKind::Svg),
            Just(opsis_compile::asset_ref::PreparedKind::Texture),
        ],
        app in any::<bool>(),
        escape in 0usize..3,
    ) {
        use opsis_compile::asset_ref::PreparedRef;
        let rel = parts.join("/");
        let source = format!("{}{rel}", if app { "app:" } else { "project:" });
        let prepared = PreparedRef::new(kind, &source).unwrap();
        prop_assert_eq!(PreparedRef::parse(&prepared.to_ref()), Some(prepared.clone()));
        prop_assert_eq!(PreparedRef::from_path(&prepared.path()), Some(prepared));
        let outside = format!("project:{}{rel}", "../".repeat(escape + 1));
        prop_assert_eq!(PreparedRef::new(kind, &outside), None);
    }
}

mod color_formats {
    use opsis_compile::color::{ColorFormat, Hsva, format_color, parse_color, rgb_of};
    use proptest::prelude::*;

    #[test]
    fn writes_oklch_precisely_enough_to_get_a_bright_green_back() {
        let green = parse_color("#05f2ad").unwrap();
        let text = format_color(green, ColorFormat::Oklch);
        assert_eq!(text, "oklch(0.8509 0.1816 163.72)");
        assert_eq!(
            format_color(parse_color(&text).unwrap(), ColorFormat::Hex),
            "#05f2ad"
        );
    }

    proptest! {
        #[test]
        fn a_formatted_colour_parses_back_to_the_same_colour(
            h in 0.0f64..360.0,
            s in 0.0f64..=1.0,
            v in 0.0f64..=1.0,
            a in prop_oneof![Just(1.0f64), 0.0f64..1.0],
            which in 0usize..4,
        ) {
            let format = ColorFormat::ALL[which];
            let original = format_color(Hsva { h, s, v, a }, ColorFormat::Hex);
            let text = format_color(Hsva { h, s, v, a }, format);
            prop_assert_eq!(ColorFormat::of(&text), format);
            let back = parse_color(&text).expect("a colour it wrote");
            prop_assert!((back.a - a).abs() < 0.01, "{} alpha {} vs {}", text, back.a, a);
            let (want, got) = (rgb_of(&original).unwrap(), rgb_of(&text).unwrap());
            let rounded = format == ColorFormat::Hsl;
            let tolerance = if rounded { 4.0 / 255.0 } else { 2.0 / 255.0 };
            prop_assert!((want.r - got.r).abs() <= tolerance
                && (want.g - got.g).abs() <= tolerance
                && (want.b - got.b).abs() <= tolerance, "{} vs {}", original, text);
        }
    }
}
