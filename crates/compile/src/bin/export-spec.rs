use std::path::PathBuf;

use opsis_compile::elements::default_props::{DEFAULT_PROPS_FILE, default_props};
use opsis_compile::elements::length_cases::{LENGTH_CASES_FILE, length_cases};
use opsis_compile::ts_export::{SPEC_TYPES_FILE, spec_types};

fn write(file: &str, text: String) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(file);
    std::fs::create_dir_all(path.parent().expect("parent")).expect("folder");
    std::fs::write(&path, text).expect("written");
    println!("wrote {}", path.display());
}

fn main() {
    write(SPEC_TYPES_FILE, spec_types());
    write(LENGTH_CASES_FILE, length_cases());
    write(DEFAULT_PROPS_FILE, default_props());
}
