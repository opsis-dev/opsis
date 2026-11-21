use std::path::{Path, PathBuf};
use std::process::Command;

use crate::app::{
    AGENT, AppFiles, FFMPEG, HOMEPAGE, MCP, PRODUCT, PROGRAM, PUBLISHER, RENDERER, VERSION,
};
use crate::error::{PackagerError, io_error};
use crate::layout::{FILE_MODE, PROGRAM_MODE};
use crate::target::Target;
use crate::tools::{copy, fresh_dir, remove_dir, run, write};

const NSIS_SCRIPT: &str = include_str!("../windows/installer.nsi");
const WIX_SOURCE: &str = include_str!("../windows/opsis.wxs");
const ICON: &str = "opsis.ico";
const ICO_SIZES: [u32; 7] = [16, 24, 32, 48, 64, 128, 256];

pub fn ico(images: &[(u32, Vec<u8>)]) -> Vec<u8> {
    let count = u16::try_from(images.len()).unwrap_or(u16::MAX);
    let mut header = vec![0, 0, 1, 0];
    header.extend(count.to_le_bytes());
    let mut offset = 6 + 16 * u32::from(count);
    let mut entries = Vec::new();
    let mut data = Vec::new();
    for (size, png) in images {
        let side = u8::try_from(*size).unwrap_or(0);
        let length = u32::try_from(png.len()).unwrap_or(u32::MAX);
        entries.extend([side, side, 0, 0]);
        entries.extend(1u16.to_le_bytes());
        entries.extend(32u16.to_le_bytes());
        entries.extend(length.to_le_bytes());
        entries.extend(offset.to_le_bytes());
        offset += length;
        data.extend_from_slice(png);
    }
    [header, entries, data].concat()
}

fn stage(
    app: &AppFiles,
    program: &Path,
    target: &Target,
    into: &Path,
) -> Result<(), PackagerError> {
    let exe = target.exe();
    copy(program, &into.join(format!("{PROGRAM}{exe}")), PROGRAM_MODE)?;
    copy(
        &app.ffmpeg,
        &into.join(format!("{FFMPEG}{exe}")),
        PROGRAM_MODE,
    )?;
    copy(
        &app.renderer,
        &into.join(format!("{RENDERER}{exe}")),
        PROGRAM_MODE,
    )?;
    copy(&app.mcp, &into.join(format!("{MCP}{exe}")), PROGRAM_MODE)?;
    copy(
        &app.agent,
        &into.join(format!("{AGENT}{exe}")),
        PROGRAM_MODE,
    )?;
    for placed in app.resource_files()? {
        copy(&placed.source, &into.join(&placed.target), FILE_MODE)?;
    }
    let mut images = Vec::new();
    for (size, path) in app.icons() {
        if ICO_SIZES.contains(&size) {
            images.push((size, std::fs::read(&path).map_err(io_error(&path))?));
        }
    }
    write(&into.join(ICON), ico(&images))
}

fn defines(
    target: &Target,
    stage: &Path,
    license: &Path,
    out_file: &Path,
) -> Vec<(String, String)> {
    [
        ("PRODUCT", PRODUCT.to_owned()),
        ("PROGRAM", format!("{PROGRAM}{}", target.exe())),
        ("FFMPEG", format!("{FFMPEG}{}", target.exe())),
        ("RENDERER", format!("{RENDERER}{}", target.exe())),
        ("MCP", format!("{MCP}{}", target.exe())),
        ("AGENT", format!("{AGENT}{}", target.exe())),
        ("VERSION", VERSION.to_owned()),
        ("PUBLISHER", PUBLISHER.to_owned()),
        ("HOMEPAGE", HOMEPAGE.to_owned()),
        ("ICON", ICON.to_owned()),
        ("STAGE", stage.to_string_lossy().into_owned()),
        ("LICENSE_FILE", license.to_string_lossy().into_owned()),
        ("OUTFILE", out_file.to_string_lossy().into_owned()),
    ]
    .into_iter()
    .map(|(name, value)| (name.to_owned(), value))
    .collect()
}

pub fn build_nsis(
    app: &AppFiles,
    program: &Path,
    target: &Target,
    out: &Path,
) -> Result<PathBuf, PackagerError> {
    let work = fresh_dir(&out.join(".nsis"))?;
    let staged = work.join("stage");
    stage(app, program, target, &staged)?;
    let script = work.join("installer.nsi");
    write(&script, NSIS_SCRIPT)?;
    let made = out.join(format!(
        "{PRODUCT}_{VERSION}_{}-setup.exe",
        target.windows_arch()
    ));
    let mut command = Command::new("makensis");
    command.args(["-V2", "-INPUTCHARSET", "UTF8"]);
    for (name, value) in defines(target, &staged, &app.root.join("LICENSE"), &made) {
        command.arg(format!("-D{name}={value}"));
    }
    run(command.arg(&script))?;
    remove_dir(&work)?;
    Ok(made)
}

pub fn build_msi(
    app: &AppFiles,
    program: &Path,
    target: &Target,
    out: &Path,
) -> Result<PathBuf, PackagerError> {
    let work = fresh_dir(&out.join(".msi"))?;
    let staged = work.join("stage");
    stage(app, program, target, &staged)?;
    let source = work.join("opsis.wxs");
    write(&source, WIX_SOURCE)?;
    let made = out.join(format!("{PRODUCT}_{VERSION}_{}.msi", target.windows_arch()));
    let mut command = Command::new("wix");
    command
        .args(["build", "-acceptEula", "wix7", "-arch"])
        .arg(target.windows_arch());
    for (name, value) in defines(target, &staged, &app.root.join("LICENSE"), &made) {
        command.arg("-d").arg(format!("{name}={value}"));
    }
    run(command.arg("-o").arg(&made).arg(&source))?;
    remove_dir(&work)?;
    Ok(made)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read_u32(bytes: &[u8], at: usize) -> usize {
        usize::try_from(u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())).unwrap()
    }

    #[test]
    fn the_uninstaller_removes_every_file_the_installer_places() {
        let app = AppFiles {
            program: PathBuf::new(),
            ffmpeg: PathBuf::new(),
            renderer: PathBuf::new(),
            mcp: PathBuf::new(),
            agent: PathBuf::new(),
            target_triple: String::new(),
            root: PathBuf::from("/repo"),
        };
        let uninstall = NSIS_SCRIPT.split("Section \"Uninstall\"").nth(1).unwrap();
        for placed in app.resource_roots() {
            let name = placed.target.to_string_lossy().into_owned();
            assert!(
                uninstall.contains(&format!("\"$INSTDIR\\{name}\"")),
                "the uninstaller leaves {name} behind"
            );
        }
        for name in [
            "${PROGRAM}",
            "${FFMPEG}",
            "${RENDERER}",
            "${MCP}",
            "${AGENT}",
            "${ICON}",
            "uninstall.exe",
        ] {
            assert!(uninstall.contains(&format!("\"$INSTDIR\\{name}\"")));
        }
    }

    #[test]
    fn each_icon_entry_points_at_its_own_image() {
        let images = vec![(16, vec![1u8; 10]), (256, vec![2u8; 30])];
        let file = ico(&images);
        assert_eq!(&file[..6], &[0, 0, 1, 0, 2, 0]);
        for (index, (size, png)) in images.iter().enumerate() {
            let entry = 6 + 16 * index;
            assert_eq!(file[entry], u8::try_from(*size).unwrap_or(0));
            let length = read_u32(&file, entry + 8);
            let offset = read_u32(&file, entry + 12);
            assert_eq!(&file[offset..offset + length], png.as_slice());
        }
    }
}
