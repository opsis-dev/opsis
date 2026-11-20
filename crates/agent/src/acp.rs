use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use agent_client_protocol::v1::ConnectionTo;
use agent_client_protocol::v1::schema::{
    AgentCapabilities, AuthMethod, AuthMethodAgent, AuthMethodTerminal, AuthenticateRequest,
    AuthenticateResponse, CancelNotification, ContentBlock, ContentChunk, CurrentModeUpdate,
    ImageContent, InitializeRequest, InitializeResponse, LoadSessionRequest, LoadSessionResponse,
    McpServer, NewSessionRequest, NewSessionResponse, PermissionOption, PermissionOptionKind,
    PromptCapabilities, PromptRequest, PromptResponse, RequestPermissionOutcome,
    RequestPermissionRequest, SessionMode, SessionModeState, SessionNotification, SessionUpdate,
    SetSessionModeRequest, SetSessionModeResponse, StopReason, TextContent, ToolCall,
    ToolCallContent, ToolCallStatus, ToolCallUpdate, ToolCallUpdateFields, ToolKind,
};
use agent_client_protocol::{Agent, Client, Error, Stdio};
use opsis_persistence::keys::{KeyError, Keyring};
use rig_core::DynModel;
use rig_core::message::{AssistantContent, Message, ToolResultContent, UserContent};
use rig_core::operation::Completion;
use serde_json::Value;
use sqlx::SqlitePool;
use tokio_util::sync::CancellationToken;

use crate::mcp::{StdioServer, ToolServers};
use crate::turn::{Decision, Ending, Mode, Peer, Turn, Update};
use crate::{model, sessions, settings, tools};

pub const LOGIN: &str =
    "Log in first: run opsis-agent login to choose a model service, a model and its key.";
const LOGIN_METHOD: &str = "login";
const ALLOW: &str = "allow";
const ALLOW_ALWAYS: &str = "allow-always";
const REJECT: &str = "reject";

struct Live {
    folder: PathBuf,
    model: DynModel<Completion>,
    servers: ToolServers,
    history: tokio::sync::Mutex<Vec<Message>>,
    cancel: Mutex<CancellationToken>,
    mode: Mutex<Mode>,
    allowed: Arc<Mutex<HashSet<String>>>,
    waiting: Arc<Mutex<Vec<String>>>,
}

impl std::fmt::Debug for Live {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Live")
            .field("folder", &self.folder)
            .field("model", &self.model.name())
            .field("servers", &self.servers)
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub struct OpsisAgent {
    pool: SqlitePool,
    keys: Result<Keyring, KeyError>,
    sessions: Mutex<HashMap<String, Arc<Live>>>,
}

fn stdio_servers(servers: &[McpServer]) -> Vec<StdioServer> {
    servers
        .iter()
        .filter_map(|server| match server {
            McpServer::Stdio(stdio) => Some(StdioServer {
                name: stdio.name.clone(),
                command: stdio.command.clone(),
                args: stdio.args.clone(),
                env: stdio
                    .env
                    .iter()
                    .map(|v| (v.name.clone(), v.value.clone()))
                    .collect(),
            }),
            _ => None,
        })
        .collect()
}

fn system_prompt(folder: &std::path::Path, servers: &ToolServers) -> String {
    let mut prompt = format!(
        "You are Opsis's assistant, a coding agent working in {}. Opsis turns recorded websites and terminals into videos. Use the tools to read and change files and the project. Be brief.",
        folder.display()
    );
    for instructions in servers.instructions() {
        prompt.push_str("\n\n");
        prompt.push_str(&instructions);
    }
    prompt
}

fn user_message(prompt: &[ContentBlock]) -> Message {
    let content: Vec<UserContent> = prompt
        .iter()
        .filter_map(|block| match block {
            ContentBlock::Text(text) => Some(UserContent::text(text.text.clone())),
            ContentBlock::Image(image) => Some(UserContent::Image(rig_core::message::Image {
                data: rig_core::message::DocumentSourceKind::Base64(image.data.clone()),
                ..rig_core::message::Image::default()
            })),
            ContentBlock::ResourceLink(link) => Some(UserContent::text(link.uri.clone())),
            _ => None,
        })
        .collect();
    Message::User { content }
}

fn with_background(message: Message, ended: &[String]) -> Message {
    if ended.is_empty() {
        return message;
    }
    let Message::User { mut content } = message else {
        return message;
    };
    content.insert(0, UserContent::text(ended.join("\n")));
    Message::User { content }
}

fn text_block(text: impl Into<String>) -> ContentBlock {
    ContentBlock::Text(TextContent::new(text))
}

fn shown_output(output: &[ToolResultContent]) -> Vec<ToolCallContent> {
    output
        .iter()
        .map(|content| match tools::image_mime(content) {
            Some((data, mime)) => {
                ToolCallContent::from(ContentBlock::Image(ImageContent::new(data, mime)))
            }
            None => ToolCallContent::from(text_block(content.to_text("[picture]"))),
        })
        .collect()
}

fn kind_of(name: &str) -> ToolKind {
    match name {
        tools::READ => ToolKind::Read,
        tools::WRITE | tools::EDIT => ToolKind::Edit,
        tools::GLOB | tools::GREP => ToolKind::Search,
        _ => ToolKind::Other,
    }
}

#[derive(Debug, Clone)]
struct AcpPeer {
    cx: ConnectionTo<Client>,
    session: String,
    pool: SqlitePool,
    allowed: Arc<Mutex<HashSet<String>>>,
    waiting: Arc<Mutex<Vec<String>>>,
}

impl AcpPeer {
    fn send(&self, update: SessionUpdate) {
        if let Err(error) = self
            .cx
            .send_notification(SessionNotification::new(self.session.clone(), update))
        {
            log::warn!("sending a session update: {error}");
        }
    }
}

impl Peer for AcpPeer {
    fn update(&self, update: Update) {
        let update = match update {
            Update::Text(text) => {
                SessionUpdate::AgentMessageChunk(ContentChunk::new(text_block(text)))
            }
            Update::ToolStarted {
                id, title, input, ..
            } => SessionUpdate::ToolCall(
                ToolCall::new(id, title.clone())
                    .kind(kind_of(&title))
                    .status(ToolCallStatus::InProgress)
                    .raw_input(input),
            ),
            Update::ToolEnded { id, ok, output } => {
                SessionUpdate::ToolCallUpdate(ToolCallUpdate::new(
                    id,
                    ToolCallUpdateFields::new()
                        .status(if ok {
                            ToolCallStatus::Completed
                        } else {
                            ToolCallStatus::Failed
                        })
                        .content(shown_output(&output)),
                ))
            }
        };
        self.send(update);
    }

