use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

const ZIG_MAJOR_MINOR: &str = "0.17.";
const WRAPPER: &str = "opsis_vtwrap";

fn main() {
    let manifest =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("Cargo sets CARGO_MANIFEST_DIR"));
    let out = PathBuf::from(env::var("OUT_DIR").expect("Cargo sets OUT_DIR"));
    let project = manifest.join("zig");
    let zig = env::var("ZIG").unwrap_or_else(|_| "zig".to_owned());

    check_zig(&zig);
    let prefix = out.join("zig-out");
    let status = Command::new(&zig)
        .current_dir(&project)
        .arg("build")
        .arg(format!("-Doptimize={}", optimize()))
        .arg(format!("-Dtarget={}", zig_target()))
        .arg("--prefix")
        .arg(&prefix)
        .arg("--cache-dir")
        .arg(out.join("zig-cache"))
        .status()
        .unwrap_or_else(|error| panic!("Couldn't start {zig}: {error}"));
    assert!(
        status.success(),
        "zig build failed in {}",
        project.display()
    );

    let lib = prefix.join("lib");
    println!("cargo:rustc-link-search=native={}", lib.display());
    for name in static_libraries(&lib) {
        println!("cargo:rustc-link-lib=static={name}");
    }
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo:rustc-link-lib=ntdll");
        println!("cargo:rustc-link-lib=kernel32");
    }
    for path in ["build.zig", "build.zig.zon", "src"] {
        println!("cargo:rerun-if-changed={}", project.join(path).display());
    }
    println!("cargo:rerun-if-env-changed=ZIG");
}

fn check_zig(zig: &str) {
    let output = Command::new(zig)
        .arg("version")
        .output()
        .unwrap_or_else(|error| {
            panic!("Zig 0.17 is needed to build opsis-vt, and {zig} couldn't start: {error}")
        });
    let version = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    assert!(
        version.starts_with(ZIG_MAJOR_MINOR),
        "Zig 0.17 is needed to build opsis-vt, found {version}"
    );
}

fn optimize() -> &'static str {
    if env::var("PROFILE").as_deref() == Ok("release") {
        "ReleaseFast"
    } else {
        "Debug"
    }
}

fn zig_target() -> String {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("Cargo sets CARGO_CFG_TARGET_ARCH");
    let os = env::var("CARGO_CFG_TARGET_OS").expect("Cargo sets CARGO_CFG_TARGET_OS");
    let abi = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    let system = match (os.as_str(), abi.as_str()) {
        ("linux", "gnu") => "linux-gnu",
        ("linux", "musl") => "linux-musl",
        ("macos", _) => "macos",
        ("windows", "msvc") => "windows-msvc",
        ("windows", "gnu") => "windows-gnu",
        _ => panic!("opsis-vt doesn't build for {os} with {abi}"),
    };
    match arch.as_str() {
        "x86_64" | "aarch64" => format!("{arch}-{system}"),
        _ => panic!("opsis-vt doesn't build for {arch}"),
    }
}

fn static_libraries(lib: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(lib)
        .unwrap_or_else(|error| panic!("Couldn't read {}: {error}", lib.display()))
        .filter_map(Result::ok)
        .filter_map(|entry| library_name(&entry.file_name().to_string_lossy()))
        .collect();
    names.sort_by_key(|name| (name != WRAPPER, name.clone()));
    names
}

fn library_name(file: &str) -> Option<String> {
    if let Some(stem) = file.strip_suffix(".a") {
        return Some(stem.strip_prefix("lib").unwrap_or(stem).to_owned());
    }
    file.strip_suffix(".lib").map(str::to_owned)
}
