use std::path::PathBuf;

use opsis_compile::ts_export::{BINDINGS_FILE, bindings};

fn main() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(BINDINGS_FILE);
    std::fs::create_dir_all(path.parent().expect("parent")).expect("folder");
    std::fs::write(&path, bindings()).expect("written");
    println!("wrote {}", path.display());
}
