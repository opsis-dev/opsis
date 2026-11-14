use std::path::PathBuf;

use opsis_compile::ts_export::{SPEC_TYPES_FILE, spec_types};

fn main() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(SPEC_TYPES_FILE);
    std::fs::create_dir_all(path.parent().expect("parent")).expect("folder");
    std::fs::write(&path, spec_types()).expect("written");
    println!("wrote {}", path.display());
}
