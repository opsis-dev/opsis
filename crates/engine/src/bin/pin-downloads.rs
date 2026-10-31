use std::path::PathBuf;

use opsis_engine::downloads::downloads;
use opsis_engine::pin::pin_downloads;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let version = std::env::args()
        .nth(1)
        .unwrap_or_else(|| downloads().tested_version.clone());
    let home = std::env::var_os("HOME").map(PathBuf::from).expect("HOME");
    let cache = home.join(".cache").join("opsis-engine-downloads");
    std::fs::create_dir_all(&cache).expect("the download cache");
    let downloads = match pin_downloads(&version, &cache).await {
        Ok(downloads) => downloads,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };
    let out = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("downloads.generated.json");
    let json = serde_json::to_string_pretty(&downloads).expect("the downloads serialize");
    std::fs::write(&out, format!("{json}\n")).expect("the downloads file");
    println!("wrote {}", out.display());
}