    async fn ask(&self, id: &str, title: &str, input: &Value) -> Decision {
        let options = vec![
            PermissionOption::new(ALLOW, "Allow once", PermissionOptionKind::AllowOnce),
            PermissionOption::new(
                ALLOW_ALWAYS,
                "Allow always",
                PermissionOptionKind::AllowAlways,
            ),
            PermissionOption::new(REJECT, "Reject", PermissionOptionKind::RejectOnce),
        ];
        let call = ToolCallUpdate::new(
            id.to_owned(),
            ToolCallUpdateFields::new()
                .title(title.to_owned())
                .raw_input(input.clone()),
        );
        let asked = self
            .cx
            .send_request(RequestPermissionRequest::new(
                self.session.clone(),
                call,
                options,
            ))
            .block_task()
            .await;
        match asked {
            Ok(answer) => match answer.outcome {
                RequestPermissionOutcome::Selected(selected) => match selected.option_id.0.as_ref()
                {
                    ALLOW => Decision::Once,
                    ALLOW_ALWAYS => Decision::Always,
                    _ => Decision::Reject,
                },
                _ => Decision::Reject,
            },
            Err(error) => {
                log::warn!("asking permission: {error}");
                Decision::Reject
            }
        }
    }

    fn allow_always(&self, allowance: &str) {
        self.allowed
            .lock()
            .expect("allowed")
            .insert(allowance.to_owned());
        let (pool, session, allowance) = (
            self.pool.clone(),
            self.session.clone(),
            allowance.to_owned(),
        );
        tokio::spawn(async move {
            if let Err(error) = sessions::allow(&pool, &session, &allowance).await {
                log::warn!("remembering what is allowed: {error}");
            }
        });
    }

    fn background_ended(&self, command: &str, report: String) {
        let message = format!("The background command `{command}` ended. {report}");
        self.send(SessionUpdate::AgentMessageChunk(ContentChunk::new(
            text_block(format!("\n{message}\n")),
        )));
        self.waiting.lock().expect("waiting").push(message);
    }
}

fn modes(current: Mode) -> SessionModeState {
    SessionModeState::new(
        current.id(),
        Mode::ALL
            .into_iter()
            .map(|mode| SessionMode::new(mode.id(), mode.name()).description(mode.description()))
            .collect(),
    )
}

impl OpsisAgent {
    pub fn new(pool: SqlitePool, keys: Result<Keyring, KeyError>) -> Self {
        Self {
            pool,
            keys,
            sessions: Mutex::default(),
        }
    }

