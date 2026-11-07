use std::fs::File;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const LIMIT: Duration = Duration::from_secs(180);
const POLL: Duration = Duration::from_millis(200);

#[test]
fn the_views_scenario_passes_on_the_offscreen_platform() {
    let folder = tempfile::tempdir().expect("a temp folder");
    let projects = folder.path().join("projects");
    std::fs::create_dir_all(&projects).expect("a projects folder");
    let log_path = folder.path().join("views.log");
    let log = File::create(&log_path).expect("a log file");
    let mut app = Command::new(env!("CARGO_BIN_EXE_opsis"))
        .env("OPSIS_USER_DATA", folder.path().join("user-data"))
        .env("OPSIS_QT_SCENARIO", "views")
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
    let checks: Vec<&str> = output
        .lines()
        .filter(|line| line.contains("check ") || line.contains("scenario views done"))
        .collect();
    assert!(
        status.is_some_and(|s| s.success())
            && output.contains("scenario views done")
            && !output.contains(" FAILED "),
        "the views scenario failed:\n{}",
        checks.join("\n")
    );
}
