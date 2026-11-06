use std::path::PathBuf;

use opsis_session::paths::HostFolders;

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

fn resources() -> PathBuf {
    let beside = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(PathBuf::from))
        .unwrap_or_default();
    if cfg!(target_os = "macos") {
        beside.join("..").join("Resources")
    } else {
        beside
    }
}

pub fn host_folders() -> HostFolders {
    let home = home();
    HostFolders {
        config: config(&home),
        resources: resources(),
        home,
    }
}
