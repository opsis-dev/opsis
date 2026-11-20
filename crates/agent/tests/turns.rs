use std::path::Path;
use std::time::Duration;

use agent_client_protocol::schema::ProtocolVersion;
use agent_client_protocol::util::MatchDispatch;
use agent_client_protocol::v1::schema::{
    CancelNotification, ContentBlock, ContentChunk, InitializeRequest, RequestPermissionOutcome,
    RequestPermissionRequest, RequestPermissionResponse, SelectedPermissionOutcome,
    SessionNotification, SessionUpdate, SetSessionModeRequest, StopReason,
};
use agent_client_protocol::v1::{ActiveSession, SessionMessage, SessionSetup};
use agent_client_protocol::{AcpAgent, AcpAgentConfig, Agent, Client};
use wiremock::matchers::{body_string_contains, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const PROGRAM: &str = env!("CARGO_BIN_EXE_opsis-agent");

fn chunk(delta: &str, finish: &str) -> String {
    format!(
        "data: {{\"id\":\"c1\",\"object\":\"chat.completion.chunk\",\"created\":1,\"model\":\"m\",\"choices\":[{{\"index\":0,\"delta\":{delta},\"finish_reason\":{finish}}}]}}\n\n"
    )
}

fn streamed(chunks: &[String]) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_raw(
        format!("{}data: [DONE]\n\n", chunks.concat()),
        "text/event-stream",
    )
}

fn writes_notes() -> ResponseTemplate {
    let call = r#"{"role":"assistant","tool_calls":[{"index":0,"id":"call_1","type":"function","function":{"name":"write","arguments":"{\"path\":\"notes.txt\",\"content\":\"hello\"}"}}]}"#;
    streamed(&[chunk(call, "null"), chunk("{}", "\"tool_calls\"")])
}

fn calls(name: &str, arguments: &str) -> ResponseTemplate {
    let arguments = serde_json::to_string(arguments).unwrap();
    let call = format!(
        r#"{{"role":"assistant","tool_calls":[{{"index":0,"id":"call_{name}","type":"function","function":{{"name":"{name}","arguments":{arguments}}}}}]}}"#
    );
    streamed(&[chunk(&call, "null"), chunk("{}", "\"tool_calls\"")])
}

#[derive(Debug)]
struct Script(std::sync::Mutex<std::collections::VecDeque<ResponseTemplate>>);

impl wiremock::Respond for Script {
    fn respond(&self, _request: &wiremock::Request) -> ResponseTemplate {
        self.0.lock().unwrap().pop_front().unwrap_or_else(says_done)
    }
}

async fn scripted(replies: Vec<ResponseTemplate>) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(Script(std::sync::Mutex::new(replies.into())))
        .mount(&server)
        .await;
    server
}

