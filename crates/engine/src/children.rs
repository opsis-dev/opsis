use std::collections::BTreeSet;
use std::path::Path;
use std::sync::Mutex;

use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, Signal, System, UpdateKind};

static STARTED: Mutex<BTreeSet<u32>> = Mutex::new(BTreeSet::new());

const PROFILE_SWITCH: &str = "--user-data-dir=";

fn started() -> std::sync::MutexGuard<'static, BTreeSet<u32>> {
    STARTED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub fn remember_child(pid: u32) {
    started().insert(pid);
}

pub fn forget_child(pid: u32) {
    started().remove(&pid);
}

fn end(system: &System, pid: u32) -> bool {
    system.process(Pid::from_u32(pid)).is_some_and(|process| {
        process
            .kill_with(Signal::Term)
            .unwrap_or_else(|| process.kill())
    })
}

pub fn end_remembered_children() -> Vec<u32> {
    let pids: Vec<u32> = std::mem::take(&mut *started()).into_iter().collect();
    if pids.is_empty() {
        return pids;
    }
    let mut system = System::new();
    let wanted: Vec<Pid> = pids.iter().copied().map(Pid::from_u32).collect();
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&wanted),
        true,
        ProcessRefreshKind::nothing(),
    );
    pids.into_iter().filter(|&pid| end(&system, pid)).collect()
}

pub fn temp_owner_of(command_line: &str, root: &Path) -> Option<u32> {
    let marker = format!(
        "{PROFILE_SWITCH}{}{}",
        root.display(),
        std::path::MAIN_SEPARATOR
    );
    let after = &command_line[command_line.find(&marker)? + marker.len()..];
    let (pid, _) = after.split_once('-')?;
    pid.parse().ok().filter(|p| *p != 0)
}

pub fn end_children_of_gone_apps(root: &Path, alive: &dyn Fn(u32) -> bool) -> Vec<u32> {
    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_cmd(UpdateKind::Always),
    );
    let me = std::process::id();
    let orphans: Vec<u32> = system
        .processes()
        .iter()
        .filter(|(_, process)| {
            let command_line = process
                .cmd()
                .iter()
                .map(|arg| arg.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" ");
            temp_owner_of(&command_line, root).is_some_and(|owner| owner != me && !alive(owner))
        })
        .map(|(pid, _)| pid.as_u32())
        .collect();
    orphans
        .into_iter()
        .filter(|&pid| end(&system, pid))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn holding(arg: &str) -> std::process::Child {
        std::process::Command::new("sh")
            .args([
                "-c",
                "sleep 60; true",
                "sh",
                "--headless",
                arg,
                "--no-sandbox",
            ])
            .spawn()
            .unwrap()
    }

    #[test]
    fn reads_the_app_that_owns_a_profile_in_the_temp_folder() {
        let root = Path::new("/data/Opsis/tmp");
        assert_eq!(
            temp_owner_of(
                "--user-data-dir=/data/Opsis/tmp/2175263-shell-views/opsis-profile-aohrOW",
                root
            ),
            Some(2_175_263)
        );
        assert_eq!(
            temp_owner_of(
                "/opt/chrome-headless-shell --headless --user-data-dir=/data/Opsis/tmp/1628536-run-97bedae8/opsis-profile-AGhC5V --no-sandbox",
                root
            ),
            Some(1_628_536)
        );
        assert_eq!(
            temp_owner_of(
                "chrome --user-data-dir=/Users/me/Library/Application Support/Opsis/tmp/77-shell-views/p",
                Path::new("/Users/me/Library/Application Support/Opsis/tmp")
            ),
            Some(77)
        );
        assert_eq!(
            temp_owner_of("--user-data-dir=/home/me/.config/google-chrome", root),
            None
        );
        assert_eq!(
            temp_owner_of("--user-data-dir=/data/Opsis/tmpfiles/12-run-x/p", root),
            None
        );
        assert_eq!(
            temp_owner_of("--disk-cache-dir=/data/Opsis/tmp/12-run-x/p", root),
            None
        );
        assert_eq!(
            temp_owner_of("--user-data-dir=/data/Opsis/tmp/stray/p", root),
            None
        );
    }

    #[test]
    fn ends_a_process_left_by_an_app_that_is_gone() {
        let root = tempfile::tempdir().unwrap();
        let profile = root.path().join("999999-run-gone0000").join("p");
        let mut child = holding(&format!("{PROFILE_SWITCH}{}", profile.display()));
        let ended = end_children_of_gone_apps(root.path(), &|pid| pid != 999_999);
        assert_eq!(ended, [child.id()]);
        assert!(!child.wait().unwrap().success());
    }

    #[test]
    fn leaves_a_process_whose_app_is_still_running() {
        let root = tempfile::tempdir().unwrap();
        let profile = root.path().join("999998-run-alive000").join("p");
        let mut child = holding(&format!("{PROFILE_SWITCH}{}", profile.display()));
        assert!(end_children_of_gone_apps(root.path(), &|_| true).is_empty());
        child.kill().unwrap();
        child.wait().unwrap();
    }

    #[test]
    fn ends_the_children_it_was_told_about_once() {
        let mut child = holding("--no-profile");
        remember_child(child.id());
        assert_eq!(end_remembered_children(), [child.id()]);
        assert!(!child.wait().unwrap().success());
        assert!(end_remembered_children().is_empty());
    }
}
