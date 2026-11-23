use std::path::{Path, PathBuf};

use crate::error::UpdateError;
use crate::fetch::Downloaded;
use crate::installer::Installer;

pub fn install(update: &Downloaded, reopen: bool, failed: &Path) -> Result<(), UpdateError> {
    match update.installer {
        Installer::AppImage => install_appimage(&update.path, reopen),
        Installer::App => install_app(&update.path, reopen),
        Installer::Nsis => run_windows_installer(&WindowsInstall {
            file: &update.path,
            version: &update.version,
            failed,
            reopen,
        }),
        Installer::Deb | Installer::Rpm | Installer::Pacman => Err(UpdateError::NotSelfUpdating),
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WindowsInstall<'a> {
    pub file: &'a Path,
    pub version: &'a str,
    pub failed: &'a Path,
    pub reopen: bool,
}

fn sibling(path: &Path, suffix: &str) -> Result<PathBuf, UpdateError> {
    let name = path.file_name().ok_or(UpdateError::NoInstallPath)?;
    let mut hidden = std::ffi::OsString::from(".");
    hidden.push(name);
    hidden.push(suffix);
    Ok(path.with_file_name(hidden))
}

pub fn replace_file(target: &Path, new: &Path) -> Result<(), UpdateError> {
    let staged = sibling(target, ".update")?;
    std::fs::copy(new, &staged)?;
    let permissions = std::fs::metadata(target)?.permissions();
    std::fs::set_permissions(&staged, permissions)?;
    if let Err(error) = std::fs::rename(&staged, target) {
        std::fs::remove_file(&staged)?;
        return Err(error.into());
    }
    Ok(())
}

fn install_appimage(new: &Path, reopen: bool) -> Result<(), UpdateError> {
    let target = std::env::var_os("APPIMAGE")
        .map(PathBuf::from)
        .ok_or(UpdateError::NoInstallPath)?;
    replace_file(&target, new)?;
    if reopen {
        std::process::Command::new(&target).spawn()?;
    }
    Ok(())
}

pub fn app_bundle(program: &Path) -> Option<PathBuf> {
    let bundle = program.parent()?.parent()?.parent()?;
    let named = bundle
        .extension()
        .is_some_and(|extension| extension == "app");
    named.then(|| bundle.to_owned())
}

pub fn swap_folder(target: &Path, new: &Path) -> Result<(), UpdateError> {
    let old = sibling(target, ".old")?;
    if old.exists() {
        std::fs::remove_dir_all(&old)?;
    }
    std::fs::rename(target, &old)?;
    if let Err(error) = std::fs::rename(new, target) {
        std::fs::rename(&old, target)?;
        return Err(error.into());
    }
    std::fs::remove_dir_all(&old)?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn install_app(archive: &Path, reopen: bool) -> Result<(), UpdateError> {
    let program = std::env::current_exe()?;
    let bundle = app_bundle(&program).ok_or(UpdateError::NoInstallPath)?;
    let staging = sibling(&bundle, ".update")?;
    if staging.exists() {
        std::fs::remove_dir_all(&staging)?;
    }
    std::fs::create_dir_all(&staging)?;
    let unpacked = unpack_app(archive, &staging);
    let swapped = unpacked.and_then(|new| swap_folder(&bundle, &new));
    std::fs::remove_dir_all(&staging)?;
    swapped?;
    if reopen {
        std::process::Command::new("/usr/bin/open")
            .arg("-n")
            .arg(&bundle)
            .spawn()?;
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn unpack_app(archive: &Path, into: &Path) -> Result<PathBuf, UpdateError> {
    let file = std::fs::File::open(archive)?;
    tar::Archive::new(flate2::read::GzDecoder::new(file)).unpack(into)?;
    std::fs::read_dir(into)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| path.extension().is_some_and(|extension| extension == "app"))
        .ok_or(UpdateError::NoInstallPath)
}

#[cfg(not(target_os = "macos"))]
const fn install_app(_archive: &Path, _reopen: bool) -> Result<(), UpdateError> {
    Err(UpdateError::NotSelfUpdating)
}

pub fn quoted(text: &str) -> String {
    format!("'{}'", text.replace('\'', "''"))
}

pub fn windows_script(install: &WindowsInstall<'_>, program: &Path, pid: u32) -> String {
    let mut steps = vec![
        format!("Wait-Process -Id {pid} -ErrorAction SilentlyContinue"),
        format!(
            "$done = Start-Process -Wait -PassThru -FilePath {} -ArgumentList '/P'",
            quoted(&install.file.to_string_lossy())
        ),
        format!(
            "if ($done.ExitCode -ne 0) {{ @{{ version = {}; reason = \"The installer stopped with code $($done.ExitCode).\" }} | ConvertTo-Json -Compress | Set-Content -LiteralPath {} -Encoding UTF8 }}",
            quoted(install.version),
            quoted(&install.failed.to_string_lossy())
        ),
    ];
    if install.reopen {
        steps.push(format!(
            "Start-Process -FilePath {}",
            quoted(&program.to_string_lossy())
        ));
    }
    steps.join("; ")
}

#[cfg(windows)]
fn run_windows_installer(install: &WindowsInstall<'_>) -> Result<(), UpdateError> {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let program = std::env::current_exe()?;
    let script = windows_script(install, &program, std::process::id());
    let powershell = std::env::var_os("SystemRoot")
        .map(PathBuf::from)
        .ok_or(UpdateError::NoInstallPath)?
        .join("System32")
        .join("WindowsPowerShell")
        .join("v1.0")
        .join("powershell.exe");
    std::process::Command::new(powershell)
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-WindowStyle",
            "Hidden",
            "-Command",
        ])
        .arg(script)
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()?;
    Ok(())
}

#[cfg(not(windows))]
const fn run_windows_installer(_install: &WindowsInstall<'_>) -> Result<(), UpdateError> {
    Err(UpdateError::NotSelfUpdating)
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        #[test]
        fn a_quoted_text_reads_back_as_powershell_reads_it(text in ".{0,40}") {
            let quoted = quoted(&text);
            let inner = &quoted[1..quoted.len() - 1];
            prop_assert_eq!(inner.replace("''", "'"), text);
        }
    }

    #[test]
    fn a_replaced_file_keeps_its_permissions_and_takes_the_new_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("Opsis.AppImage");
        let new = dir.path().join("download");
        std::fs::write(&target, b"old").unwrap();
        std::fs::write(&new, b"new").unwrap();
        let before = std::fs::metadata(&target).unwrap().permissions();
        replace_file(&target, &new).unwrap();
        assert_eq!(std::fs::read(&target).unwrap(), b"new");
        assert_eq!(std::fs::metadata(&target).unwrap().permissions(), before);
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 2);
    }

    #[test]
    fn a_swapped_folder_holds_the_new_contents_and_leaves_nothing_behind() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("Opsis.app");
        let new = dir.path().join("staged").join("Opsis.app");
        std::fs::create_dir_all(target.join("Contents")).unwrap();
        std::fs::write(target.join("Contents").join("old"), b"").unwrap();
        std::fs::create_dir_all(new.join("Contents")).unwrap();
        std::fs::write(new.join("Contents").join("new"), b"").unwrap();
        swap_folder(&target, &new).unwrap();
        assert!(target.join("Contents").join("new").exists());
        assert!(!target.join("Contents").join("old").exists());
        assert!(!sibling(&target, ".old").unwrap().exists());
    }

    #[test]
    fn the_app_bundle_is_found_from_its_program() {
        assert_eq!(
            app_bundle(Path::new("/Applications/Opsis.app/Contents/MacOS/opsis")),
            Some(PathBuf::from("/Applications/Opsis.app"))
        );
        assert_eq!(app_bundle(Path::new("/usr/bin/opsis")), None);
    }

    #[test]
    fn the_windows_script_waits_installs_notes_a_failure_and_reopens_only_when_asked() {
        let mut install = WindowsInstall {
            file: Path::new("C:\\Users\\a b\\Opsis setup.exe"),
            version: "1.2.0",
            failed: Path::new("C:\\Users\\a b\\Opsis\\update-failed.json"),
            reopen: false,
        };
        let program = Path::new("C:\\Users\\a b\\Opsis\\opsis.exe");
        assert_eq!(
            windows_script(&install, program, 42),
            "Wait-Process -Id 42 -ErrorAction SilentlyContinue; $done = Start-Process -Wait -PassThru -FilePath 'C:\\Users\\a b\\Opsis setup.exe' -ArgumentList '/P'; if ($done.ExitCode -ne 0) { @{ version = '1.2.0'; reason = \"The installer stopped with code $($done.ExitCode).\" } | ConvertTo-Json -Compress | Set-Content -LiteralPath 'C:\\Users\\a b\\Opsis\\update-failed.json' -Encoding UTF8 }"
        );
        install.reopen = true;
        assert!(
            windows_script(&install, program, 42)
                .ends_with("; Start-Process -FilePath 'C:\\Users\\a b\\Opsis\\opsis.exe'")
        );
    }
}
