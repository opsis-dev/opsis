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
fn width_and_height_match_the_browser() {
    assert_case("size/width-height", &opsis_cases::size::width_height());
}

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn min_and_max_sizes_match_the_browser() {
    assert_case("size/min-max-width", &opsis_cases::size::min_max_width());
}

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn positions_match_the_browser() {
    assert_case("size/position", &opsis_cases::size::position());
}

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn insets_match_the_browser() {
    assert_case("size/inset", &opsis_cases::size::inset());
}

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn z_index_layers_match_the_browser() {
    assert_case("size/z-index", &opsis_cases::size::z_index());
}
