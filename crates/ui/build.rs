use std::env;

fn main() {
    let file = env::var("DEP_OPSIS_QT_LINK_ARGS").expect("opsis-qt publishes its link arguments");
    let args = std::fs::read_to_string(&file).expect("reading the link arguments of opsis-qt");
    for arg in args.lines().filter(|line| !line.is_empty()) {
        println!("cargo:rustc-link-arg-bins={arg}");
    }
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("linux") {
        println!("cargo:rustc-link-arg-bins=-Wl,--exclude-libs,ALL");
    }
    println!("cargo:rerun-if-changed={file}");
}
