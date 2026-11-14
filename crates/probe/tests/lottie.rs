use std::io::Write;

use opsis_probe::lottie::{animation_json, uses_expressions};
use serde_json::{Value, json};

fn packed(animation: &Value, picture: &[u8]) -> Vec<u8> {
    let mut out = std::io::Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut out);
        let options = zip::write::SimpleFileOptions::default();
        zip.start_file("manifest.json", options)
            .expect("a manifest");
        zip.write_all(br#"{"animations":[{"id":"a"}]}"#)
            .expect("the manifest");
        zip.start_file("animations/a.json", options)
            .expect("an animation");
        zip.write_all(animation.to_string().as_bytes())
            .expect("the animation");
        zip.start_file("images/img_0.png", options)
            .expect("a picture");
        zip.write_all(picture).expect("the picture");
        zip.finish().expect("a finished file");
    }
    out.into_inner()
}

#[test]
fn a_packed_animation_carries_its_pictures_inside() {
    let animation = json!({"v": "5.7.4", "assets": [{"id": "image_0", "p": "img_0.png", "u": "/images/", "e": 0}]});
    let unpacked = animation_json(&packed(&animation, b"picture bytes")).expect("an animation");
    let value: Value = serde_json::from_slice(&unpacked).expect("JSON");
    let asset = &value["assets"][0];
    assert_eq!(asset["p"], "data:image/png;base64,cGljdHVyZSBieXRlcw==");
    assert_eq!(asset["e"], 1);
    assert_eq!(asset["u"], "");
}

#[test]
fn a_plain_animation_is_passed_through() {
    let bytes = br#"{"v":"5.7.4"}"#;
    assert_eq!(animation_json(bytes).as_deref(), Some(&bytes[..]));
}

#[test]
fn expressions_are_found_in_any_property() {
    let with = json!({"layers": [{"ks": {"o": {"a": 0, "k": 100, "x": "loopOut()"}}}]});
    let without = json!({"layers": [{"ks": {"o": {"a": 0, "k": 100}}}]});
    assert!(uses_expressions(with.to_string().as_bytes()));
    assert!(!uses_expressions(without.to_string().as_bytes()));
}
