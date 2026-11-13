use std::fmt::Write;
use std::path::PathBuf;
use std::process::Command;

const MATC_ENV: &str = "DEP_OPSIS_FILAMENT_MATC";

fn main() {
    let manifest = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("the crate folder"));
    let out = PathBuf::from(std::env::var_os("OUT_DIR").expect("the build folder"));
    let matc = PathBuf::from(
        std::env::var_os(MATC_ENV).expect("opsis-filament names the material compiler"),
    );
    let materials = manifest.join("materials");
    let mut sources: Vec<PathBuf> = std::fs::read_dir(&materials)
        .expect("the materials folder")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "mat"))
        .collect();
    sources.sort();
    let mut table = String::from("pub const MATERIALS: &[(&str, &[u8])] = &[\n");
    for source in &sources {
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
            .arg(source)
            .status()
            .unwrap_or_else(|e| panic!("running {}: {e}", matc.display()));
        assert!(
            status.success(),
            "matc couldn't compile {}",
            source.display()
        );
        writeln!(
            table,
            "    ({name:?}, include_bytes!({:?})),",
            target.display().to_string()
        )
        .expect("writing to a string");
        println!("cargo:rerun-if-changed={}", source.display());
    }
    table.push_str("];\n");
    std::fs::write(out.join("materials.rs"), table).expect("the material table");
    println!("cargo:rerun-if-changed={}", materials.display());
    println!("cargo:rerun-if-env-changed={MATC_ENV}");
}
