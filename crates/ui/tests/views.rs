use std::fs::File;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const LIMIT: Duration = Duration::from_secs(180);
const POLL: Duration = Duration::from_millis(200);

fn run_scenario(name: &str, folder: &Path, launch: usize) -> (bool, String) {
    let projects = folder.join("projects");
    std::fs::create_dir_all(&projects).expect("a projects folder");
    let log_path = folder.join(format!("{name}-{launch}.log"));
    let log = File::create(&log_path).expect("a log file");
    let mut app = Command::new(env!("CARGO_BIN_EXE_opsis"))
        .env("OPSIS_USER_DATA", folder.join("user-data"))
        .env("OPSIS_QT_SCENARIO", name)
        .env("OPSIS_QT_PROJECTS", &projects)
        .env("OPSIS_QT_SHOTS", "off")
        .env("OPSIS_AUDIO_DEVICE", "alsa:null")
        .env("QT_QPA_PLATFORM", "offscreen")
        .env_remove("WAYLAND_DISPLAY")
        .env_remove("DISPLAY")
        .stdout(Stdio::from(log.try_clone().expect("a second log handle")))
        .stderr(Stdio::from(log))
        .spawn()
        .expect("the app starts");
    let started = Instant::now();
    let status = loop {
        if let Some(status) = app.try_wait().expect("the app's status") {
            break Some(status);
        }
        if started.elapsed() > LIMIT {
            app.kill().expect("stopping the app");
            break None;
        }
        std::thread::sleep(POLL);
    };
    let output = std::fs::read_to_string(&log_path).expect("the log");
    let done = format!("scenario {name} done");
    let passed = status.is_some_and(|s| s.success())
        && output.contains(&done)
        && !output.contains(" FAILED ");
    let checks: Vec<&str> = output
        .lines()
        .filter(|line| line.contains("check ") || line.contains(&done))
        .collect();
    (passed, checks.join("\n"))
}

#[test]
fn the_views_scenario_passes_on_the_offscreen_platform() {
    let folder = tempfile::tempdir().expect("a temp folder");
    let (passed, checks) = run_scenario("views", folder.path(), 1);
    assert!(passed, "the views scenario failed:\n{checks}");
}

#[test]
fn a_popped_out_panel_comes_back_popped_out_when_the_app_starts_again() {
    let folder = tempfile::tempdir().expect("a temp folder");
    let (first, checks) = run_scenario("reopen", folder.path(), 1);
    assert!(first, "the first launch failed:\n{checks}");
    let (second, checks) = run_scenario("reopen", folder.path(), 2);
    assert!(second, "the second launch failed:\n{checks}");
    assert!(
        checks.contains("reopen-the-timeline-is-still-popped-out"),
        "the second launch didn't check the timeline:\n{checks}"
    );
}
