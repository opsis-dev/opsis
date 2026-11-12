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
fn display_matches_the_browser() {
    assert_case("flex/display", &opsis_cases::flex::display());
}

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn flex_direction_matches_the_browser() {
    assert_case("flex/flex-direction", &opsis_cases::flex::flex_direction());
}

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn justify_content_matches_the_browser() {
    assert_case(
        "flex/justify-content",
        &opsis_cases::flex::justify_content(),
    );
}

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn gap_matches_the_browser() {
    assert_case("flex/gap", &opsis_cases::flex::gap());
}

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn flex_matches_the_browser() {
    assert_case("flex/flex", &opsis_cases::flex::flex());
}

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn flex_wrap_matches_the_browser() {
    assert_case("flex/flex-wrap", &opsis_cases::flex::flex_wrap());
}

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn grid_template_columns_match_the_browser() {
    assert_case(
        "flex/grid-template-columns",
        &opsis_cases::flex::grid_template_columns(),
    );
}

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn align_items_match_the_browser() {
    assert_case("flex/align-items", &opsis_cases::flex::align_items());
}

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn flex_shrink_matches_the_browser() {
    assert_case("flex/flex-shrink", &opsis_cases::flex::flex_shrink());
}

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn place_items_match_the_browser() {
    assert_case("flex/place-items", &opsis_cases::flex::place_items());
}
