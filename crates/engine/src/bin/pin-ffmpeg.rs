use std::path::PathBuf;

use opsis_engine::pin::pin_ffmpeg;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let home = std::env::var_os("HOME").map(PathBuf::from).expect("HOME");
    let cache = home.join(".cache").join("opsis-ffmpeg");
    let pins = match pin_ffmpeg(&cache).await {
        Ok(pins) => pins,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };
    let out = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ffmpeg.generated.json");
    let json = serde_json::to_string_pretty(&pins).expect("the pins serialize");
    std::fs::write(&out, format!("{json}\n")).expect("the pins file");
    println!("wrote {}", out.display());
}