    async fn model(&self) -> Result<DynModel<Completion>, Error> {
        let chosen = settings::read(&self.pool)
            .await
            .map_err(Error::into_internal_error)?
            .ok_or_else(|| Error::auth_required().data(Value::String(LOGIN.to_owned())))?;
        let key = if chosen.service.needs_key() {
            let keys = self.keys.clone().map_err(Error::into_internal_error)?;
            let name = chosen.service.key_name();
            let key = tokio::task::spawn_blocking(move || keys.key(&name))
                .await
                .map_err(Error::into_internal_error)?
                .map_err(Error::into_internal_error)?
                .ok_or_else(|| Error::auth_required().data(Value::String(LOGIN.to_owned())))?;
            Some(key)
        } else {
            None
        };
        Ok(model::connect(&chosen, key.as_deref()))
    }

    async fn open(
        &self,
        id: &str,
        folder: PathBuf,
        servers: &[McpServer],
        history: Vec<Message>,
        mode: Mode,
    ) -> Result<(), Error> {
        let allowed: HashSet<String> = sessions::allowed(&self.pool, id)
            .await
            .map_err(Error::into_internal_error)?
            .into_iter()
            .collect();
        let model = self.model().await?;
        let servers = ToolServers::start(&stdio_servers(servers), &folder).await;
        let live = Arc::new(Live {
            folder,
            model,
            servers,
            history: tokio::sync::Mutex::new(history),
            cancel: Mutex::new(CancellationToken::new()),
            mode: Mutex::new(mode),
            allowed: Arc::new(Mutex::new(allowed)),
            waiting: Arc::default(),
        });
        self.sessions
            .lock()
            .expect("sessions")
            .insert(id.to_owned(), live);
        Ok(())
    }

    fn live(&self, id: &str) -> Option<Arc<Live>> {
        self.sessions.lock().expect("sessions").get(id).cloned()
    }

    fn peer(&self, cx: ConnectionTo<Client>, session: &str, live: Option<&Live>) -> AcpPeer {
        AcpPeer {
            cx,
            session: session.to_owned(),
            pool: self.pool.clone(),
            allowed: live.map_or_default(|l| l.allowed.clone()),
            waiting: live.map_or_default(|l| l.waiting.clone()),
        }
    }

    async fn set_mode(
        &self,
        request: SetSessionModeRequest,
        cx: ConnectionTo<Client>,
    ) -> Result<(), Error> {
        let session = request.session_id.0.to_string();
        let live = self.live(&session).ok_or_else(Error::invalid_params)?;
        let mode = Mode::from_id(&request.mode_id.0).ok_or_else(Error::invalid_params)?;
        *live.mode.lock().expect("mode") = mode;
        sessions::set_mode(&self.pool, &session, mode.id())
            .await
            .map_err(Error::into_internal_error)?;
        self.peer(cx, &session, Some(&live))
            .send(SessionUpdate::CurrentModeUpdate(CurrentModeUpdate::new(
                mode.id(),
            )));
        Ok(())
    }

    fn replay(peer: &AcpPeer, history: &[Message]) {
        for message in history {
            match message {
                Message::User { content } => {
                    for block in content {
                        if let UserContent::Text(text) = block {
                            peer.send(SessionUpdate::UserMessageChunk(ContentChunk::new(
                                text_block(text.text.clone()),
                            )));
                        }
                    }
                }
                Message::Assistant(turn) => {
                    for block in &turn.content {
                        if let AssistantContent::Text(text) = block {
                            peer.send(SessionUpdate::AgentMessageChunk(ContentChunk::new(
                                text_block(text.text.clone()),
                            )));
                        }
                    }
                }
                Message::System { .. } => {}
            }
        }
    }

