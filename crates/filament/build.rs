use std::fmt::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use sha2::{Digest, Sha256};

const FILAMENT_ENV: &str = "OPSIS_FILAMENT";
const PIN_FILE: &str = "downloads.generated.json";

#[derive(serde::Deserialize)]
struct Pins {
    version: String,
}

#[derive(serde::Deserialize)]
struct Built {
    version: String,
    sources: String,
    links: Vec<String>,
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

fn read<T: serde::de::DeserializeOwned>(file: &Path, missing: &str) -> T {
    let text = std::fs::read_to_string(file)
        .unwrap_or_else(|_| panic!("There's no {}. {missing}", file.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", file.display()))
}

fn sources_hash(dir: &Path) -> String {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "cc" || e == "h"))
        .collect();
    files.sort();
    let mut hash = Sha256::new();
    for file in files {
        let name = file
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        hash.update(name.as_bytes());
        hash.update(std::fs::read(&file).unwrap_or_else(|e| panic!("{}: {e}", file.display())));
    }
    hash.finalize()
        .iter()
        .fold(String::new(), |mut text, byte| {
            write!(text, "{byte:02x}").expect("writing to a string");
            text
        })
}

fn compile_materials(manifest: &Path, filament: &Path, out: &Path) {
    let matc = filament.join("filament").join("bin").join("matc");
    let materials = manifest.join("materials");
    for entry in std::fs::read_dir(&materials)
        .expect("the materials folder")
        .filter_map(Result::ok)
    {
        let source = entry.path();
        if source.extension().is_none_or(|e| e != "mat") {
            continue;
        }
        let name = source
            .file_stem()
            .and_then(|n| n.to_str())
            .expect("a material name");
        let target = out.join(format!("{name}.filamat"));
        let status = Command::new(&matc)
            .args([
                "-a", "vulkan", "-a", "opengl", "-a", "metal", "-p", "desktop", "-o",
            ])
            .arg(&target)
            .arg(&source)
            .status()
            .unwrap_or_else(|e| panic!("running {}: {e}", matc.display()));
        assert!(
            status.success(),
            "matc couldn't compile {}",
            source.display()
        );
        println!("cargo:rerun-if-changed={}", source.display());
    }
}

fn main() {
    let manifest = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("the crate folder"));
    let out = PathBuf::from(std::env::var_os("OUT_DIR").expect("the build folder"));
    let pins: Pins = read(
        &manifest.join(PIN_FILE),
        "Run `bun scripts/filament-pin.ts` first.",
    );
    let root = std::env::var_os(FILAMENT_ENV)
        .map_or_else(|| home().join(".cache/opsis-dev/filament"), PathBuf::from);
    let filament = root.join(&pins.version).join(platform());
    let build = filament.join("build");
    let sealed: Built = read(
        &build.join("build.json"),
        "Run `bun run filament:build` (or `bun run filament:build --docker`) first.",
    );
    assert!(
        sealed.version == pins.version,
        "The Filament build is {}, not {}. Run `bun run filament:build` again.",
        sealed.version,
        pins.version
    );
    let cpp = manifest.join("cpp");
    assert!(
        sealed.sources == sources_hash(&cpp),
        "crates/filament/cpp changed since the last Filament build. Run `bun run filament:build` again."
    );
    compile_materials(&manifest, &filament, &out);
    println!(
        "cargo:matc={}",
        filament.join("filament").join("bin").join("matc").display()
    );
    println!("cargo:rustc-link-search=native={}", build.display());
    println!("cargo:rustc-link-lib=static=opsis_filament");
    for library in &sealed.links {
        println!("cargo:rustc-link-lib={library}");
    }
    println!(
        "cargo:rerun-if-changed={}",
        build.join("build.json").display()
    );
    println!("cargo:rerun-if-changed={}", cpp.display());
    println!(
        "cargo:rerun-if-changed={}",
        manifest.join("materials").display()
    );
    println!("cargo:rerun-if-env-changed={FILAMENT_ENV}");
}
