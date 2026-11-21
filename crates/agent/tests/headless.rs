#![cfg(unix)]

mod support;

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use agent_client_protocol::Client;
use agent_client_protocol::schema::ProtocolVersion;
use agent_client_protocol::v1::SessionSetup;
use agent_client_protocol::v1::schema::{
    EnvVariable, InitializeRequest, McpServer, McpServerStdio, NewSessionRequest, StopReason,
};
use support::*;

const START_WITHIN: Duration = Duration::from_secs(120);
const NAME: &str = "Named by opsis-agent";

fn beside_the_agent(program: &str, package: &str) -> PathBuf {
    let path = Path::new(PROGRAM).with_file_name(program);
    if !path.exists() {
        let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
        let built = Command::new(cargo)
            .args(["build", "-p", package])
            .status()
            .expect("cargo runs");
        assert!(built.success(), "{program} didn't build");
    }
    path
}

fn ask(socket: &Path, path: &str) -> String {
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

struct Headless(Child);

impl Drop for Headless {
    fn drop(&mut self) {
        self.0.kill().ok();
        self.0.wait().ok();
    }
}

fn start_headless(project: &Path, user_data: &Path) -> Headless {
    let app = Command::new(beside_the_agent("opsis", "opsis-ui"))
        .arg("--headless")
        .arg(project)
        .env("OPSIS_USER_DATA", user_data)
        .env("OPSIS_AUDIO_DEVICE", "alsa:null")
        .env_remove("OPSIS_API_SOCKET")
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("the app starts");
    let socket = user_data.join("api.sock");
    let started = Instant::now();
    while !(socket.exists() && ask(&socket, "/project").starts_with("HTTP/1.1 200")) {
        assert!(
            started.elapsed() < START_WITHIN,
            "the headless app never served the project"
        );
        std::thread::sleep(Duration::from_millis(200));
    }
    Headless(app)
}

#[tokio::test(flavor = "multi_thread")]
async fn an_agent_turn_changes_a_headless_project_through_opsis_mcp() {
    let folder = tempfile::tempdir().unwrap();
    let project = folder.path().join("project");
    std::fs::create_dir_all(&project).unwrap();
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/todomvc/project.opsis.json"),
        project.join("project.opsis.json"),
    )
    .unwrap();
    let user_data = folder.path().join("data");
    let mcp = beside_the_agent("opsis-mcp", "opsis-mcp");
    let _app = start_headless(&project, &user_data);
    let source = format!("opsis.project.name = '{NAME}'\nreturn opsis.project.name");
    let arguments = serde_json::json!({ "source": source }).to_string();
    let server = scripted(vec![calls("opsis__run_lua", &arguments), says("Done.")]).await;
    log_in(&user_data, &server);
    let request = NewSessionRequest::new(&project).mcp_servers(vec![McpServer::Stdio(
        McpServerStdio::new("opsis", mcp)
            .args(vec!["stdio".to_owned()])
            .env(vec![EnvVariable::new(
                "OPSIS_USER_DATA",
                user_data.to_string_lossy(),
            )]),
    )]);
    let turned = Client
        .v1()
        .connect_with(agent(&user_data), async |cx| {
            cx.send_request(InitializeRequest::new(ProtocolVersion::V1))
                .block_task()
                .await?;
            cx.build_session_from(request)
                .block_task()
                .run_until(async |SessionSetup { mut session, .. }| {
                    session.send_prompt("Rename the project")?;
                    Ok(answer_turn(&mut session, "allow").await)
                })
                .await
        })
        .await
        .unwrap();
    assert_eq!(turned.stop, Some(StopReason::EndTurn), "{turned:?}");
    assert_eq!((turned.asked, turned.tools), (1, 1), "{turned:?}");
    let shown = ask(&user_data.join("api.sock"), "/project");
    assert!(shown.contains(NAME), "{shown}");
}
