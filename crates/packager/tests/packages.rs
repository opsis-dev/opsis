use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use opsis_packager::app::desktop_entry;
use opsis_packager::{AppFiles, Format, package};

const RESOURCES: [(&str, &str); 7] = [
    ("film-bundle/index.html", "<html></html>"),
    ("film-bundle/assets/index.js", "export {}"),
    ("terminal-bundle/index.html", "<html></html>"),
    ("resources/LICENSE.ffmpeg.txt", "FFmpeg licence"),
    (
        "resources/engine-licences/LICENSES.chromium.html",
        "credits",
    ),
    ("THIRD_PARTY_NOTICES.md", "# Notices"),
    ("QT_NOTICES.generated.md", "# Qt notices"),
];
const ICON_SIZES: [u32; 8] = [16, 24, 32, 48, 64, 128, 256, 512];

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn triple() -> String {
    format!("{}-unknown-linux-gnu", std::env::consts::ARCH)
}

fn write(path: &Path, text: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

fn fixture(dir: &Path) -> AppFiles {
    let root = dir.join("root");
    for (path, text) in RESOURCES {
        write(&root.join(path), text);
    }
    std::fs::copy(repo().join("LICENSE"), root.join("LICENSE")).unwrap();
    let icons = root.join("brand").join("icons");
    std::fs::create_dir_all(&icons).unwrap();
    for size in ICON_SIZES {
        let name = format!("{size}.png");
        std::fs::copy(
            repo().join("brand").join("icons").join(&name),
            icons.join(&name),
        )
        .unwrap();
    }
    let bin = dir.join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    std::fs::copy("/usr/bin/true", bin.join("opsis")).unwrap();
    let ffmpeg = bin.join(format!("opsis-ffmpeg-{}", triple()));
    std::fs::copy("/usr/bin/true", &ffmpeg).unwrap();
    AppFiles {
        program: bin.join("opsis"),
        ffmpeg,
        target_triple: triple(),
        root,
    }
}

fn expected_files() -> BTreeSet<String> {
    let mut files: BTreeSet<String> = [
        "usr/bin/opsis",
        "usr/bin/opsis-ffmpeg",
        "usr/share/applications/opsis.desktop",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    files.extend(
        RESOURCES
            .iter()
            .map(|(path, _)| format!("usr/lib/opsis/{path}")),
    );
    files.insert("usr/lib/opsis/LICENSE".to_owned());
    files.extend(
        ICON_SIZES
            .iter()
            .map(|size| format!("usr/share/icons/hicolor/{size}x{size}/apps/opsis.png")),
    );
    files
}

fn output(command: &mut Command) -> String {
    let out = command.output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

fn listed_files(listing: &str) -> BTreeSet<String> {
    let entries: BTreeSet<String> = listing
        .lines()
        .map(|line| {
            line.trim_start_matches("./")
                .trim_end_matches('/')
                .to_owned()
        })
        .filter(|line| !line.is_empty())
        .collect();
    entries
        .iter()
        .filter(|entry| {
            !entries
                .iter()
                .any(|other| other.starts_with(&format!("{entry}/")))
        })
        .cloned()
        .collect()
}

fn one_ending(paths: &[PathBuf], suffix: &str) -> PathBuf {
    paths
        .iter()
        .find(|path| path.to_string_lossy().ends_with(suffix))
        .unwrap_or_else(|| panic!("no {suffix} in {paths:?}"))
        .clone()
}

#[test]
fn the_rpm_installs_the_app_and_requires_the_libraries_it_loads() {
    let dir = tempfile::tempdir().unwrap();
    let app = fixture(dir.path());
    let made = package(&app, &[Format::Rpm], &dir.path().join("out")).unwrap();
    let rpm = rpm::Package::open(made.first().unwrap()).unwrap();
    let files: BTreeSet<String> = rpm
        .metadata
        .get_file_paths()
        .unwrap()
        .iter()
        .map(|path| path.to_string_lossy().trim_start_matches('/').to_owned())
        .collect();
    assert_eq!(files, expected_files());
    for entry in rpm.metadata.get_file_entries().unwrap() {
        let program = entry.path().starts_with("/usr/bin");
        assert_eq!(
            entry.mode().permissions(),
            if program { 0o755 } else { 0o644 },
            "{entry:?}"
        );
    }
    let requires: Vec<String> = rpm
        .metadata
        .get_requires()
        .unwrap()
        .into_iter()
        .map(|dependency| dependency.name)
        .collect();
    assert!(
        requires.iter().any(|name| name == "libc.so.6()(64bit)"),
        "{requires:?}"
    );
    assert_eq!(rpm.metadata.get_name().unwrap(), "opsis");
    assert_eq!(rpm.metadata.get_license().unwrap(), "GPL-3.0-or-later");
}

#[test]
fn the_deb_rpm_and_pacman_packages_install_the_same_files() {
    let dir = tempfile::tempdir().unwrap();
    let app = fixture(dir.path());
    let made = package(
        &app,
        &[Format::Deb, Format::Rpm, Format::Pacman],
        &dir.path().join("out"),
    )
    .unwrap();
    let deb = one_ending(&made, ".deb");
    let deb_tree = output(Command::new("sh").arg("-c").arg(format!(
        "dpkg-deb --fsys-tarfile '{}' | tar -t",
        deb.display()
    )));
    assert_eq!(listed_files(&deb_tree), expected_files());
    let desktop = output(Command::new("sh").arg("-c").arg(format!(
        "dpkg-deb --fsys-tarfile '{}' | tar -xO usr/share/applications/opsis.desktop",
        deb.display()
    )));
    assert_eq!(desktop, desktop_entry());
    let depends = output(Command::new("dpkg-deb").arg("-f").arg(&deb).arg("Depends"));
    assert!(depends.contains("libc6"), "{depends}");
    let pacman = one_ending(&made, ".tar.gz");
    let pacman_tree = output(Command::new("tar").arg("-tzf").arg(&pacman));
    assert_eq!(listed_files(&pacman_tree), expected_files());
    let rpm = rpm::Package::open(one_ending(&made, ".rpm")).unwrap();
    let rpm_files: BTreeSet<String> = rpm
        .metadata
        .get_file_paths()
        .unwrap()
        .iter()
        .map(|path| path.to_string_lossy().trim_start_matches('/').to_owned())
        .collect();
    assert_eq!(rpm_files, expected_files());
}

#[test]
fn a_format_for_another_system_is_refused_before_anything_is_built() {
    let dir = tempfile::tempdir().unwrap();
    let app = fixture(dir.path());
    let out = dir.path().join("out");
    assert!(package(&app, &[Format::Deb, Format::Nsis], &out).is_err());
    assert!(!out.exists());
}

#[test]
#[ignore = "downloads linuxdeploy"]
fn the_appimage_holds_the_same_files() {
    let dir = tempfile::tempdir().unwrap();
    let app = fixture(dir.path());
    let made = package(&app, &[Format::AppImage], &dir.path().join("out")).unwrap();
    let image = one_ending(&made, ".AppImage");
    let extract = dir.path().join("extract");
    std::fs::create_dir_all(&extract).unwrap();
    output(
        Command::new(&image)
            .arg("--appimage-extract")
            .current_dir(&extract),
    );
    let tree = output(
        Command::new("sh")
            .arg("-c")
            .arg("cd squashfs-root && find usr -type f")
            .current_dir(&extract),
    );
    let files: BTreeSet<String> = tree.lines().map(str::to_owned).collect();
    assert!(expected_files().is_subset(&files), "{files:?}");
}
