use opsis_compile::fonts::{film_fonts, font_choices};
use opsis_compile::hosted::hosted;
use opsis_types::fonts::{FontFamilyDef, ProjectFonts};
use opsis_types::timeline::TimelineSpec;
use serde_json::{Value, json};

fn element(id: &str, props: Value) -> Value {
    json!({ "id": id, "from": 0, "durationInFrames": 30, "kind": "element", "component": "title", "props": props })
}

fn spec(base: &str, fonts: Value) -> TimelineSpec {
    serde_json::from_value(json!({
        "width": 1920,
        "height": 1080,
        "fps": 30,
        "durationInFrames": 30,
        "tracks": [{
            "id": "t",
            "items": [
                element("a", json!({
                    "image": "project:assets/shot one.png",
                    "model": "prepared:model/project/assets/robot.glb",
                    "logo": "app:placeholders/logo.svg",
                    "rows": [{ "icon": "project:assets/icon.png" }],
                    "deep": { "a": { "b": { "c": { "d": "project:assets/too-deep.png" } } } },
                    "scenes": { "tracks": [], "src": "project:assets/nested.png" },
                })),
                {
                    "id": "g",
                    "from": 0,
                    "durationInFrames": 30,
                    "kind": "group",
                    "props": {
                        "wrapper": { "component": "frame", "props": { "backdrop": "project:assets/bg.jpg" } },
                        "tracks": [{ "id": "i", "items": [element("b", json!({ "src": "project:assets/inner.mp4" }))] }],
                    },
                },
            ],
        }],
        "audio": [{ "id": "m", "src": "project:assets/music.mp3", "from": 0 }],
        "assets": { "projectBase": base },
        "fonts": fonts,
        "mix": { "src": "project:.opsis/cache/mix/master.wav", "hash": "h" },
    }))
    .unwrap()
}

fn serif() -> FontFamilyDef {
    serde_json::from_value(json!({
        "id": "lora",
        "family": "Lora",
        "category": "serif",
        "source": "file",
        "faces": [{ "file": "project:assets/fonts/lora/lora.woff2", "weight": "400 700", "style": "normal", "format": "woff2" }],
    }))
    .unwrap()
}

fn as_json(spec: &TimelineSpec) -> Value {
    serde_json::to_value(spec).unwrap()
}

#[test]
fn a_hosted_spec_names_every_project_and_prepared_file_by_its_address_on_the_host() {
    let fonts = serde_json::to_value(film_fonts(
        &ProjectFonts {
            main: "Lora".into(),
            families: vec![serif()],
            ..ProjectFonts::default()
        },
        &[],
    ))
    .unwrap();
    let out = as_json(&hosted(&spec("http://127.0.0.1:9/p/", fonts)));
    let props = &out["tracks"][0]["items"][0]["props"];
    assert_eq!(props["image"], "http://127.0.0.1:9/p/assets/shot%20one.png");
    assert_eq!(
        props["model"],
        "http://127.0.0.1:9/p/.opsis/prepared/model/project/assets/robot.glb"
    );
    assert_eq!(props["logo"], "app:placeholders/logo.svg");
    assert_eq!(
        props["rows"][0]["icon"],
        "http://127.0.0.1:9/p/assets/icon.png"
    );
    assert_eq!(
        props["deep"]["a"]["b"]["c"]["d"],
        "project:assets/too-deep.png"
    );
    assert_eq!(props["scenes"]["src"], "project:assets/nested.png");
    let group = &out["tracks"][0]["items"][1]["props"];
    assert_eq!(
        group["wrapper"]["props"]["backdrop"],
        "http://127.0.0.1:9/p/assets/bg.jpg"
    );
    assert_eq!(
        group["tracks"][0]["items"][0]["props"]["src"],
        "http://127.0.0.1:9/p/assets/inner.mp4"
    );
    assert_eq!(
        out["audio"][0]["src"],
        "http://127.0.0.1:9/p/assets/music.mp3"
    );
    assert_eq!(
        out["mix"]["src"],
        "http://127.0.0.1:9/p/.opsis/cache/mix/master.wav"
    );
    assert_eq!(
        out["fonts"]["families"]["df-main"]["faces"][0]["file"],
        "http://127.0.0.1:9/p/assets/fonts/lora/lora.woff2"
    );
    assert_eq!(
        out["fonts"]["families"]["inter"]["family"], "Lora",
        "the main font stands in for Inter"
    );
    assert!(out.get("assets").is_none());
}

