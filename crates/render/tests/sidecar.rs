use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use opsis_render::protocol::{COMPOSITION_ID, RenderTarget, SidecarCall, StillJob};
use opsis_render::sidecar::{Sidecar, SidecarCommand, SidecarError};
use serde_json::json;
use tokio_util::sync::CancellationToken;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn bun(args: &[&str], env: Vec<(OsString, OsString)>) -> SidecarCommand {
    SidecarCommand {
        program: "bun".into(),
        args: args.iter().map(OsString::from).collect(),
        env,
        cwd: Some(repo()),
    }
}

fn quiet() -> Arc<dyn Fn(&str) + Send + Sync> {
    Arc::new(|_| {})
}

#[tokio::test]
async fn answers_a_ping() {
    let sidecar = Sidecar::new(
        bun(&["run", "apps/sidecar/src/main.ts"], Vec::new()),
        quiet(),
    );
    sidecar.ping().await.unwrap();
    sidecar.ping().await.unwrap();
}

#[tokio::test]
async fn reports_a_render_that_fails_with_its_message_and_keeps_answering() {
    let sidecar = Sidecar::new(
        bun(&["run", "apps/sidecar/src/main.ts"], Vec::new()),
        quiet(),
    );
    let spec = serde_json::from_value(json!({ "width": 64, "height": 64, "fps": 30, "durationInFrames": 30, "tracks": [], "audio": [] })).unwrap();
    let target = RenderTarget {
        serve_url: "/nowhere/bundle".into(),
        browser_executable: "/nowhere/headless_shell".into(),
        binaries_directory: None,
        license_key: "free-license".into(),
        composition: COMPOSITION_ID.into(),
        spec,
        gl: serde_json::from_value(json!("angle")).unwrap(),
    };
    let job = StillJob {
        target,
        frame: 0.0,
        output: "/nowhere/out.png".into(),
        format: serde_json::from_value(json!("png")).unwrap(),
        scale: None,
        jpeg_quality: None,
    };
    let failed = sidecar
        .call(
            SidecarCall::RenderStill(job),
            &CancellationToken::new(),
            None,
        )
        .await;
    assert!(
        matches!(failed, Err(SidecarError::Failed(ref message)) if !message.is_empty()),
        "{failed:?}"
    );
    sidecar.ping().await.unwrap();
}

const CRASHES_ONCE: &str = r#"
const fs = require("node:fs");
const first = !fs.existsSync(process.env.MARKER);
fs.writeFileSync(process.env.MARKER, "");
require("node:readline").createInterface({ input: process.stdin }).on("line", (line) => {
  if (first) process.exit(1);
  process.stdout.write(JSON.stringify({ id: JSON.parse(line).id, result: null }) + "\n");
});
"#;

#[tokio::test]
async fn fails_the_job_in_flight_when_it_crashes_and_starts_again_for_the_next() {
    let dir = tempfile::tempdir().unwrap();
    let marker = (
        OsString::from("MARKER"),
        dir.path().join("started").into_os_string(),
    );
    let sidecar = Sidecar::new(bun(&["-e", CRASHES_ONCE], vec![marker]), quiet());
    assert!(matches!(sidecar.ping().await, Err(SidecarError::Stopped)));
    sidecar.ping().await.unwrap();
}
