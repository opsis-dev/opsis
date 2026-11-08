use std::path::{Path, PathBuf};
use std::process::Command;

use crate::app::{AppFiles, DESCRIPTION, HOMEPAGE, PROGRAM, PUBLISHER, SUMMARY, VERSION};
use crate::error::PackagerError;
use crate::layout::{stage_linux, tree_size};
use crate::linking::Linked;
use crate::linux_deps::deb_depends;
use crate::target::Target;
use crate::tools::{fresh_dir, remove_dir, run, write};

const SECTION: &str = "video";

pub fn control(depends: &[String], arch: &str, installed_kib: u64) -> String {
    let long = DESCRIPTION
        .lines()
        .map(|line| format!(" {line}"))
        .collect::<Vec<_>>()
        .join("\n");
    [
        format!("Package: {PROGRAM}"),
        format!("Version: {VERSION}"),
        format!("Architecture: {arch}"),
        format!("Maintainer: {PUBLISHER}"),
        format!("Installed-Size: {installed_kib}"),
        format!("Depends: {}", depends.join(", ")),
        format!("Section: {SECTION}"),
        "Priority: optional".to_owned(),
        format!("Homepage: {HOMEPAGE}"),
        format!("Description: {SUMMARY}"),
        long,
        String::new(),
    ]
    .join("\n")
}

pub fn build_deb(
    app: &AppFiles,
    linked: &Linked,
    program: &Path,
    target: &Target,
    out: &Path,
) -> Result<PathBuf, PackagerError> {
    let work = fresh_dir(&out.join(".deb"))?;
    let root = work.join("root");
    stage_linux(app, program, &root)?;
    let installed_kib = tree_size(&root)?.div_ceil(1024);
    write(
        &root.join("DEBIAN").join("control"),
        control(&deb_depends(linked)?, target.deb_arch(), installed_kib),
    )?;
    let deb = out.join(format!("{PROGRAM}_{VERSION}_{}.deb", target.deb_arch()));
    run(Command::new("dpkg-deb")
        .args(["--root-owner-group", "-Zzstd", "--build"])
        .arg(&root)
        .arg(&deb))?;
    remove_dir(&work)?;
    Ok(deb)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_control_file_names_the_package_and_its_dependencies() {
        let text = control(&["libc6".to_owned(), "libx11-6".to_owned()], "amd64", 12);
        assert!(text.starts_with("Package: opsis\nVersion: "));
        assert!(text.contains("\nArchitecture: amd64\n"));
        assert!(text.contains("\nDepends: libc6, libx11-6\n"));
        assert!(text.contains("\nInstalled-Size: 12\n"));
        assert!(text.ends_with('\n'));
    }
}
