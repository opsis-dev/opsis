#![allow(
    dead_code,
    reason = "each test file uses only some of the shared helpers"
)]

use std::path::Path;
use std::time::Duration;

use agent_client_protocol::schema::ProtocolVersion;
use agent_client_protocol::util::MatchDispatch;
use agent_client_protocol::v1::schema::{
    ContentBlock, ContentChunk, InitializeRequest, RequestPermissionOutcome,
    RequestPermissionRequest, RequestPermissionResponse, SelectedPermissionOutcome,
    SessionNotification, SessionUpdate, SetSessionModeRequest, StopReason,
};
use agent_client_protocol::v1::{ActiveSession, SessionMessage, SessionSetup};
use agent_client_protocol::{AcpAgent, AcpAgentConfig, Agent, Client};
use wiremock::matchers::{body_string_contains, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

pub const PROGRAM: &str = env!("CARGO_BIN_EXE_opsis-agent");

pub fn chunk(delta: &str, finish: &str) -> String {
    format!(
        "data: {{\"id\":\"c1\",\"object\":\"chat.completion.chunk\",\"created\":1,\"model\":\"m\",\"choices\":[{{\"index\":0,\"delta\":{delta},\"finish_reason\":{finish}}}]}}\n\n"
    )
}

pub fn streamed(chunks: &[String]) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_raw(
        format!("{}data: [DONE]\n\n", chunks.concat()),
        "text/event-stream",
    )
}

pub fn writes_notes() -> ResponseTemplate {
    let call = r#"{"role":"assistant","tool_calls":[{"index":0,"id":"call_1","type":"function","function":{"name":"write","arguments":"{\"path\":\"notes.txt\",\"content\":\"hello\"}"}}]}"#;
    streamed(&[chunk(call, "null"), chunk("{}", "\"tool_calls\"")])
}

pub fn calls(name: &str, arguments: &str) -> ResponseTemplate {
    let arguments = serde_json::to_string(arguments).unwrap();
    let call = format!(
        r#"{{"role":"assistant","tool_calls":[{{"index":0,"id":"call_{name}","type":"function","function":{{"name":"{name}","arguments":{arguments}}}}}]}}"#
    );
    streamed(&[chunk(&call, "null"), chunk("{}", "\"tool_calls\"")])
}

pub type Bodies = std::sync::Arc<std::sync::Mutex<Vec<String>>>;

#[derive(Debug)]
pub struct Script {
    replies: std::sync::Mutex<std::collections::VecDeque<ResponseTemplate>>,
    bodies: Bodies,
}

impl wiremock::Respond for Script {
    fn respond(&self, request: &wiremock::Request) -> ResponseTemplate {
        self.bodies
            .lock()
            .unwrap()
            .push(String::from_utf8_lossy(&request.body).into_owned());
        self.replies
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_else(says_done)
    }
}

pub async fn scripted_with(replies: Vec<ResponseTemplate>) -> (MockServer, Bodies) {
    let server = MockServer::start().await;
    let bodies = Bodies::default();
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(Script {
            replies: std::sync::Mutex::new(replies.into()),
            bodies: bodies.clone(),
        })
        .mount(&server)
        .await;
    (server, bodies)
}

pub async fn scripted(replies: Vec<ResponseTemplate>) -> MockServer {
    scripted_with(replies).await.0
}

pub fn says(text: &str) -> ResponseTemplate {
    let delta = serde_json::json!({ "role": "assistant", "content": text }).to_string();
    streamed(&[chunk(&delta, "null"), chunk("{}", "\"stop\"")])
}

pub async fn later_text(session: &mut ActiveSession<'_, Agent>, wanted: &str) -> String {
    let mut later = String::new();
    while !later.contains(wanted) {
        let Ok(Ok(SessionMessage::Message(dispatch))) =
            tokio::time::timeout(Duration::from_secs(10), session.read_update()).await
        else {
            break;
        };
        let text = std::sync::Mutex::new(String::new());
        MatchDispatch::new(dispatch.into_raw())
            .if_notification(async |notification: SessionNotification| {
                if let SessionUpdate::AgentMessageChunk(ContentChunk {
                    content: ContentBlock::Text(chunk),
                    ..
                }) = notification.update
                {
                    text.lock().unwrap().push_str(&chunk.text);
                }
                Ok(())
            })
            .await
            .otherwise_ignore()
            .unwrap();
        later.push_str(&text.into_inner().unwrap());
    }
    later
}

