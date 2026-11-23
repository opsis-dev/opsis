use std::path::{Path, PathBuf};
use std::process::Command;

use crate::app::{
    AGENT, AppFiles, FFMPEG, HOMEPAGE, MCP, PRODUCT, PROGRAM, PUBLISHER, RENDERER, VERSION,
};
use crate::error::PackagerError;
use crate::layout::{FILE_MODE, PROGRAM_MODE};
use crate::target::Target;
use crate::tools::{copy, fresh_dir, remove_dir, run, write};

const NSIS_SCRIPT: &str = include_str!("../windows/installer.nsi");
const ICON: &str = "opsis.ico";

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
    copy(&app.windows_icon(), &into.join(ICON), FILE_MODE)
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
