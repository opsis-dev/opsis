use std::path::{Path, PathBuf};
use std::process::Command;

use crate::app::{AppFiles, HOMEPAGE, LICENSE, PROGRAM, SUMMARY, VERSION};
use crate::error::{PackagerError, io_error};
use crate::layout::stage_linux;
use crate::linking::Linked;
use crate::linux_deps::pacman_depends;
use crate::target::Target;
use crate::tools::{fresh_dir, remove_dir, run, write};

const RELEASE: &str = "1";
const EXTENSION: &str = ".pkg.tar.zst";

fn shell_quoted(text: &str) -> String {
    format!("'{}'", text.replace('\'', r"'\''"))
}

pub fn pkgbuild(depends: &[String], arch: &str, root: &Path) -> String {
    let depends = depends
        .iter()
        .map(|name| shell_quoted(name))
        .collect::<Vec<_>>()
        .join(" ");
    [
        format!("pkgname={PROGRAM}"),
        format!("pkgver={VERSION}"),
        format!("pkgrel={RELEASE}"),
        format!("pkgdesc={}", shell_quoted(SUMMARY)),
        format!("arch=({})", shell_quoted(arch)),
        format!("url={}", shell_quoted(HOMEPAGE)),
        format!("license=({})", shell_quoted(LICENSE)),
        format!("depends=({depends})"),
        "options=('!strip' '!debug' '!lto' '!emptydirs')".to_owned(),
        "package() {".to_owned(),
        format!(
            "  cp -a {}/. \"$pkgdir/\"",
            shell_quoted(&root.to_string_lossy())
        ),
        "}".to_owned(),
        String::new(),
    ]
    .join("\n")
}

pub fn build_pacman(
    app: &AppFiles,
    linked: &Linked,
    program: &Path,
    target: &Target,
    out: &Path,
) -> Result<PathBuf, PackagerError> {
    let work = fresh_dir(&out.join(".pacman"))?;
    let root = work.join("root");
    stage_linux(app, program, &root)?;
    let build = work.join("build");
    write(
        &build.join("PKGBUILD"),
        pkgbuild(&pacman_depends(linked)?, target.arch, &root),
    )?;
    let name = format!("{PROGRAM}-{VERSION}-{RELEASE}-{}{EXTENSION}", target.arch);
    let made = out.join(&name);
    if made.exists() {
        std::fs::remove_file(&made).map_err(io_error(&made))?;
    }
    run(Command::new("makepkg")
        .args(["--nodeps", "--noconfirm", "--noprogressbar"])
        .current_dir(&build)
        .env("PKGDEST", out)
        .env("BUILDDIR", work.join("makepkg"))
        .env("SRCDEST", work.join("sources"))
        .env("PKGEXT", EXTENSION)
        .env("CARCH", target.arch))?;
    remove_dir(&work)?;
    Ok(made)
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        #[test]
        fn a_quoted_word_reads_back_through_the_shell_rules(text in ".{0,30}") {
            let quoted = shell_quoted(&text);
            let inner = &quoted[1..quoted.len() - 1];
            prop_assert_eq!(inner.replace(r"'\''", "'"), text);
        }
    }

    #[test]
    fn the_pkgbuild_copies_the_staged_tree_into_the_package() {
        let text = pkgbuild(&["glibc".to_owned()], "x86_64", Path::new("/work/root"));
        assert!(text.contains("\narch=('x86_64')\n"));
        assert!(text.contains("\ndepends=('glibc')\n"));
        assert!(text.contains("\n  cp -a '/work/root'/. \"$pkgdir/\"\n"));
    }
}
