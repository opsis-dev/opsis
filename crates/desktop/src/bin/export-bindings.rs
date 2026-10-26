use std::path::PathBuf;

use opsis_desktop::bindings::{BINDINGS_FILE, bindings};

fn main() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(BINDINGS_FILE);
    let text = bindings().expect("the bindings export");
    std::fs::write(&path, text).expect("written");
    println!("wrote {}", path.display());
}
