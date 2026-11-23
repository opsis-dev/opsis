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
        .env("XDG_DATA_HOME", folder.join("data"))
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
    if !passed {
        return (passed, format!("the app's log:\n{output}"));
    }
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

#[cfg(unix)]
fn ask(socket: &Path, path: &str) -> String {
    use std::io::{Read, Write};
    let mut stream = std::os::unix::net::UnixStream::connect(socket).expect("the socket answers");
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"
    )
    .expect("a request");
    let mut answer = String::new();
    stream.read_to_string(&mut answer).expect("an answer");
    answer
}

#[cfg(unix)]
struct Headless {
    app: std::process::Child,
    project: std::path::PathBuf,
    user_data: std::path::PathBuf,
    log_path: std::path::PathBuf,
    shown: Option<String>,
    _folder: tempfile::TempDir,
}

#[cfg(unix)]
impl Headless {
    fn start() -> Self {
        let folder = tempfile::tempdir().expect("a temp folder");
        let project = folder.path().join("project");
        std::fs::create_dir_all(&project).expect("a project folder");
        std::fs::copy(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/todomvc/project.opsis.json"),
            project.join("project.opsis.json"),
        )
        .expect("a project");
        let user_data = folder.path().join("user-data");
        let socket = user_data.join("api.sock");
        let log_path = folder.path().join("headless.log");
        let log = File::create(&log_path).expect("a log file");
        let mut app = Command::new(env!("CARGO_BIN_EXE_opsis"))
            .arg("--headless")
            .arg(&project)
            .env("OPSIS_USER_DATA", &user_data)
            .env("XDG_DATA_HOME", folder.path().join("data"))
            .env("OPSIS_AUDIO_DEVICE", "alsa:null")
            .env_remove("QT_QPA_PLATFORM")
            .env_remove("OPSIS_API_SOCKET")
            .env_remove("WAYLAND_DISPLAY")
            .env_remove("DISPLAY")
            .stdout(Stdio::from(log.try_clone().expect("a second log handle")))
            .stderr(Stdio::from(log))
            .spawn()
            .expect("the app starts");
        let started = Instant::now();
        let shown = loop {
            if socket.exists() {
                let answer = ask(&socket, "/project");
                if answer.starts_with("HTTP/1.1 200") {
                    break Some(answer);
                }
            }
            if started.elapsed() > LIMIT || app.try_wait().expect("the app's status").is_some() {
                break None;
            }
            std::thread::sleep(POLL);
        };
        Self {
            app,
            project,
            user_data,
            log_path,
            shown,
            _folder: folder,
        }
    }

    fn log(&self) -> String {
        std::fs::read_to_string(&self.log_path).expect("the log")
    }

    fn end(&mut self) -> std::process::ExitStatus {
        let pid = self.app.id().to_string();
        Command::new("kill")
            .args(["-TERM", &pid])
            .status()
            .expect("a signal");
        self.app.wait().expect("the app ends")
    }
}

#[cfg(unix)]
#[test]
fn a_headless_start_serves_the_project_and_ends_on_a_signal() {
    let mut headless = Headless::start();
    let output = headless.log();
    let shown = headless
        .shown
        .clone()
        .unwrap_or_else(|| panic!("the project was never served:\n{output}"));
    let status = headless.end();
    assert!(shown.contains("Sample: to-do app"), "{shown}");
    assert!(output.contains("platform offscreen"), "{output}");
    assert!(status.success(), "{status:?}\n{output}");
}

#[cfg(unix)]
fn opsis_mcp() -> std::path::PathBuf {
    let program = Path::new(env!("CARGO_BIN_EXE_opsis")).with_file_name("opsis-mcp");
    if !program.exists() {
        let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
        let built = Command::new(cargo)
            .args(["build", "-p", "opsis-mcp"])
            .status()
            .expect("cargo runs");
        assert!(built.success(), "opsis-mcp didn't build");
    }
    program
}

#[cfg(unix)]
#[test]
fn an_agent_changes_the_headless_project_through_opsis_mcp() {
    use std::io::{BufRead, BufReader, Write};
    let mut headless = Headless::start();
    assert!(
        headless.shown.is_some(),
        "the project was never served:\n{}",
        headless.log()
    );
    let mut mcp = Command::new(opsis_mcp())
        .arg("stdio")
        .env("OPSIS_USER_DATA", &headless.user_data)
        .env_remove("OPSIS_API_SOCKET")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("opsis-mcp starts");
    let mut input = mcp.stdin.take().expect("its input");
    let messages = [
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"1"}}}"#,
        r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"run_lua","arguments":{"source":"opsis.project.name = 'Named by an agent'\nreturn opsis.project.name"}}}"#,
    ];
    for message in messages {
        writeln!(input, "{message}").expect("a message");
    }
    let lines = BufReader::new(mcp.stdout.take().expect("its output")).lines();
    let answer = lines
        .map_while(Result::ok)
        .find(|line| line.contains(r#""id":2"#))
        .expect("an answer to the call");
    drop(input);
    mcp.wait().expect("opsis-mcp ends");
    let status = headless.end();
    assert!(answer.contains(r#""isError":false"#), "{answer}");
    assert!(answer.contains("Named by an agent"), "{answer}");
    let saved =
        std::fs::read_to_string(headless.project.join("project.opsis.json")).expect("the project");
    assert!(
        saved.contains("Named by an agent"),
        "the change wasn't saved"
    );
    assert!(status.success(), "{status:?}");
}
