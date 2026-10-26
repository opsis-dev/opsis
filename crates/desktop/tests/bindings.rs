use std::path::Path;

use opsis_desktop::bindings::{BINDINGS_FILE, bindings};

#[test]
fn bindings_file_is_current() {
    let file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(BINDINGS_FILE);
    let written = std::fs::read_to_string(&file).unwrap();
    assert!(
        written == bindings().unwrap(),
        "{BINDINGS_FILE} is stale: run `bun run types:export`"
    );
}
