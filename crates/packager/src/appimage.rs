use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::app::{AppFiles, PRODUCT, PROGRAM, VERSION, desktop_entry};
use crate::error::PackagerError;
use crate::layout::{FILE_MODE, PROGRAM_MODE, stage_linux};
use crate::target::Target;
use crate::tools::{copy, fresh_dir, output, remove_dir, run, symlink, write};

const BASE_SYSTEM: [&str; 13] = [
    "ld-linux-x86-64.so.2",
    "ld-linux-aarch64.so.1",
    "libc.so.6",
    "libm.so.6",
    "libdl.so.2",
    "libpthread.so.0",
    "librt.so.1",
    "libresolv.so.2",
    "libutil.so.1",
    "libgcc_s.so.1",
    "libstdc++.so.6",
    "linux-vdso.so.1",
    "linux-gate.so.1",
];

pub const LEFT_TO_THE_SYSTEM: [&str; 7] = [
    "libasound.so.2",
    "libfontconfig.so.1",
    "libfreetype.so.6",
    "libX11.so.6",
    "libX11-xcb.so.1",
    "libxcb.so.1",
    "libwayland-client.so.0",
];

const ICON_SIZE: u32 = 256;

#[derive(Debug, Clone)]
pub struct AppImageTools {
    pub appimagetool: PathBuf,
    pub runtime: PathBuf,
}

pub fn read_ldd(text: &str) -> BTreeMap<String, PathBuf> {
    text.lines()
        .filter_map(|line| {
            let (name, rest) = line.trim().split_once(" => ")?;
            let path = rest.split(" (").next()?.trim();
            path.starts_with('/')
                .then(|| (name.trim().to_owned(), PathBuf::from(path)))
        })
        .collect()
}

pub fn unresolved(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| line.trim().strip_suffix(" => not found"))
        .map(str::to_owned)
        .collect()
}

fn ldd(file: &Path) -> Result<BTreeMap<String, PathBuf>, PackagerError> {
    let text = output(Command::new("ldd").arg(file))?;
    let missing = unresolved(&text);
    if !missing.is_empty() {
        return Err(PackagerError::SystemLibraryMissing(missing.join(", ")));
    }
    Ok(read_ldd(&text))
}

pub fn bundled(
    needed: &BTreeMap<String, PathBuf>,
    system_closure: &BTreeSet<String>,
) -> BTreeMap<String, PathBuf> {
    needed
        .iter()
        .filter(|(name, _)| {
            !BASE_SYSTEM.contains(&name.as_str()) && !system_closure.contains(name.as_str())
        })
        .map(|(name, path)| (name.clone(), path.clone()))
        .collect()
}

fn libraries_to_bundle(program: &Path) -> Result<BTreeMap<String, PathBuf>, PackagerError> {
    let needed = ldd(program)?;
    let mut system = BTreeSet::new();
    for name in LEFT_TO_THE_SYSTEM {
        let Some(path) = needed.get(name) else {
            continue;
        };
        system.insert(name.to_owned());
        system.extend(ldd(path)?.into_keys());
    }
    Ok(bundled(&needed, &system))
}

fn set_runpath(file: &Path, runpath: &str) -> Result<(), PackagerError> {
    run(Command::new("patchelf")
        .arg("--set-rpath")
        .arg(runpath)
        .arg(file))
}

pub fn build_appimage(
    app: &AppFiles,
    program: &Path,
    target: &Target,
    tools: &AppImageTools,
    out: &Path,
) -> Result<PathBuf, PackagerError> {
    let work = fresh_dir(&out.join(".appimage"))?;
    let appdir = work.join(format!("{PRODUCT}.AppDir"));
    stage_linux(app, program, &appdir)?;
    let lib = appdir.join("usr").join("lib");
    for (name, path) in libraries_to_bundle(program)? {
        let copied = lib.join(&name);
        copy(&path, &copied, FILE_MODE)?;
        set_runpath(&copied, "$ORIGIN")?;
    }
    let installed = appdir.join("usr").join("bin").join(PROGRAM);
    set_runpath(&installed, "$ORIGIN/../lib")?;
    crate::tools::set_mode(&installed, PROGRAM_MODE)?;
    write(&appdir.join(format!("{PROGRAM}.desktop")), desktop_entry())?;
    let icon = app
        .icons()
        .into_iter()
        .find(|(size, _)| *size == ICON_SIZE)
        .map(|(_, path)| path)
        .ok_or_else(|| PackagerError::Missing(PathBuf::from(format!("{ICON_SIZE}.png"))))?;
    copy(&icon, &appdir.join(format!("{PROGRAM}.png")), FILE_MODE)?;
    symlink(&format!("usr/bin/{PROGRAM}"), &appdir.join("AppRun"))?;
    symlink(&format!("{PROGRAM}.png"), &appdir.join(".DirIcon"))?;
    let made = out.join(format!("{PRODUCT}_{VERSION}_{}.AppImage", target.arch));
    run(Command::new(&tools.appimagetool)
        .arg("--runtime-file")
        .arg(&tools.runtime)
        .arg("--no-appstream")
        .arg(&appdir)
        .arg(&made)
        .env("ARCH", target.arch)
        .env("APPIMAGE_EXTRACT_AND_RUN", "1"))?;
    remove_dir(&work)?;
    Ok(made)
}

#[cfg(test)]
mod tests {
    use super::*;

    const LDD: &str = "\tlinux-vdso.so.1 (0x00007ffd)\n\tlibdbus-1.so.3 => /lib/x86_64-linux-gnu/libdbus-1.so.3 (0x00007f01)\n\tlibxcb.so.1 => /lib/x86_64-linux-gnu/libxcb.so.1 (0x00007f02)\n\tlibXau.so.6 => /lib/x86_64-linux-gnu/libXau.so.6 (0x00007f03)\n\tlibc.so.6 => /lib/x86_64-linux-gnu/libc.so.6 (0x00007f04)\n\t/lib64/ld-linux-x86-64.so.2 (0x00007f05)\n";

    #[test]
    fn reads_each_resolved_library_from_ldd() {
        let read = read_ldd(LDD);
        assert_eq!(
            read.keys().map(String::as_str).collect::<Vec<_>>(),
            ["libXau.so.6", "libc.so.6", "libdbus-1.so.3", "libxcb.so.1"]
        );
        assert_eq!(
            read["libdbus-1.so.3"],
            Path::new("/lib/x86_64-linux-gnu/libdbus-1.so.3")
        );
    }

    #[test]
    fn names_each_library_ldd_couldnt_find() {
        let text = format!("{LDD}\tlibxkbcommon.so.0 => not found\n");
        assert_eq!(unresolved(&text), ["libxkbcommon.so.0"]);
        assert_eq!(unresolved(LDD), Vec::<String>::new());
    }

    #[test]
    fn bundles_what_neither_the_base_system_nor_a_system_library_brings() {
        let needed = read_ldd(LDD);
        let system: BTreeSet<String> = ["libxcb.so.1", "libXau.so.6"].map(str::to_owned).into();
        assert_eq!(
            bundled(&needed, &system).into_keys().collect::<Vec<_>>(),
            ["libdbus-1.so.3"]
        );
    }
}
