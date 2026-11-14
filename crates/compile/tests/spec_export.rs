use std::path::Path;

use opsis_compile::ts_export::{SPEC_TYPES_FILE, spec_types};

fn written(file: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(file),
    )
    .unwrap_or_default()
}

#[test]
fn spec_types_file_is_current() {
    assert!(
        written(SPEC_TYPES_FILE) == spec_types(),
        "{SPEC_TYPES_FILE} is stale: run `bun run types:export`"
    );
}
