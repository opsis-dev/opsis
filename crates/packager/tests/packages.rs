use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use opsis_packager::app::desktop_entry;
use opsis_packager::appimage::AppImageTools;
use opsis_packager::{AppFiles, Format, Tools, package};
use opsis_update::installer::{Installer, MARKER_LEN, UNMARKED, read_marker};

const RESOURCES: [(&str, &str); 5] = [
    ("public/sfx/mouse-click.wav", "sound"),
    ("resources/LICENSE.ffmpeg.txt", "FFmpeg licence"),
    (
        "resources/engine-licences/LICENSES.chromium.html",
        "credits",
    ),
    ("THIRD_PARTY_NOTICES.md", "# Notices"),
    ("QT_NOTICES.generated.md", "# Qt notices"),
];
const ICON_SIZES: [u32; 8] = [16, 24, 32, 48, 64, 128, 256, 512];
const APPIMAGETOOL_ENV: &str = "OPSIS_APPIMAGETOOL";
const RUNTIME_ENV: &str = "OPSIS_APPIMAGE_RUNTIME";

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

fn unmarked_program(path: &Path) {
    let mut bytes = std::fs::read("/usr/bin/true").unwrap();
    bytes.extend_from_slice(&UNMARKED);
    std::fs::write(path, bytes).unwrap();
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
    unmarked_program(&bin.join("opsis"));
    let ffmpeg = bin.join(format!("opsis-ffmpeg-{}", triple()));
    std::fs::copy("/usr/bin/true", &ffmpeg).unwrap();
    let renderer = bin.join("opsis-renderer");
    std::fs::copy("/usr/bin/true", &renderer).unwrap();
    AppFiles {
        program: bin.join("opsis"),
        ffmpeg,
        renderer,
        target_triple: triple(),
        root,
    }
}