    async fn prompt(
        self: Arc<Self>,
        request: PromptRequest,
        cx: ConnectionTo<Client>,
    ) -> Result<StopReason, Error> {
        let session = request.session_id.0.to_string();
        let live = self.live(&session).ok_or_else(Error::invalid_params)?;
        let cancel = CancellationToken::new();
        *live.cancel.lock().expect("cancel") = cancel.clone();
        let mut history = live.history.lock().await;
        let ended_meanwhile: Vec<String> =
            std::mem::take(&mut *live.waiting.lock().expect("waiting"));
        let message = with_background(user_message(&request.prompt), &ended_meanwhile);
        sessions::append(&self.pool, &session, &message)
            .await
            .map_err(Error::into_internal_error)?;
        history.push(message);
        let peer = self.peer(cx, &session, Some(&live));
        let turn = Turn {
            folder: live.folder.clone(),
            system: system_prompt(&live.folder, &live.servers),
            model: &live.model,
            servers: &live.servers,
            cancel,
            mode: *live.mode.lock().expect("mode"),
            allowed: Mutex::new(live.allowed.lock().expect("allowed").clone()),
        };
        let mut unsaved = Vec::new();
        let ended = turn
            .run(&mut history, &peer, |message| unsaved.push(message.clone()))
            .await;
        for message in &unsaved {
            if let Err(error) = sessions::append(&self.pool, &session, message).await {
                log::warn!("saving the session: {error}");
            }
        }
        match ended {
            Ok(Ending::Done) => Ok(StopReason::EndTurn),
            Ok(Ending::Cancelled) => Ok(StopReason::Cancelled),
            Ok(Ending::Refused) => Ok(StopReason::Refusal),
            Err(error) => {
                peer.send(SessionUpdate::AgentMessageChunk(ContentChunk::new(
                    text_block(error.to_string()),
                )));
                Ok(StopReason::EndTurn)
            }
        }
    }
}

fn login_method(terminal: bool) -> AuthMethod {
    if terminal {
        AuthMethod::Terminal(
            AuthMethodTerminal::new(LOGIN_METHOD, "Log in")
                .args(vec![LOGIN_METHOD.to_owned()])
                .description(LOGIN),
        )
    } else {
        AuthMethod::Agent(AuthMethodAgent::new(LOGIN_METHOD, "Log in").description(LOGIN))
    }
}

pub async fn serve(agent: OpsisAgent) -> Result<(), Error> {
    let agent = Arc::new(agent);
    let on_new = agent.clone();
    let on_load = agent.clone();
    let on_prompt = agent.clone();
    let on_cancel = agent.clone();
    let on_auth = agent.clone();
    let on_mode = agent.clone();
    Agent
        .v1()
        .name("opsis-agent")
        .on_receive_request(
            async move |request: InitializeRequest, responder, _cx| {
                responder.respond(
                    InitializeResponse::new(request.protocol_version)
                        .agent_capabilities(
                            AgentCapabilities::new()
                                .load_session(true)
                                .prompt_capabilities(PromptCapabilities::new().image(true)),
                        )
                        .auth_methods(vec![login_method(
                            request.client_capabilities.auth.terminal,
                        )]),
                )
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_request(
            async move |_request: AuthenticateRequest, responder, _cx| match on_auth.model().await {
                Ok(_) => responder.respond(AuthenticateResponse::new()),
                Err(error) => responder.respond_with_error(error),
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_request(
            async move |request: NewSessionRequest, responder, _cx| {
                let id = sessions::new_id();
                let created = async {
                    on_new
                        .open(
                            &id,
                            request.cwd.clone(),
                            &request.mcp_servers,
                            Vec::new(),
                            Mode::Ask,
                        )
                        .await?;
                    sessions::create(&on_new.pool, &id, &request.cwd, Mode::Ask.id())
                        .await
                        .map_err(Error::into_internal_error)
                };
                match created.await {
                    Ok(()) => {
                        responder.respond(NewSessionResponse::new(id).modes(modes(Mode::Ask)))
                    }
                    Err(error) => responder.respond_with_error(error),
                }
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_request(
            async move |request: LoadSessionRequest, responder, cx| {
                let id = request.session_id.0.to_string();
                let loaded = async {
                    let history = sessions::messages(&on_load.pool, &id)
                        .await
                        .map_err(Error::into_internal_error)?;
                    let stored = sessions::find(&on_load.pool, &id)
                        .await
                        .map_err(Error::into_internal_error)?
                        .ok_or_else(Error::invalid_params)?;
                    let mode = Mode::from_id(&stored.mode).unwrap_or(Mode::Ask);
                    OpsisAgent::replay(&on_load.peer(cx, &id, None), &history);
                    on_load
                        .open(
                            &id,
                            request.cwd.clone(),
                            &request.mcp_servers,
                            history,
                            mode,
                        )
                        .await?;
                    Ok::<_, Error>(mode)
                };
                match loaded.await {
                    Ok(mode) => responder.respond(LoadSessionResponse::new().modes(modes(mode))),
                    Err(error) => responder.respond_with_error(error),
                }
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_request(
            async move |request: PromptRequest, responder, cx| {
                let agent = on_prompt.clone();
                let task_cx = cx.clone();
                cx.spawn(async move {
                    match agent.prompt(request, task_cx).await {
                        Ok(stop) => responder.respond(PromptResponse::new(stop)),
                        Err(error) => responder.respond_with_error(error),
                    }
                })
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_request(
            async move |request: SetSessionModeRequest, responder, cx| match on_mode
                .set_mode(request, cx)
                .await
            {
                Ok(()) => responder.respond(SetSessionModeResponse::new()),
                Err(error) => responder.respond_with_error(error),
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_notification(
            async move |notification: CancelNotification, _cx| {
                if let Some(live) = on_cancel.live(&notification.session_id.0) {
                    live.cancel.lock().expect("cancel").cancel();
                }
                Ok(())
            },
            agent_client_protocol::on_receive_notification!(),
        )
        .connect_to(Stdio::new())
        .await
}
