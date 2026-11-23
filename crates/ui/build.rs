use std::env;
use std::path::PathBuf;

const ICON: &str = "../../brand/icons/opsis.ico";
const PROGRAM: &str = "opsis";

fn main() {
    let file = env::var("DEP_OPSIS_QT_LINK_ARGS").expect("opsis-qt publishes its link arguments");
    let args = std::fs::read_to_string(&file).expect("reading the link arguments of opsis-qt");
    for arg in args.lines().filter(|line| !line.is_empty()) {
        println!("cargo:rustc-link-arg-bins={arg}");
    }
    let os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if os == "linux" {
        println!("cargo:rustc-link-arg-bins=-Wl,--exclude-libs,ALL");
    }
    if os == "windows" {
        embed_icon();
    }
    println!("cargo:rerun-if-changed={file}");
}

fn embed_icon() {
    let manifest =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("Cargo sets CARGO_MANIFEST_DIR"));
    let icon = manifest.join(ICON);
    assert!(
        icon.exists(),
        "{} is missing: run bun run brand:icons",
        icon.display()
    );
    let out = PathBuf::from(env::var("OUT_DIR").expect("Cargo sets OUT_DIR"));
    let script = out.join("opsis.rc");
    let quoted = icon.to_string_lossy().replace('\\', "/");
    std::fs::write(&script, format!("IDI_ICON1 ICON \"{quoted}\"\n"))
        .expect("writing the program's resource script");
    embed_resource::compile_for(&script, [PROGRAM], embed_resource::NONE)
        .manifest_optional()
        .expect("embedding the program's icon");
    println!("cargo:rerun-if-changed={}", icon.display());
}
