use std::path::PathBuf;

use opsis_compile::ts_export::{BINDINGS_FILE, bindings};

#[test]
fn bindings_file_is_current() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(BINDINGS_FILE);
    let written = std::fs::read_to_string(&path).unwrap_or_default();
    assert!(
        written == bindings(),
        "{BINDINGS_FILE} is stale: run `bun run types:export`"
    );
}