pub async fn one_turn(
    server: &MockServer,
    mode: Option<&'static str>,
    answer: &'static str,
    wait_for: Option<&'static str>,
) -> (Turned, String, tempfile::TempDir) {
    let user_data = tempfile::tempdir().unwrap();
    let folder = tempfile::tempdir().unwrap();
    log_in(user_data.path(), server);
    let folder_path = folder.path().to_path_buf();
    let ended = Client
        .v1()
        .connect_with(agent(user_data.path()), async |cx| {
            cx.send_request(InitializeRequest::new(ProtocolVersion::V1))
                .block_task()
                .await?;
            cx.build_session(&folder_path)
                .block_task()
                .run_until(async |SessionSetup { mut session, .. }| {
                    if let Some(mode) = mode {
                        session
                            .connection()
                            .send_request(SetSessionModeRequest::new(
                                session.session_id().clone(),
                                mode,
                            ))
                            .block_task()
                            .await?;
                    }
                    session.send_prompt("Go")?;
                    let turned = answer_turn(&mut session, answer).await;
                    let later = match wait_for {
                        Some(wanted) => later_text(&mut session, wanted).await,
                        None => String::new(),
                    };
                    Ok((turned, later))
                })
                .await
        })
        .await
        .unwrap();
    (ended.0, ended.1, folder)
}

pub fn says_done() -> ResponseTemplate {
    streamed(&[
        chunk(r#"{"role":"assistant","content":"Done."}"#, "null"),
        chunk("{}", "\"stop\""),
    ])
}

pub async fn model_server(first: ResponseTemplate) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .and(body_string_contains("\"role\":\"tool\""))
        .respond_with(says_done())
        .with_priority(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(first)
        .with_priority(5)
        .mount(&server)
        .await;
    server
}

pub fn log_in(user_data: &Path, server: &MockServer) {
    use std::io::Write;
    let mut login = std::process::Command::new(PROGRAM)
        .arg("login")
        .env("OPSIS_USER_DATA", user_data)
        .stdin(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let mut typed = login.stdin.take().unwrap();
    writeln!(typed, "local\nm\n{}", server.uri()).unwrap();
    drop(typed);
    assert!(login.wait().unwrap().success());
}

pub fn agent(user_data: &Path) -> AcpAgent {
    AcpAgent::new(AcpAgentConfig::new(PROGRAM).env("OPSIS_USER_DATA", user_data.to_string_lossy()))
}

#[derive(Debug, Default)]
pub struct Turned {
    pub text: String,
    pub asked: usize,
    pub tools: usize,
    pub stop: Option<StopReason>,
}

pub async fn read_turn(session: &mut ActiveSession<'_, Agent>) -> Turned {
    answer_turn(session, "allow").await
}

pub async fn answer_turn(session: &mut ActiveSession<'_, Agent>, answer: &'static str) -> Turned {
    let mut turned = Turned::default();
    loop {
        match session.read_update().await.unwrap() {
            SessionMessage::Message(dispatch) => {
                let asked = std::sync::atomic::AtomicUsize::new(0);
                let notice = std::sync::Mutex::new(None);
                MatchDispatch::new(dispatch.into_raw())
                    .if_request(async |_request: RequestPermissionRequest, responder| {
                        asked.store(1, std::sync::atomic::Ordering::Relaxed);
                        responder.respond(RequestPermissionResponse::new(
                            RequestPermissionOutcome::Selected(SelectedPermissionOutcome::new(
                                answer,
                            )),
                        ))
                    })
                    .await
                    .if_notification(async |notification: SessionNotification| {
                        *notice.lock().unwrap() = Some(notification.update);
                        Ok(())
                    })
                    .await
                    .otherwise_ignore()
                    .unwrap();
                turned.asked += asked.load(std::sync::atomic::Ordering::Relaxed);
                match notice.into_inner().unwrap() {
                    Some(SessionUpdate::AgentMessageChunk(ContentChunk {
                        content: ContentBlock::Text(text),
                        ..
                    })) => {
                        turned.text.push_str(&text.text);
                    }
                    Some(SessionUpdate::ToolCall(_)) => turned.tools += 1,
                    _ => {}
                }
            }
            SessionMessage::PromptResponse(response) => {
                turned.stop = Some(response.unwrap().stop_reason);
                return turned;
            }
            _ => {}
        }
    }
}