fn expected_files() -> BTreeSet<String> {
    let mut files: BTreeSet<String> = [
        "usr/bin/opsis",
        "usr/bin/opsis-ffmpeg",
        "usr/bin/opsis-renderer",
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

fn shell(script: &str) -> String {
    output(Command::new("sh").arg("-c").arg(script))
}

fn listed_files(listing: &str) -> BTreeSet<String> {
    let entries: BTreeSet<String> = listing
        .lines()
        .map(|line| {
            line.trim_start_matches("./")
                .trim_start_matches('/')
                .trim_end_matches('/')
                .to_owned()
        })
        .filter(|line| !line.is_empty() && !line.starts_with('.'))
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

fn installer_of(program: &Path) -> Option<Installer> {
    let bytes = std::fs::read(program).unwrap();
    bytes
        .windows(MARKER_LEN)
        .find_map(|window| read_marker(window.try_into().unwrap()))
}

fn extracted(dir: &Path, script: &str) -> PathBuf {
    let into = dir.join("extracted");
    let _ = std::fs::remove_dir_all(&into);
    std::fs::create_dir_all(&into).unwrap();
    shell(&format!("cd '{}' && {script}", into.display()));
    into.join("usr").join("bin").join("opsis")
}

#[test]
fn the_deb_installs_the_app_marked_as_a_deb() {
    let dir = tempfile::tempdir().unwrap();
    let app = fixture(dir.path());
    let made = package(
        &app,
        &[Format::Deb],
        &Tools::default(),
        &dir.path().join("out"),
    )
    .unwrap();
    let deb = one_ending(&made, ".deb");
    let tree = shell(&format!(
        "dpkg-deb --fsys-tarfile '{}' | tar -t",
        deb.display()
    ));
    assert_eq!(listed_files(&tree), expected_files());
    let desktop = shell(&format!(
        "dpkg-deb --fsys-tarfile '{}' | tar -xO ./usr/share/applications/opsis.desktop",
        deb.display()
    ));
    assert_eq!(desktop, desktop_entry());
    let depends = output(Command::new("dpkg-deb").arg("-f").arg(&deb).arg("Depends"));
    assert!(depends.contains("libc6"), "{depends}");
    let program = extracted(dir.path(), &format!("dpkg-deb -x '{}' .", deb.display()));
    assert_eq!(installer_of(&program), Some(Installer::Deb));
    assert_eq!(installer_of(&app.program), None);
}

#[test]
#[ignore = "needs rpmbuild, makepkg and bsdtar; bun run packager:test runs it"]
fn the_deb_rpm_and_pacman_packages_install_the_same_files_each_marked_as_itself() {
    let dir = tempfile::tempdir().unwrap();
    let app = fixture(dir.path());
    let made = package(
        &app,
        &[Format::Deb, Format::Rpm, Format::Pacman],
        &Tools::default(),
        &dir.path().join("out"),
    )
    .unwrap();
    let rpm = one_ending(&made, ".rpm");
    let rpm_tree = output(Command::new("rpm").arg("-qlp").arg(&rpm));
    assert_eq!(listed_files(&rpm_tree), expected_files());
    let modes = output(
        Command::new("rpm")
            .args([
                "-qp",
                "--queryformat",
                "[%{FILEMODES:perms} %{FILENAMES}\n]",
            ])
            .arg(&rpm),
    );
    for line in modes.lines().filter(|line| line.starts_with('-')) {
        let program = line.contains(" /usr/bin/");
        assert!(
            line.starts_with(if program { "-rwxr-xr-x" } else { "-rw-r--r--" }),
            "{line}"
        );
    }
    let requires = output(Command::new("rpm").args(["-qp", "--requires"]).arg(&rpm));
    assert!(requires.contains("libc.so.6()(64bit)"), "{requires}");
    let program = extracted(dir.path(), &format!("bsdtar -xf '{}'", rpm.display()));
    assert_eq!(installer_of(&program), Some(Installer::Rpm));
    let pacman = one_ending(&made, ".pkg.tar.zst");
    let pacman_tree = output(Command::new("bsdtar").arg("-tf").arg(&pacman));
    assert_eq!(listed_files(&pacman_tree), expected_files());
    let program = extracted(dir.path(), &format!("bsdtar -xf '{}'", pacman.display()));
    assert_eq!(installer_of(&program), Some(Installer::Pacman));
}

#[test]
fn a_format_for_another_system_is_refused_before_anything_is_built() {
    let dir = tempfile::tempdir().unwrap();
    let app = fixture(dir.path());
    let out = dir.path().join("out");
    assert!(package(&app, &[Format::Deb, Format::Nsis], &Tools::default(), &out).is_err());
    assert!(!out.exists());
}

#[test]
#[ignore = "needs patchelf and the pinned AppImage tools; bun run packager:test runs it"]
fn the_appimage_holds_the_same_files_marked_as_an_appimage() {
    let dir = tempfile::tempdir().unwrap();
    let app = fixture(dir.path());
    let tools = Tools {
        appimage: Some(AppImageTools {
            appimagetool: std::env::var_os(APPIMAGETOOL_ENV).unwrap().into(),
            runtime: std::env::var_os(RUNTIME_ENV).unwrap().into(),
        }),
    };
    let made = package(&app, &[Format::AppImage], &tools, &dir.path().join("out")).unwrap();
    let image = one_ending(&made, ".AppImage");
    let extract = dir.path().join("extract");
    std::fs::create_dir_all(&extract).unwrap();
    output(
        Command::new(&image)
            .arg("--appimage-extract")
            .current_dir(&extract)
            .env("APPIMAGE_EXTRACT_AND_RUN", "1"),
    );
    let root = extract.join("squashfs-root");
    let tree = output(
        Command::new("find")
            .args(["usr", "-type", "f"])
            .current_dir(&root),
    );
    let files: BTreeSet<String> = tree.lines().map(str::to_owned).collect();
    assert!(expected_files().is_subset(&files), "{files:?}");
    assert_eq!(
        installer_of(&root.join("usr").join("bin").join("opsis")),
        Some(Installer::AppImage)
    );
}
