mod support;

use opsis_scene::node::Node;
use support::{LAYOUT, check};

fn assert_case(id: &str, root: &Node) {
    let found = check(id, root, LAYOUT);
    assert!(
        found.is_empty(),
        "{id}: {} differences\n{}",
        found.len(),
        found.join("\n")
    );
}

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn translations_match_the_browser() {
    assert_case(
        "effects/translate",
        &opsis_cases::transforms::translate_case(),
    );
}

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn rotations_match_the_browser() {
    assert_case("effects/rotate", &opsis_cases::transforms::rotate_case());
}

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn scales_match_the_browser() {
    assert_case("effects/scale", &opsis_cases::transforms::scale_case());
}

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn transforms_match_the_browser() {
    assert_case(
        "effects/transform",
        &opsis_cases::transforms::transform_case(),
    );
}

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn transform_origins_match_the_browser() {
    assert_case(
        "effects/transform-origin",
        &opsis_cases::transforms::transform_origin_case(),
    );
}

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn perspective_matches_the_browser() {
    assert_case(
        "effects/perspective",
        &opsis_cases::transforms::perspective_case(),
    );
}

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn matrix3d_matches_the_browser() {
    assert_case(
        "effects/matrix3d",
        &opsis_cases::transforms::matrix3d_case(),
    );
}
