use std::path::PathBuf;
use std::time::Duration;

use opsis_capture::browser::launch;
use opsis_engine::chrome_for_testing::full_chrome;
use opsis_engine::paths::ShellPlatform;
use opsis_engine::tools::{browser_version, tool_output};

const CREDITS_PAGE: &str = "chrome://credits";
const PRODUCT: &str = ".product";
const WAIT: Duration = Duration::from_millis(250);
const TRIES: u32 = 240;

fn fail(message: impl std::fmt::Display) -> ! {
    eprintln!("{message}");
    std::process::exit(1);
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut args = std::env::args().skip(1).map(PathBuf::from);
    let (Some(browser), Some(output)) = (args.next(), args.next()) else {
        fail("Usage: chromium-credits <recording browser> <output file>");
    };
    let version = tool_output(&browser, &["--version"])
        .await
        .ok()
        .and_then(|text| browser_version(&text))
        .unwrap_or_else(|| fail(format!("{} isn't a recording browser.", browser.display())));
    let platform = ShellPlatform::current().unwrap_or_else(|e| fail(e));
    let home = std::env::var_os("HOME").map(PathBuf::from).expect("HOME");
    let cache = home.join(".cache").join("opsis-chromium-credits");
    let chrome = full_chrome(&version, platform, &cache)
        .await
        .unwrap_or_else(|e| fail(format!("Chrome for Testing {version}: {e}")));
    let temp = tempfile::tempdir().expect("a temp folder");
    let launched = launch(&chrome, temp.path(), &[])
        .await
        .unwrap_or_else(|e| fail(format!("Chrome didn't start: {e}")));
    let page = launched
        .browser
        .new_page(CREDITS_PAGE)
        .await
        .unwrap_or_else(|e| fail(format!("{CREDITS_PAGE} didn't open: {e}")));
    let mut found = false;
    for _ in 0..TRIES {
        if page.find_element(PRODUCT).await.is_ok() {
            found = true;
            break;
        }
        tokio::time::sleep(WAIT).await;
    }
    if !found {
        fail(format!("{CREDITS_PAGE} listed no products."));
    }
    let html = page
        .content()
        .await
        .unwrap_or_else(|e| fail(format!("{CREDITS_PAGE} couldn't be read: {e}")));
    launched.close().await;
    if let Some(dir) = output.parent() {
        std::fs::create_dir_all(dir).expect("the output folder");
    }
    std::fs::write(&output, &html).expect("the credits file");
    println!(
        "Saved the licences of Chromium {version} ({} bytes) to {}.",
        html.len(),
        output.display()
    );
}
