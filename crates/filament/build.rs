use std::path::{Path, PathBuf};
use std::process::Command;

const FILAMENT_ENV: &str = "OPSIS_FILAMENT";
const SOURCES: [&str; 3] = ["opsis_filament.cc", "opsis_model.cc", "opsis_scene.cc"];
const HEADERS: [&str; 2] = ["opsis_filament.h", "opsis_internal.h"];
const LIBRARY_DIR: &str = "opsis";
const LIBRARIES: [&str; 23] = [
    "gltfio_core",
    "uberarchive",
    "uberzlib",
    "dracodec",
    "meshoptimizer",
    "basis_transcoder",
    "stb",
    "mikktspace",
    "filament",
    "backend",
    "bluevk",
    "filabridge",
    "filaflat",
    "utils",
    "geometry",
    "ktxreader",
    "image",
    "ibl-lite",
    "shaders",
    "smol-v",
    "abseil",
    "perfetto",
    "zstd",
];
const LINUX_SYSTEM: [&str; 4] = ["X11", "pthread", "dl", "m"];
const MACOS_FRAMEWORKS: [&str; 5] = ["Cocoa", "QuartzCore", "Metal", "CoreVideo", "Foundation"];
const WINDOWS_SYSTEM: [&str; 4] = ["Shlwapi", "ws2_32", "shell32", "user32"];

fn filament_dir(manifest: &Path) -> PathBuf {
    if let Some(dir) = std::env::var_os(FILAMENT_ENV) {
        return PathBuf::from(dir);
    }
    let file = manifest.join("../../.setup/paths.json");
    println!("cargo:rerun-if-changed={}", file.display());
    let text = std::fs::read_to_string(&file)
        .unwrap_or_else(|e| panic!("{}: {e}. Run bun run setup first.", file.display()));
    let paths: serde_json::Value =
        serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", file.display()));
    paths["filament"].as_str().map_or_else(
        || {
            panic!(
                "{} names no filament. Run bun run setup again.",
                file.display()
            )
        },
        PathBuf::from,
    )
}

fn compile_materials(manifest: &Path, matc: &Path, out: &Path) {
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
        let status = Command::new(matc)
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
    let filament = filament_dir(&manifest);
    let matc = filament
        .join("bin")
        .join(format!("matc{}", std::env::consts::EXE_SUFFIX));
    compile_materials(&manifest, &matc, &out);
    let cpp = manifest.join("cpp");
    let mut build = cc::Build::new();
    build
        .cpp(true)
        .std("c++20")
        .include(&cpp)
        .include(filament.join("include"))
        .flag_if_supported("-fno-rtti")
        .flag_if_supported("-Wno-unused-parameter");
    for source in SOURCES {
        build.file(cpp.join(source));
    }
    build.compile("opsis_filament");
    let libraries = filament.join("lib").join(LIBRARY_DIR);
    println!("cargo:rustc-link-search=native={}", libraries.display());
    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let linux = os == "linux";
    if linux {
        println!("cargo:rustc-link-arg=-Wl,--start-group");
    }
    for library in LIBRARIES {
        println!("cargo:rustc-link-lib=static={library}");
    }
    if linux {
        println!("cargo:rustc-link-arg=-Wl,--end-group");
        for library in LINUX_SYSTEM {
            println!("cargo:rustc-link-lib={library}");
        }
    }
    if os == "macos" {
        for framework in MACOS_FRAMEWORKS {
            println!("cargo:rustc-link-lib=framework={framework}");
        }
    }
    if os == "windows" {
        for library in WINDOWS_SYSTEM {
            println!("cargo:rustc-link-lib={library}");
        }
    }
    println!("cargo:matc={}", matc.display());
    for file in SOURCES.iter().chain(HEADERS.iter()) {
        println!("cargo:rerun-if-changed={}", cpp.join(file).display());
    }
    println!(
        "cargo:rerun-if-changed={}",
        manifest.join("materials").display()
    );
    println!("cargo:rerun-if-env-changed={FILAMENT_ENV}");
}
