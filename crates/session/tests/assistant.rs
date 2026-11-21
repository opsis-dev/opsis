use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use agent_client_protocol::v1::schema::{
    AuthMethod, AuthMethodAgent, AuthenticateRequest, AuthenticateResponse, ContentBlock,
    ContentChunk, InitializeRequest, InitializeResponse, McpServer, NewSessionRequest,
    NewSessionResponse, PermissionOption, PermissionOptionKind, PromptRequest, PromptResponse,
    RequestPermissionOutcome, RequestPermissionRequest, SessionMode, SessionModeState,
    SessionNotification, SessionUpdate, SetSessionModeRequest, SetSessionModeResponse, StopReason,
    TextContent, ToolCallUpdate, ToolCallUpdateFields,
};
use agent_client_protocol::{Agent, Channel, Error};
use opsis_folders::HostFolders;
use opsis_persistence::keys::KeyError;
use opsis_session::Session;
use opsis_session::assistant::{
    Command, Context, Entry, SIGN_IN, Status, WAITING, converse, panel_state,
};
use opsis_session::paths::AppPaths;
use opsis_session::platform::{Closed, FileFilter, PickedFiles, PickedFolder, Platform, Versions};
use opsis_session::state::AppState;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

const WITHIN: Duration = Duration::from_secs(10);

fn runtime() -> &'static Runtime {
    static SHARED: OnceLock<Runtime> = OnceLock::new();
    SHARED.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
    })
}

#[derive(Debug)]
struct Quiet;

impl Platform for Quiet {
    fn pick_folder(&self, _: &str, _: Option<&Path>, _: PickedFolder) {}
    fn pick_file(&self, _: &str, _: PickedFolder) {}
    fn pick_files(&self, _: &str, _: Option<FileFilter>, _: PickedFiles) {}
    fn open_path(&self, _: &Path) -> Result<(), String> {
        Ok(())
    }
    fn reveal_path(&self, _: &Path) -> Result<(), String> {
        Ok(())
    }
    fn notify(&self, _: &str, _: &str) {}
    fn editor_focused(&self) -> bool {
        true
    }
    fn raise_editor(&self) {}
    fn exit(&self, _: i32) {}
    fn versions(&self) -> Versions {
        Versions::default()
    }
    fn open_view_window(&self, _: &str, _: &str, _: Closed) -> Result<(), String> {
        Ok(())
    }
    fn close_view_window(&self, _: &str) {}
    fn open_terminal_window(&self, _: &str, _: &str, _: Closed) -> Result<(), String> {
        Ok(())
    }
    fn close_terminal_window(&self, _: &str) {}
}

#[derive(Debug, Default)]
struct Seen {
    servers: Mutex<Vec<McpServer>>,
    signed_in: AtomicBool,
    mode: Mutex<String>,
}

fn text(text: &str) -> ContentBlock {
    ContentBlock::Text(TextContent::new(text))
}

fn fake_agent(transport: Channel, seen: Arc<Seen>) {
    let on_new = seen.clone();
    let on_auth = seen.clone();
    let on_mode = seen;
    tokio::spawn(async move {
        let served = Agent
            .v1()
            .on_receive_request(
                async |request: InitializeRequest, responder, _cx| {
                    responder.respond(
                        InitializeResponse::new(request.protocol_version)
                            .auth_methods(vec![AuthMethod::Agent(AuthMethodAgent::new("log-in", "Log in"))]),
                    )
                },
                agent_client_protocol::on_receive_request!(),
            )
            .on_receive_request(
                async move |_request: AuthenticateRequest, responder, _cx| {
                    on_auth.signed_in.store(true, Ordering::Relaxed);
                    responder.respond(AuthenticateResponse::new())
                },
                agent_client_protocol::on_receive_request!(),
            )
            .on_receive_request(
                async move |request: NewSessionRequest, responder, _cx| {
                    if !on_new.signed_in.load(Ordering::Relaxed) {
                        return responder.respond_with_error(Error::auth_required());
                    }
                    on_new.servers.lock().unwrap().extend(request.mcp_servers);
                    let modes = SessionModeState::new(
                        "ask",
                        vec![SessionMode::new("ask", "Ask"), SessionMode::new("full", "Full")],
                    );
                    responder.respond(NewSessionResponse::new("fake-1").modes(modes))
                },
                agent_client_protocol::on_receive_request!(),
            )
            .on_receive_request(
                async move |request: SetSessionModeRequest, responder, _cx| {
                    *on_mode.mode.lock().unwrap() = request.mode_id.0.to_string();
                    responder.respond(SetSessionModeResponse::new())
                },
                agent_client_protocol::on_receive_request!(),
            )
            .on_receive_request(
                async |request: PromptRequest, responder, cx| {
                    let task_cx = cx.clone();
                    cx.spawn(async move {
                        let session = request.session_id.clone();
                        let say = |words: &str| {
                            task_cx.send_notification(SessionNotification::new(
                                session.clone(),
                                SessionUpdate::AgentMessageChunk(ContentChunk::new(text(words))),
                            ))
                        };
                        say("Hello")?;
                        let asked = task_cx
                            .send_request(RequestPermissionRequest::new(
                                session.clone(),
                                ToolCallUpdate::new("t1", ToolCallUpdateFields::new().title("write".to_owned())),
                                vec![
                                    PermissionOption::new("allow", "Allow once", PermissionOptionKind::AllowOnce),
                                    PermissionOption::new("reject", "Reject", PermissionOptionKind::RejectOnce),
                                ],
                            ))
                            .block_task()
                            .await?;
                        let allowed = matches!(
                            asked.outcome,
                            RequestPermissionOutcome::Selected(selected) if selected.option_id.0.as_ref() == "allow"
                        );
                        say(if allowed { " allowed" } else { " refused" })?;
                        responder.respond(PromptResponse::new(StopReason::EndTurn))
                    })
                },
                agent_client_protocol::on_receive_request!(),
            )
            .connect_to(transport)
            .await;
        if let Err(error) = served {
            log::debug!("the fake agent ended: {error}");
        }
    });
}

