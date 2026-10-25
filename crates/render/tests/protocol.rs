use std::path::PathBuf;

use opsis_render::protocol::{COMPOSITION_ID, PROTOCOL_FILE, protocol_types};

fn repo_file(rel: &str) -> String {
    std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(rel),
    )
    .unwrap_or_default()
}

#[test]
fn protocol_file_is_current() {
    assert!(
        repo_file(PROTOCOL_FILE) == protocol_types(),
        "{PROTOCOL_FILE} is stale: run `bun run types:export`"
    );
}

#[test]
fn asks_for_the_composition_the_video_bundle_registers() {
    assert_eq!(
        repo_file("packages/video/src/composition-id.ts").trim(),
        format!("export const COMPOSITION_ID = \"{COMPOSITION_ID}\";")
    );
}
