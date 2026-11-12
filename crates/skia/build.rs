use std::path::{Path, PathBuf};

const SKIA_COMMIT: &str = "fef60e46c522b87e5802ddf0a215a763dd5e0483";
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
    commit: String,
    defines: Vec<String>,
    includes: Vec<PathBuf>,
    libraries: Vec<String>,
}

fn home() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .expect("a home folder")
}

fn platform() -> String {
    let os = match std::env::var("CARGO_CFG_TARGET_OS").as_deref() {
        Ok("macos") => "darwin".to_owned(),
        Ok("windows") => "win32".to_owned(),
        Ok(other) => other.to_owned(),
        Err(_) => "linux".to_owned(),
    };
    let arch = match std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() {
        Ok("x86_64") | Err(_) => "x64".to_owned(),
        Ok("aarch64") => "arm64".to_owned(),
        Ok(other) => other.to_owned(),
    };
    format!("{os}-{arch}")
}

fn skia_dir() -> PathBuf {
    std::env::var_os(SKIA_ENV).map_or_else(
        || home().join(".cache/opsis-dev/skia").join(SKIA_COMMIT),
        PathBuf::from,
    )
}

fn consumer(out: &Path) -> Consumer {
    let file = out.join("consumer.json");
    let text = std::fs::read_to_string(&file).unwrap_or_else(|_| {
        panic!(
            "There's no Skia build at {}. Run `bun run skia:build` (or `bun run skia:build --docker`) first.",
            out.display()
        )
    });
    let consumer: Consumer =
        serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", file.display()));
    assert!(
        consumer.commit == SKIA_COMMIT,
        "The Skia build at {} is {}, not {SKIA_COMMIT}. Run `bun run skia:build` again.",
        out.display(),
        consumer.commit
    );
    consumer
}

fn main() {
    let manifest = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("the crate folder"));
    let cpp = manifest.join("cpp");
    let skia = skia_dir();
    let out = skia.join("out").join(platform());
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
