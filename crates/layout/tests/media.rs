mod support;

use support::{LAYOUT, check};

#[test]
#[ignore = "needs the oracle's case boxes (bun run oracle boxes)"]
fn images_match_the_browser() {
    let found = check("media/img", &opsis_cases::media::img_case(), LAYOUT);
    assert!(found.is_empty(), "{}", found.join("\n"));
}
