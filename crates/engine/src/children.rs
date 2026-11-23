use std::collections::BTreeSet;
use std::path::Path;
use std::sync::Mutex;

use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, Signal, System, UpdateKind};
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};

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

pub fn processes() -> Option<System> {
    System::new()
        .inspect_err(|error| log::warn!("couldn't read the computer's processes: {error}"))
        .ok()
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
    let Some(mut system) = processes() else {
        return Vec::new();
    };
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
    let Some(mut system) = processes() else {
        return Vec::new();
    };
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

pub async fn read_lines(stream: impl AsyncRead + Unpin, mut each: impl FnMut(&str)) {
    let mut lines = BufReader::new(stream).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        each(&line);
    }
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

    fn profile_in(root: &Path, inside: &str) -> String {
        format!("{PROFILE_SWITCH}{}", root.join(inside).display())
    }

    #[test]
    fn reads_the_app_that_owns_a_profile_in_the_temp_folder() {
        let root = Path::new("/data/Opsis/tmp");
        assert_eq!(
            temp_owner_of(
                &profile_in(root, "2175263-shell-views/opsis-profile-aohrOW"),
                root
            ),
            Some(2_175_263)
        );
        assert_eq!(
            temp_owner_of(
                &format!(
                    "/opt/chrome-headless-shell --headless {} --no-sandbox",
                    profile_in(root, "1628536-run-97bedae8/opsis-profile-AGhC5V")
                ),
                root
            ),
            Some(1_628_536)
        );
        let spaced = Path::new("/Users/me/Library/Application Support/Opsis/tmp");
        assert_eq!(
            temp_owner_of(
                &format!("chrome {}", profile_in(spaced, "77-shell-views/p")),
                spaced
            ),
            Some(77)
        );
        assert_eq!(
            temp_owner_of("--user-data-dir=/home/me/.config/google-chrome", root),
            None
        );
        assert_eq!(
            temp_owner_of(
                &format!("{PROFILE_SWITCH}{}files/12-run-x/p", root.display()),
                root
            ),
            None
        );
        assert_eq!(
            temp_owner_of(
                &format!("--disk-cache-dir={}", root.join("12-run-x/p").display()),
                root
            ),
            None
        );
        assert_eq!(temp_owner_of(&profile_in(root, "stray/p"), root), None);
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
        assert_eq!(
            end_children_of_gone_apps(root.path(), &|_| true),
            [] as [u32; 0]
        );
        child.kill().unwrap();
        child.wait().unwrap();
    }

    #[test]
    fn ends_the_children_it_was_told_about_once() {
        let mut child = holding("--no-profile");
        remember_child(child.id());
        assert_eq!(end_remembered_children(), [child.id()]);
        assert!(!child.wait().unwrap().success());
        assert_eq!(end_remembered_children(), [] as [u32; 0]);
    }
}
