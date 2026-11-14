use std::path::{Path, PathBuf};
use std::process::Command;

use crate::app::{
    AppFiles, COPYRIGHT, FFMPEG, IDENTIFIER, MACOS_MINIMUM, PRODUCT, PROGRAM, RENDERER, VERSION,
};
use crate::error::{PackagerError, io_error};
use crate::layout::{FILE_MODE, PROGRAM_MODE};
use crate::target::Target;
use crate::tools::{copy, fresh_dir, remove_dir, run, symlink, write};

const ICONSET: [(&str, u32); 9] = [
    ("icon_16x16.png", 16),
    ("icon_16x16@2x.png", 32),
    ("icon_32x32.png", 32),
    ("icon_32x32@2x.png", 64),
    ("icon_128x128.png", 128),
    ("icon_128x128@2x.png", 256),
    ("icon_256x256.png", 256),
    ("icon_256x256@2x.png", 512),
    ("icon_512x512.png", 512),
];
const CATEGORY: &str = "public.app-category.video";

fn escaped(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub fn info_plist() -> String {
    let entries = [
        ("CFBundleDevelopmentRegion", "en"),
        ("CFBundleDisplayName", PRODUCT),
        ("CFBundleExecutable", PROGRAM),
        ("CFBundleIconFile", PROGRAM),
        ("CFBundleIdentifier", IDENTIFIER),
        ("CFBundleInfoDictionaryVersion", "6.0"),
        ("CFBundleName", PRODUCT),
        ("CFBundlePackageType", "APPL"),
        ("CFBundleShortVersionString", VERSION),
        ("CFBundleVersion", VERSION),
        ("LSApplicationCategoryType", CATEGORY),
        ("LSMinimumSystemVersion", MACOS_MINIMUM),
        ("NSHumanReadableCopyright", COPYRIGHT),
    ];
    let mut lines = vec![
        r#"<?xml version="1.0" encoding="UTF-8"?>"#.to_owned(),
        r#"<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">"#.to_owned(),
        r#"<plist version="1.0">"#.to_owned(),
        "<dict>".to_owned(),
    ];
    for (key, value) in entries {
        lines.push(format!("  <key>{key}</key>"));
        lines.push(format!("  <string>{}</string>", escaped(value)));
    }
    lines.extend([
        "  <key>NSHighResolutionCapable</key>".to_owned(),
        "  <true/>".to_owned(),
        "</dict>".to_owned(),
        "</plist>".to_owned(),
        String::new(),
    ]);
    lines.join("\n")
}

fn icns(app: &AppFiles, work: &Path, into: &Path) -> Result<(), PackagerError> {
    let iconset = fresh_dir(&work.join(format!("{PROGRAM}.iconset")))?;
    let icons = app.icons();
    for (name, size) in ICONSET {
        let source = icons
            .iter()
            .find(|(have, _)| *have == size)
            .map(|(_, path)| path)
            .ok_or_else(|| PackagerError::Missing(PathBuf::from(format!("{size}.png"))))?;
        copy(source, &iconset.join(name), FILE_MODE)?;
    }
    run(Command::new("iconutil")
        .args(["-c", "icns", "-o"])
        .arg(into)
        .arg(&iconset))
}

fn codesign(path: &Path) -> Result<(), PackagerError> {
    run(Command::new("codesign")
        .args(["--force", "--sign", "-"])
        .arg(path))
}

pub fn build_app(app: &AppFiles, program: &Path, out: &Path) -> Result<PathBuf, PackagerError> {
    let work = fresh_dir(&out.join(".app"))?;
    let bundle = out.join(format!("{PRODUCT}.app"));
    if bundle.exists() {
        std::fs::remove_dir_all(&bundle).map_err(io_error(&bundle))?;
    }
    let contents = bundle.join("Contents");
    let macos = contents.join("MacOS");
    let resources = contents.join("Resources");
    copy(program, &macos.join(PROGRAM), PROGRAM_MODE)?;
    copy(&app.ffmpeg, &macos.join(FFMPEG), PROGRAM_MODE)?;
    copy(&app.renderer, &macos.join(RENDERER), PROGRAM_MODE)?;
    for placed in app.resource_files()? {
        copy(&placed.source, &resources.join(&placed.target), FILE_MODE)?;
    }
    std::fs::create_dir_all(&resources).map_err(io_error(&resources))?;
    icns(app, &work, &resources.join(format!("{PROGRAM}.icns")))?;
    write(&contents.join("Info.plist"), info_plist())?;
    write(&contents.join("PkgInfo"), "APPL????")?;
    codesign(&macos.join(FFMPEG))?;
    codesign(&macos.join(RENDERER))?;
    codesign(&bundle)?;
    remove_dir(&work)?;
    Ok(bundle)
}

pub fn update_archive(
    bundle: &Path,
    target: &Target,
    out: &Path,
) -> Result<PathBuf, PackagerError> {
    let made = out.join(format!("{PRODUCT}_{VERSION}_{}.app.tar.gz", target.arch));
    let parent = bundle
        .parent()
        .ok_or_else(|| PackagerError::Missing(bundle.to_owned()))?;
    let name = bundle
        .file_name()
        .ok_or_else(|| PackagerError::Missing(bundle.to_owned()))?;
    run(Command::new("tar")
        .arg("-czf")
        .arg(&made)
        .arg("-C")
        .arg(parent)
        .arg(name)
        .env("COPYFILE_DISABLE", "1"))?;
    Ok(made)
}

pub fn build_dmg(bundle: &Path, target: &Target, out: &Path) -> Result<PathBuf, PackagerError> {
    let work = fresh_dir(&out.join(".dmg"))?;
    let staged = work.join(format!("{PRODUCT}.app"));
    run(Command::new("ditto").arg(bundle).arg(&staged))?;
    symlink("/Applications", &work.join("Applications"))?;
    let made = out.join(format!("{PRODUCT}_{VERSION}_{}.dmg", target.arch));
    run(Command::new("hdiutil")
        .args(["create", "-volname", PRODUCT, "-srcfolder"])
        .arg(&work)
        .args(["-ov", "-format", "UDZO"])
        .arg(&made))?;
    remove_dir(&work)?;
    Ok(made)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_info_list_names_the_program_and_the_version() {
        let plist = info_plist();
        assert!(plist.contains("<key>CFBundleExecutable</key>\n  <string>opsis</string>"));
        assert!(plist.contains(&format!(
            "<key>CFBundleShortVersionString</key>\n  <string>{VERSION}</string>"
        )));
        assert!(plist.ends_with("</plist>\n"));
    }
}