#[test]
fn a_spec_without_fonts_gets_the_default_project_fonts() {
    let out = hosted(&spec("http://127.0.0.1:9/p/", Value::Null));
    let fonts = out.fonts.unwrap();
    assert_eq!(fonts.families["df-main"].family, "Inter");
    assert_eq!(fonts.families["df-code"].family, "JetBrains Mono");
    assert_eq!(fonts.families["df-heading"].family, "Inter");
    assert_eq!(
        fonts.families["df-main"].faces[0].file,
        "app:fonts/bundled/inter/inter-latin-wght-normal.woff2"
    );
}

#[test]
fn a_film_preloads_its_role_fonts_every_font_its_items_name_and_the_fonts_its_elements_draw_with() {
    let tracks: Vec<opsis_types::timeline::TimelineTrack> = serde_json::from_value(json!([{
        "id": "t",
        "items": [
            element("a", json!({ "fontFamily": "\"Fraunces\", serif" })),
            element("b", json!({ "fontFamily": "Inter" })),
            {
                "id": "g",
                "from": 0,
                "durationInFrames": 10,
                "kind": "group",
                "props": {
                    "wrapper": { "component": "laptop-frame", "props": { "fontFamily": "\"Outfit\", sans-serif" } },
                    "tracks": [{ "id": "i", "items": [element("c", json!({ "fontFamily": "Manrope" }))] }],
                },
            },
            element("d", json!({})),
        ],
    }]))
    .unwrap();
    let fonts = film_fonts(&ProjectFonts::default(), &tracks);
    assert_eq!(
        fonts.preload,
        [
            "df-main",
            "df-heading",
            "df-code",
            "Inter",
            "JetBrains Mono",
            "Fraunces",
            "Geist",
            "Instrument Serif",
            "Manrope",
            "Montserrat",
            "Outfit",
            "Space Grotesk"
        ]
    );
    assert_eq!(
        fonts.families["google sans"].family, "Google Sans Flex",
        "an alias names the family it stands for"
    );
    assert!(fonts.families.keys().all(|k| *k == k.to_lowercase()));
}

#[test]
fn the_font_picker_lists_each_family_once_with_the_project_first() {
    let library: Vec<FontFamilyDef> = vec![
        serde_json::from_value(json!({ "id": "lora", "family": "Lora", "category": "serif", "source": "online", "faces": [] })).unwrap(),
        serde_json::from_value(json!({ "id": "inter", "family": "Inter", "category": "sans-serif", "source": "online", "faces": [] })).unwrap(),
        serde_json::from_value(json!({ "id": "lato", "family": "Lato", "category": "sans-serif", "source": "online", "faces": [] })).unwrap(),
    ];
    let choices = font_choices("lora", &[serif()], &library);
    assert_eq!(choices.project.len(), 1);
    assert!(choices.project[0].selected);
    assert_eq!(choices.project[0].preview, "df-preview-file-lora");
    assert_eq!(choices.project[0].stack, "\"df-preview-file-lora\", serif");
    assert_eq!(
        choices
            .library
            .iter()
            .map(|c| c.def.family.as_str())
            .collect::<Vec<_>>(),
        ["Lato"]
    );
    assert!(choices.built_in.iter().all(|c| !c.selected));
    assert_eq!(
        choices.current.map(|c| c.def.source),
        Some(opsis_types::fonts::FontSource::File)
    );
}
