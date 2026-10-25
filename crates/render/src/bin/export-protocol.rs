use std::path::PathBuf;

use opsis_render::protocol::{PROTOCOL_FILE, protocol_types};

fn main() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(PROTOCOL_FILE);
    std::fs::create_dir_all(path.parent().expect("parent")).expect("folder");
    std::fs::write(&path, protocol_types()).expect("written");
    println!("wrote {}", path.display());
}
