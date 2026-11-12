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
fn borders_match_the_browser() {
    assert_case("box/border", &opsis_cases::boxes::border());
}

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn padding_matches_the_browser() {
    assert_case("box/padding", &opsis_cases::boxes::padding());
}

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn overflow_matches_the_browser() {
    assert_case("box/overflow", &opsis_cases::boxes::overflow());
}

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn object_fit_boxes_match_the_browser() {
    assert_case("box/object-fit", &opsis_cases::boxes::object_fit());
}
