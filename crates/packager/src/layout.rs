use std::path::{Path, PathBuf};

use crate::app::{AGENT, AppFiles, FFMPEG, MCP, PROGRAM, RENDERER, desktop_entry};
use crate::error::PackagerError;
use crate::tools::{copy, write};

pub const PROGRAM_MODE: u32 = 0o755;
pub const FILE_MODE: u32 = 0o644;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinuxTree {
    pub root: PathBuf,
    pub owned: Vec<String>,
}

pub fn resources_dir() -> String {
    format!("/usr/lib/{PROGRAM}")
}

fn place(root: &Path, install: &str) -> PathBuf {
    root.join(install.trim_start_matches('/'))
}

fn put(
    root: &Path,
    owned: &mut Vec<String>,
    source: &Path,
    install: String,
    mode: u32,
) -> Result<(), PackagerError> {
    copy(source, &place(root, &install), mode)?;
    owned.push(install);
    Ok(())
}

pub fn stage_linux(
    app: &AppFiles,
    program: &Path,
    root: &Path,
) -> Result<LinuxTree, PackagerError> {
    let mut owned = Vec::new();
    put(
        root,
        &mut owned,
        program,
        format!("/usr/bin/{PROGRAM}"),
        PROGRAM_MODE,
    )?;
    put(
        root,
        &mut owned,
        &app.ffmpeg,
        format!("/usr/bin/{FFMPEG}"),
        PROGRAM_MODE,
    )?;
    put(
        root,
        &mut owned,
        &app.renderer,
        format!("/usr/bin/{RENDERER}"),
        PROGRAM_MODE,
    )?;
    put(
        root,
        &mut owned,
        &app.mcp,
        format!("/usr/bin/{MCP}"),
        PROGRAM_MODE,
    )?;
    put(
        root,
        &mut owned,
        &app.agent,
        format!("/usr/bin/{AGENT}"),
        PROGRAM_MODE,
    )?;
    for placed in app.resource_files()? {
        copy(
            &placed.source,
            &place(
                root,
                &format!("{}/{}", resources_dir(), placed.target.to_string_lossy()),
            ),
            FILE_MODE,
        )?;
    }
    owned.push(resources_dir());
    for (size, icon) in app.icons() {
        put(
            root,
            &mut owned,
            &icon,
            format!("/usr/share/icons/hicolor/{size}x{size}/apps/{PROGRAM}.png"),
            FILE_MODE,
        )?;
    }
    let desktop = format!("/usr/share/applications/{PROGRAM}.desktop");
    write(&place(root, &desktop), desktop_entry())?;
    crate::tools::set_mode(&place(root, &desktop), FILE_MODE)?;
    owned.push(desktop);
    Ok(LinuxTree {
        root: root.to_owned(),
        owned,
    })
}

pub fn tree_size(path: &Path) -> Result<u64, PackagerError> {
    let meta = std::fs::symlink_metadata(path).map_err(crate::error::io_error(path))?;
    if !meta.is_dir() {
        return Ok(meta.len());
    }
    let mut total = 0;
    for entry in std::fs::read_dir(path).map_err(crate::error::io_error(path))? {
        let entry = entry.map_err(crate::error::io_error(path))?;
        total += tree_size(&entry.path())?;
    }
    Ok(total)
}
