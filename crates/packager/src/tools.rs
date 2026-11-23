use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{PackagerError, io_error};

fn tool_name(command: &Command) -> String {
    command.get_program().to_string_lossy().into_owned()
}

pub fn output(command: &mut Command) -> Result<String, PackagerError> {
    let tool = tool_name(command);
    let done = command
        .output()
        .map_err(|source| PackagerError::ToolMissing {
            tool: tool.clone(),
            source,
        })?;
    if !done.status.success() {
        let mut text = String::from_utf8_lossy(&done.stdout).into_owned();
        text.push_str(&String::from_utf8_lossy(&done.stderr));
        return Err(PackagerError::ToolFailed {
            tool,
            status: done.status.to_string(),
            output: text.trim().to_owned(),
        });
    }
    Ok(String::from_utf8_lossy(&done.stdout).into_owned())
}

pub fn run(command: &mut Command) -> Result<(), PackagerError> {
    output(command).map(|_| ())
}

pub fn fresh_dir(path: &Path) -> Result<PathBuf, PackagerError> {
    if path.exists() {
        std::fs::remove_dir_all(path).map_err(io_error(path))?;
    }
    std::fs::create_dir_all(path).map_err(io_error(path))?;
    Ok(path.to_owned())
}

pub fn remove_dir(path: &Path) -> Result<(), PackagerError> {
    std::fs::remove_dir_all(path).map_err(io_error(path))
}

pub fn write(path: &Path, contents: impl AsRef<[u8]>) -> Result<(), PackagerError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(io_error(parent))?;
    }
    std::fs::write(path, contents).map_err(io_error(path))
}

#[cfg(unix)]
pub fn set_mode(path: &Path, mode: u32) -> Result<(), PackagerError> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).map_err(io_error(path))
}

#[cfg(not(unix))]
pub const fn set_mode(_path: &Path, _mode: u32) -> Result<(), PackagerError> {
    Ok(())
}

pub fn copy(from: &Path, to: &Path, mode: u32) -> Result<(), PackagerError> {
    if let Some(parent) = to.parent() {
        std::fs::create_dir_all(parent).map_err(io_error(parent))?;
    }
    std::fs::copy(from, to).map_err(io_error(from))?;
    set_mode(to, mode)
}

#[cfg(unix)]
pub fn symlink(target: &str, link: &Path) -> Result<(), PackagerError> {
    std::os::unix::fs::symlink(target, link).map_err(io_error(link))
}

#[cfg(not(unix))]
pub fn symlink(target: &str, link: &Path) -> Result<(), PackagerError> {
    Err(PackagerError::Usage(format!(
        "links ({} → {target}) are made only on Unix",
        link.display()
    )))
}
