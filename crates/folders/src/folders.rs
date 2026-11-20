use std::path::{Path, PathBuf};

fn env_dir(name: &str) -> Option<PathBuf> {
    std::env::var_os(name)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

fn home() -> PathBuf {
    env_dir("HOME")
        .or_else(|| env_dir("USERPROFILE"))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn config(home: &std::path::Path) -> PathBuf {
    if cfg!(target_os = "windows") {
        env_dir("APPDATA").unwrap_or_else(|| home.join("AppData").join("Roaming"))
    } else if cfg!(target_os = "macos") {
        home.join("Library").join("Application Support")
    } else {
        env_dir("XDG_CONFIG_HOME").unwrap_or_else(|| home.join(".config"))
    }
}

const NOTICES: &str = "THIRD_PARTY_NOTICES.md";

fn resources() -> PathBuf {
    let beside = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(PathBuf::from))
        .unwrap_or_default();
    resources_for(&beside)
}

fn resources_for(program_dir: &Path) -> PathBuf {
    if cfg!(target_os = "macos") {
        return program_dir.join("..").join("Resources");
    }
    let installed = program_dir.join("..").join("lib").join("opsis");
    if !program_dir.join(NOTICES).is_file() && installed.join(NOTICES).is_file() {
        installed
    } else {
        program_dir.to_owned()
    }
}

#[derive(Debug, Clone)]
pub struct HostFolders {
    pub config: PathBuf,
    pub resources: PathBuf,
    pub home: PathBuf,
}

pub const APP_FOLDER: &str = "Opsis";
pub const USER_DATA_ENV: &str = "OPSIS_USER_DATA";

pub fn user_data(folders: &HostFolders) -> PathBuf {
    env_dir(USER_DATA_ENV).unwrap_or_else(|| folders.config.join(APP_FOLDER))
}

pub fn host_folders() -> HostFolders {
    let home = home();
    HostFolders {
        config: config(&home),
        resources: resources(),
        home,
    }
}

#[cfg(all(test, not(target_os = "macos")))]
mod tests {
    use super::*;

    fn notices_in(dir: &Path) {
        std::fs::create_dir_all(dir).expect("a folder");
        std::fs::write(dir.join(NOTICES), "").expect("the notices");
    }

    #[test]
    fn a_folder_with_its_resources_beside_the_program_reads_them_there() {
        let root = tempfile::tempdir().expect("a temp folder");
        let program = root.path().join("opsis");
        notices_in(&program);
        notices_in(&root.path().join("lib").join("opsis"));
        assert_eq!(resources_for(&program), program);
    }

    #[test]
    fn an_installed_package_reads_its_resources_from_lib() {
        let root = tempfile::tempdir().expect("a temp folder");
        let bin = root.path().join("usr").join("bin");
        std::fs::create_dir_all(&bin).expect("usr/bin");
        notices_in(&root.path().join("usr").join("lib").join("opsis"));
        assert_eq!(
            resources_for(&bin),
            bin.join("..").join("lib").join("opsis")
        );
    }

    #[test]
    fn a_program_with_no_resources_anywhere_looks_beside_itself() {
        let root = tempfile::tempdir().expect("a temp folder");
        assert_eq!(resources_for(root.path()), root.path());
    }
}