async fn until(what: &str, mut done: impl FnMut() -> bool) {
    let started = Instant::now();
    while !done() {
        assert!(started.elapsed() < WITHIN, "{what} didn't happen: {:?}", "");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

#[test]
fn a_panel_session_signs_in_asks_permission_switches_mode_and_records_its_events() {
    runtime().block_on(async {
        let temp = tempfile::tempdir().unwrap();
        let paths = AppPaths::new(&HostFolders {
            config: temp.path().join("config"),
            resources: temp.path().join("resources"),
            home: temp.path().to_path_buf(),
        })
        .unwrap();
        let app = Session::start(paths, Err(KeyError::NoKeyring), Arc::new(Quiet))
            .await
            .unwrap();
        let (client_end, agent_end) = Channel::duplex();
        let seen = Arc::new(Seen::default());
        fake_agent(agent_end, seen.clone());
        let context = Context {
            session: "panel-1".to_owned(),
            project: temp.path().join("project"),
            agent: "fake".to_owned(),
        };
        let (commands, receiver) = mpsc::unbounded_channel();
        let talking = tokio::spawn(converse(app.clone(), client_end, context, receiver));

        until("the sign-in ask", || {
            panel_state(&app).asks.iter().any(|ask| ask.id == SIGN_IN)
        })
        .await;
        assert_eq!(panel_state(&app).status, Status::SignIn);
        commands
            .send(Command::Answer {
                ask: SIGN_IN.to_owned(),
                option: "log-in".to_owned(),
            })
            .unwrap();
        until("the session", || panel_state(&app).status == Status::Ready).await;
        let state = panel_state(&app);
        assert_eq!(state.mode.as_deref(), Some("ask"));
        assert_eq!(state.modes.len(), 2);
        let servers = format!("{:?}", seen.servers.lock().unwrap());
        assert!(
            servers.contains("opsis-mcp") && servers.contains("panel-1"),
            "{servers}"
        );

        commands
            .send(Command::Prompt("Write a file".to_owned()))
            .unwrap();
        until("the permission ask", || {
            panel_state(&app).asks.iter().any(|ask| ask.id == "t1")
        })
        .await;
        assert!(
            app.state::<AppState>()
                .tasks
                .labels()
                .contains(&WAITING.to_owned())
        );
        commands
            .send(Command::Answer {
                ask: "t1".to_owned(),
                option: "allow".to_owned(),
            })
            .unwrap();
        until("the turn's end", || {
            panel_state(&app).status == Status::Ready
        })
        .await;
        assert_eq!(
            panel_state(&app).entries,
            [
                Entry::User("Write a file".to_owned()),
                Entry::Agent("Hello allowed".to_owned())
            ]
        );
        assert!(
            !app.state::<AppState>()
                .tasks
                .labels()
                .contains(&WAITING.to_owned())
        );

        commands.send(Command::SetMode("full".to_owned())).unwrap();
        until("the mode", || {
            panel_state(&app).mode.as_deref() == Some("full")
        })
        .await;
        assert_eq!(*seen.mode.lock().unwrap(), "full");

        commands.send(Command::Stop).unwrap();
        assert!(talking.await.unwrap().unwrap());
        let db = app.state::<AppState>().db();
        let mut messages = Vec::new();
        let started = Instant::now();
        while messages.len() < 2 && started.elapsed() < WITHIN {
            messages = db.assistant_messages("panel-1").await.unwrap();
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        let said: Vec<(String, String)> = messages.into_iter().map(|m| (m.role, m.text)).collect();
        assert_eq!(
            said,
            [
                ("user".to_owned(), "Write a file".to_owned()),
                ("agent".to_owned(), "Hello allowed".to_owned())
            ]
        );
        let requests = db.assistant_requests("panel-1").await.unwrap();
        assert!(
            requests
                .iter()
                .any(|r| r.request == "t1" && r.answer.as_deref() == Some("allow"))
        );
    });
}
