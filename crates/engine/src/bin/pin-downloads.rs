use std::path::PathBuf;

use opsis_engine::downloads::ShellSource;
use opsis_engine::paths::ShellPlatform;
use opsis_engine::pin::{pin_downloads, shell_url};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut args = std::env::args().skip(1);
    let (Some(version), Some(remotion_url)) = (args.next(), args.next()) else {
        eprintln!(
            "Usage: pin-downloads <Remotion's tested browser version> <Remotion's mac-arm64 address>"
        );
        std::process::exit(2);
    };
    if shell_url(&version, ShellPlatform::MacArm64, ShellSource::Remotion) != remotion_url {
        eprintln!("Remotion's download addresses changed: {remotion_url}");
        std::process::exit(1);
    }
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