async fn later_text(session: &mut ActiveSession<'_, Agent>, wanted: &str) -> String {
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

async fn one_turn(
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

fn says_done() -> ResponseTemplate {
    streamed(&[
        chunk(r#"{"role":"assistant","content":"Done."}"#, "null"),
        chunk("{}", "\"stop\""),
    ])
}

async fn model_server(first: ResponseTemplate) -> MockServer {
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

fn log_in(user_data: &Path, server: &MockServer) {
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

fn agent(user_data: &Path) -> AcpAgent {
    AcpAgent::new(AcpAgentConfig::new(PROGRAM).env("OPSIS_USER_DATA", user_data.to_string_lossy()))
}

#[derive(Debug, Default)]
struct Turned {
    text: String,
    asked: usize,
    tools: usize,
    stop: Option<StopReason>,
}

async fn read_turn(session: &mut ActiveSession<'_, Agent>) -> Turned {
    answer_turn(session, "allow").await
}

async fn answer_turn(session: &mut ActiveSession<'_, Agent>, answer: &'static str) -> Turned {
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

#[tokio::test(flavor = "multi_thread")]
async fn a_turn_asks_before_writing_runs_the_tool_and_answers_then_loads_again() {
    let user_data = tempfile::tempdir().unwrap();
    let folder = tempfile::tempdir().unwrap();
    let server = model_server(writes_notes()).await;
    log_in(user_data.path(), &server);
    let folder_path = folder.path().to_path_buf();
    let session_id = Client
        .v1()
        .connect_with(agent(user_data.path()), async |cx| {
            cx.send_request(InitializeRequest::new(ProtocolVersion::V1))
                .block_task()
                .await?;
            cx.build_session(&folder_path)
                .block_task()
                .run_until(async |SessionSetup { mut session, .. }| {
                    session.send_prompt("Write hello into notes.txt")?;
                    let turned = read_turn(&mut session).await;
                    assert_eq!(turned.stop, Some(StopReason::EndTurn));
                    assert_eq!(turned.asked, 1, "{turned:?}");
                    assert_eq!(turned.tools, 1);
                    assert_eq!(turned.text, "Done.");
                    Ok(session.session_id().clone())
                })
                .await
        })
        .await
        .unwrap();
    assert_eq!(
        std::fs::read_to_string(folder.path().join("notes.txt")).unwrap(),
        "hello"
    );
    let replayed = Client
        .v1()
        .connect_with(agent(user_data.path()), async |cx| {
            cx.send_request(InitializeRequest::new(ProtocolVersion::V1))
                .block_task()
                .await?;
            let SessionSetup { mut session, .. } = cx
                .load_session(session_id.clone(), &folder_path)
                .block_task()
                .start_session()
                .await?;
            let mut replayed = String::new();
            while let Ok(Ok(SessionMessage::Message(dispatch))) =
                tokio::time::timeout(Duration::from_secs(1), session.read_update()).await
            {
                MatchDispatch::new(dispatch.into_raw())
                    .if_notification(async |notification: SessionNotification| {
                        if let SessionUpdate::AgentMessageChunk(ContentChunk {
                            content: ContentBlock::Text(text),
                            ..
                        })
                        | SessionUpdate::UserMessageChunk(ContentChunk {
                            content: ContentBlock::Text(text),
                            ..
                        }) = notification.update
                        {
                            replayed.push_str(&text.text);
                            replayed.push('\n');
                        }
                        Ok(())
                    })
                    .await
                    .otherwise_ignore()?;
            }
            Ok(replayed)
        })
        .await
        .unwrap();
    assert_eq!(replayed, "Write hello into notes.txt\nDone.\n");
}

#[tokio::test(flavor = "multi_thread")]
async fn cancelling_ends_the_turn_while_the_model_is_still_answering() {
    let user_data = tempfile::tempdir().unwrap();
    let folder = tempfile::tempdir().unwrap();
    let server = model_server(says_done().set_delay(Duration::from_secs(30))).await;
    log_in(user_data.path(), &server);
    let folder_path = folder.path().to_path_buf();
    let stop = Client
        .v1()
        .connect_with(agent(user_data.path()), async |cx| {
            cx.send_request(InitializeRequest::new(ProtocolVersion::V1))
                .block_task()
                .await?;
            cx.build_session(&folder_path)
                .block_task()
                .run_until(async |SessionSetup { mut session, .. }| {
                    session.send_prompt("Say something")?;
                    tokio::time::sleep(Duration::from_millis(300)).await;
                    session
                        .connection()
                        .send_notification(CancelNotification::new(session.session_id().clone()))?;
                    let turned =
                        tokio::time::timeout(Duration::from_secs(10), read_turn(&mut session))
                            .await
                            .expect("the turn ended");
                    Ok(turned.stop)
                })
                .await
        })
        .await
        .unwrap();
    assert_eq!(stop, Some(StopReason::Cancelled));
}

#[tokio::test(flavor = "multi_thread")]
async fn allowing_always_stops_asking_for_the_same_tool() {
    let server = scripted(vec![
        calls("write", r#"{"path":"a.txt","content":"one"}"#),
        calls("write", r#"{"path":"b.txt","content":"two"}"#),
    ])
    .await;
    let (turned, _, folder) = one_turn(&server, None, "allow-always", None).await;
    assert_eq!((turned.asked, turned.tools), (1, 2), "{turned:?}");
    assert_eq!(
        std::fs::read_to_string(folder.path().join("b.txt")).unwrap(),
        "two"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_command_that_only_reads_runs_without_asking() {
    let server = scripted(vec![calls("shell", r#"{"command":"ls"}"#)]).await;
    let (turned, _, _) = one_turn(&server, None, "reject", None).await;
    assert_eq!((turned.asked, turned.tools), (0, 1), "{turned:?}");
    assert_eq!(turned.stop, Some(StopReason::EndTurn));
}

#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
async fn full_mode_never_asks_and_a_background_command_reports_when_it_ends() {
    let server = scripted(vec![calls(
        "shell",
        r#"{"command":"sleep 1 && echo from-the-background","background":true}"#,
    )])
    .await;
    let (turned, later, _) =
        one_turn(&server, Some("full"), "reject", Some("from-the-background")).await;
    assert_eq!((turned.asked, turned.tools), (0, 1), "{turned:?}");
    assert!(later.contains("from-the-background"), "{later}");
}
