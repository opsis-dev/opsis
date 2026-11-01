use std::path::{Path, PathBuf};

use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate};

use crate::children::processes;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChildKind {
    Run,
    Analysis,
    Setup,
    Shell,
}

impl ChildKind {
    fn id(self) -> &'static str {
        match self {
            Self::Run => "run",
            Self::Analysis => "analysis",
            Self::Setup => "setup",
            Self::Shell => "shell",
        }
    }
}

pub fn child_temp_name(kind: ChildKind, id: &str, pid: u32) -> String {
    let short: String = id
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .take(8)
        .collect();
    format!("{pid}-{}-{short}", kind.id())
}

pub fn make_child_temp(root: &Path, name: &str) -> std::io::Result<PathBuf> {
    let dir = root.join(name);
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn remove_child_temp(dir: &Path) {
    for _ in 0..3 {
        match std::fs::remove_dir_all(dir) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => {}
            _ => return,
        }
    }
}

pub fn process_alive(pid: u32) -> bool {
    let Some(mut system) = processes() else {
        return true;
    };
    let pid = Pid::from_u32(pid);
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[pid]),
        true,
        ProcessRefreshKind::nothing(),
    );
    system.process(pid).is_some()
}

fn owner(name: &str) -> Option<u32> {
    let (pid, _) = name.split_once('-')?;
    pid.parse().ok().filter(|p| *p != 0)
}

pub fn clear_stale_child_temps(root: &Path, alive: &dyn Fn(u32) -> bool) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(root) else {
        return Vec::new();
    };
    let me = std::process::id();
    let stale: Vec<String> = entries
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|name| owner(name).is_none_or(|pid| pid != me && !alive(pid)))
        .collect();
    for name in &stale {
        remove_child_temp(&root.join(name));
    }
    stale
}

pub fn temp_env(dir: &Path) -> [(&'static str, PathBuf); 3] {
    [
        ("TMPDIR", dir.to_path_buf()),
        ("TEMP", dir.to_path_buf()),
        ("TMP", dir.to_path_buf()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_a_folder_after_the_app_process_the_kind_and_the_run() {
        assert_eq!(
            child_temp_name(ChildKind::Run, "c6bad309-c2f3-469a-bbc5-6a493245e67e", 4242),
            "4242-run-c6bad309"
        );
    }

    #[test]
    fn makes_and_removes_a_folder_with_everything_in_it() {
        let root = tempfile::tempdir().unwrap();
        let dir =
            make_child_temp(root.path(), &child_temp_name(ChildKind::Analysis, "abc", 1)).unwrap();
        std::fs::create_dir_all(dir.join("react-motion-render123").join("frames")).unwrap();
        remove_child_temp(&dir);
        assert!(!dir.exists());
    }

    #[test]
    fn clears_folders_left_by_processes_that_are_gone_and_keeps_live_ones() {
        let root = tempfile::tempdir().unwrap();
        let mine = format!("{}-run-mine0000", std::process::id());
        for name in [
            mine.as_str(),
            "111-run-alive000",
            "222-run-gone0000",
            "stray-folder",
        ] {
            std::fs::create_dir(root.path().join(name)).unwrap();
        }
        let mut removed = clear_stale_child_temps(root.path(), &|pid| pid == 111);
        removed.sort();
        assert_eq!(removed, ["222-run-gone0000", "stray-folder"]);
        let mut left: Vec<String> = std::fs::read_dir(root.path())
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        left.sort();
        let mut wanted = vec!["111-run-alive000".to_owned(), mine];
        wanted.sort();
        assert_eq!(left, wanted);
    }

    #[test]
    fn does_nothing_when_the_folder_doesnt_exist_yet() {
        let root = tempfile::tempdir().unwrap();
        assert_eq!(
            clear_stale_child_temps(&root.path().join("missing"), &|_| true),
            [] as [String; 0]
        );
    }

    #[test]
    #[cfg(unix)]
    fn tells_a_running_process_from_one_that_has_ended() {
        let mut child = std::process::Command::new("sleep")
            .arg("30")
            .spawn()
            .unwrap();
        let pid = child.id();
        assert!(process_alive(pid));
        child.kill().unwrap();
        child.wait().unwrap();
        assert!(!process_alive(pid));
    }
}
