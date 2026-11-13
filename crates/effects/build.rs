use std::fmt::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

const MATC_ENV: &str = "DEP_OPSIS_FILAMENT_MATC";
const INCLUDE: &str = "#include \"";

fn expanded(source: &Path, includes: &Path) -> String {
    let text =
        std::fs::read_to_string(source).unwrap_or_else(|e| panic!("{}: {e}", source.display()));
    text.lines()
        .map(|line| {
            line.trim()
                .strip_prefix(INCLUDE)
                .and_then(|rest| rest.strip_suffix('"'))
                .map_or_else(
                    || line.to_owned(),
                    |name| {
                        let file = includes.join(name);
                        println!("cargo:rerun-if-changed={}", file.display());
                        std::fs::read_to_string(&file)
                            .unwrap_or_else(|e| panic!("{}: {e}", file.display()))
                    },
                )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

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
        let flat = out.join(format!("{name}.mat"));
        std::fs::write(&flat, expanded(source, &materials.join("include")))
            .unwrap_or_else(|e| panic!("{}: {e}", flat.display()));
        let status = Command::new(&matc)
            .args([
                "-a", "vulkan", "-a", "opengl", "-a", "metal", "-p", "desktop", "-o",
            ])
            .arg(&target)
            .arg(&flat)
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
