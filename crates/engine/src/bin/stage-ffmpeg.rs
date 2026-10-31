use std::path::PathBuf;

use opsis_engine::ffmpeg::{FFMPEG_LICENCE_FILE, stage_ffmpeg};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let target = std::env::args()
        .nth(1)
        .expect("Name the target triple to stage FFmpeg for.");
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .expect("a home folder");
    let cache = home.join(".cache").join("opsis-ffmpeg");
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("the repository folder")
        .to_path_buf();
    let binaries = root.join("crates").join("desktop").join("binaries");
    let licence = root.join("resources").join(FFMPEG_LICENCE_FILE);
    match stage_ffmpeg(&target, &cache, &binaries, &licence).await {
        Ok(staged) => println!(
            "{}",
            serde_json::to_string(&staged).expect("the staged files serialize")
        ),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
