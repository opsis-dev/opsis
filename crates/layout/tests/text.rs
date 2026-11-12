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
fn font_families_match_the_browser() {
    assert_case("text/font-family", &opsis_cases::text::font_family());
}

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn font_sizes_match_the_browser() {
    assert_case("text/font-size", &opsis_cases::text::font_size());
}

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn letter_spacing_matches_the_browser() {
    assert_case("text/letter-spacing", &opsis_cases::text::letter_spacing());
}

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn line_heights_match_the_browser() {
    assert_case("text/line-height", &opsis_cases::text::line_height());
}

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn text_alignment_matches_the_browser() {
    assert_case("text/text-align", &opsis_cases::text::text_align());
}

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn white_space_matches_the_browser() {
    assert_case("text/white-space", &opsis_cases::text::white_space());
}

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn color_matches_the_browser() {
    assert_case("text/color", &opsis_cases::text::color());
}

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn font_weight_matches_the_browser() {
    assert_case("text/font-weight", &opsis_cases::text::font_weight());
}

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn font_variant_numeric_matches_the_browser() {
    assert_case(
        "text/font-variant-numeric",
        &opsis_cases::text::font_variant_numeric(),
    );
}

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn font_variation_settings_matches_the_browser() {
    assert_case(
        "text/font-variation-settings",
        &opsis_cases::text::font_variation_settings(),
    );
}

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn text_transform_matches_the_browser() {
    assert_case("text/text-transform", &opsis_cases::text::text_transform());
}

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn text_shadow_matches_the_browser() {
    assert_case("text/text-shadow", &opsis_cases::text::text_shadow());
}

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn text_stroke_matches_the_browser() {
    assert_case("text/text-stroke", &opsis_cases::text::text_stroke());
}

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn text_overflow_matches_the_browser() {
    assert_case("text/text-overflow", &opsis_cases::text::text_overflow());
}

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn wrapping_matches_the_browser() {
    let found = support::check_skipping(
        "text/wrapping",
        &opsis_cases::text::wrapping(),
        LAYOUT,
        &[&[0, 4]],
    );
    assert!(found.is_empty(), "{}", found.join("\n"));
}

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn spans_match_the_browser() {
    assert_case("text/spans", &opsis_cases::text::spans());
}

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn vertical_align_matches_the_browser() {
    assert_case("text/vertical-align", &opsis_cases::text::vertical_align());
}
