use std::path::{Path, PathBuf};
use std::process::Command;

use crate::app::{AppFiles, DESCRIPTION, HOMEPAGE, LICENSE, PROGRAM, PUBLISHER, SUMMARY, VERSION};
use crate::error::PackagerError;
use crate::layout::stage_linux;
use crate::linking::Linked;
use crate::linux_deps::rpm_requires;
use crate::target::Target;
use crate::tools::{fresh_dir, remove_dir, run, write};

const RELEASE: &str = "1";

pub fn spec(requires: &[String], root: &Path, owned: &[String]) -> String {
    let mut lines = vec![
        "%global debug_package %{nil}".to_owned(),
        "%global __os_install_post %{nil}".to_owned(),
        "%global _build_id_links none".to_owned(),
        "%global _binary_payload w19.zstdio".to_owned(),
        format!("Name: {PROGRAM}"),
        format!("Version: {VERSION}"),
        format!("Release: {RELEASE}"),
        format!("Summary: {SUMMARY}"),
        format!("License: {LICENSE}"),
        format!("URL: {HOMEPAGE}"),
        format!("Vendor: {PUBLISHER}"),
        "AutoReqProv: no".to_owned(),
    ];
    lines.extend(requires.iter().map(|soname| format!("Requires: {soname}")));
    lines.extend([
        String::new(),
        "%description".to_owned(),
        DESCRIPTION.to_owned(),
        String::new(),
        "%install".to_owned(),
        format!("cp -a '{}/.' %{{buildroot}}/", root.display()),
        String::new(),
        "%files".to_owned(),
    ]);
    lines.extend(owned.iter().cloned());
    lines.push(String::new());
    lines.join("\n")
}

pub fn build_rpm(
    app: &AppFiles,
    linked: &Linked,
    program: &Path,
    target: &Target,
    out: &Path,
) -> Result<PathBuf, PackagerError> {
    let work = fresh_dir(&out.join(".rpm"))?;
    let root = work.join("root");
    let tree = stage_linux(app, program, &root)?;
    let spec_file = work.join(format!("{PROGRAM}.spec"));
    write(&spec_file, spec(&rpm_requires(linked)?, &root, &tree.owned))?;
    let name = format!("{PROGRAM}-{VERSION}-{RELEASE}.{}.rpm", target.arch);
    run(Command::new("rpmbuild")
        .arg("-bb")
        .arg("--define")
        .arg(format!("_topdir {}", work.join("top").display()))
        .arg("--define")
        .arg(format!("_rpmdir {}", out.display()))
        .arg("--define")
        .arg(format!("_build_name_fmt {name}"))
        .arg("--target")
        .arg(target.arch)
        .arg(&spec_file))?;
    remove_dir(&work)?;
    Ok(out.join(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_spec_requires_each_library_and_owns_each_path() {
        let text = spec(
            &["libc.so.6()(64bit)".to_owned()],
            Path::new("/work/root"),
            &["/usr/bin/opsis".to_owned(), "/usr/lib/opsis".to_owned()],
        );
        assert!(text.contains("\nRequires: libc.so.6()(64bit)\n"));
        assert!(text.contains("\ncp -a '/work/root/.' %{buildroot}/\n"));
        assert!(text.ends_with("%files\n/usr/bin/opsis\n/usr/lib/opsis\n"));
    }
}
