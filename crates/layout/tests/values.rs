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
fn calc_matches_the_browser() {
    assert_case("values/calc", &opsis_cases::values::calc());
}

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn color_mix_grid_matches_the_browser() {
    assert_case("values/color-mix", &opsis_cases::values::color_mix());
}

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn oklab_grid_matches_the_browser() {
    assert_case("values/oklab", &opsis_cases::values::oklab());
}

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn oklch_grid_matches_the_browser() {
    assert_case("values/oklch", &opsis_cases::values::oklch());
}

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn container_units_match_the_browser() {
    assert_case(
        "values/container-units",
        &opsis_cases::values::container_units(),
    );
}

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn relative_units_match_the_browser() {
    assert_case(
        "values/relative-units",
        &opsis_cases::values::relative_units(),
    );
}
