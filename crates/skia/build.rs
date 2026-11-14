use std::path::{Path, PathBuf};

const SKIA_ENV: &str = "OPSIS_SKIA";
const SOURCES: [&str; 6] = [
    "surface.cc",
    "paint.cc",
    "path.cc",
    "text.cc",
    "image.cc",
    "lottie.cc",
];

#[derive(serde::Deserialize)]
struct Consumer {
    defines: Vec<String>,
    includes: Vec<PathBuf>,
    libraries: Vec<String>,
}

fn skia_dir(manifest: &Path) -> PathBuf {
    if let Some(dir) = std::env::var_os(SKIA_ENV) {
        return PathBuf::from(dir);
    }
    let file = manifest.join("../../.setup/paths.json");
    println!("cargo:rerun-if-changed={}", file.display());
    let text = std::fs::read_to_string(&file)
        .unwrap_or_else(|e| panic!("{}: {e}. Run bun run setup first.", file.display()));
    let paths: serde_json::Value =
        serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", file.display()));
    paths["skia"].as_str().map_or_else(
        || panic!("{} names no skia. Run bun run setup again.", file.display()),
        PathBuf::from,
    )
}

fn consumer(out: &Path) -> Consumer {
    let file = out.join("consumer.json");
    let text = std::fs::read_to_string(&file).unwrap_or_else(|_| {
        panic!(
            "There's no Skia build at {}. Run bun run setup first.",
            out.display()
        )
    });
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", file.display()))
}

fn main() {
    let manifest = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("the crate folder"));
    let cpp = manifest.join("cpp");
    let skia = skia_dir(&manifest);
    let out = skia.join("out");
    let consumer = consumer(&out);
    let mut build = cxx_build::bridge("src/bridge.rs");
    for source in SOURCES {
        build.file(cpp.join(source));
    }
    build
        .std("c++20")
        .include(&cpp)
        .include(skia.join("src"))
        .includes(&consumer.includes)
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wno-attributes");
    for define in &consumer.defines {
        match define.split_once('=') {
            Some((name, value)) => build.define(name, value),
            None => build.define(define, None),
        };
    }
    build.compile("opsis_skia");
    println!("cargo:rustc-link-search=native={}", out.display());
    let group = std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("linux");
    if group {
        println!("cargo:rustc-link-arg=-Wl,--start-group");
    }
    for library in &consumer.libraries {
        println!("cargo:rustc-link-lib=static={library}");
    }
    if group {
        println!("cargo:rustc-link-arg=-Wl,--end-group");
    }
    for source in SOURCES {
        println!("cargo:rerun-if-changed={}", cpp.join(source).display());
    }
    for header in ["opsis_skia.h", "convert.h"] {
        println!("cargo:rerun-if-changed={}", cpp.join(header).display());
    }
    println!(
        "cargo:rerun-if-changed={}",
        out.join("consumer.json").display()
    );
    println!("cargo:rerun-if-env-changed={SKIA_ENV}");
}
