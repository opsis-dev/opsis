use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

use agent_client_protocol::schema::ProtocolVersion;
use agent_client_protocol::util::MatchDispatch;
use agent_client_protocol::v1::schema::{
    AuthMethod, AuthenticateRequest, CancelNotification, ContentBlock, ContentChunk,
    InitializeRequest, McpServer, McpServerStdio, NewSessionRequest, PermissionOptionKind,
    RequestPermissionOutcome, RequestPermissionRequest, RequestPermissionResponse,
    SelectedPermissionOutcome, SessionUpdate, SetSessionModeRequest, StopReason, ToolCallStatus,
    ToolKind,
};
use agent_client_protocol::v1::{ActiveSession, SessionMessage, SessionSetup};
use agent_client_protocol::{AcpAgent, AcpAgentConfig, Agent, Client, ConnectTo, Error, Responder};
use opsis_persistence::logs::{RequestLog, assistant_logs};
use opsis_store::assistant::{AssistantEvent, AssistantThread, Recorded};
use serde_json::{Value, json};
use tokio::sync::mpsc;

use crate::Session;
use crate::agents::{AgentChoice, BUNDLED_ID, available};
use crate::events::{AssistantChanged, Emit};
use crate::state::AppState;

pub const WAITING: &str = "Waiting for the agent";
pub const SIGN_IN: &str = "sign-in";
pub const TURN_OFF: &str =
    "Turn on the assistant in Settings → Assistant first, so the agent can reach the project.";
const RESTARTS: u32 = 3;
const RESTART_AFTER: Duration = Duration::from_secs(1);

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Status {
    #[default]
    Off,
    Starting,
    SignIn,
    Ready,
    Working,
    Ended(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entry {
    User(String),
    Agent(String),
    Tool {
        id: String,
        kind: String,
        title: String,
        input: Option<Value>,
        status: String,
        output: String,
    },
    Notice(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AskOption {
    pub id: String,
    pub name: String,
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ask {
    pub id: String,
    pub title: String,
    pub input: Option<Value>,
    pub sign_in: bool,
    pub options: Vec<AskOption>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModeChoice {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PanelState {
    pub agent: Option<String>,
    pub agent_name: String,
    pub status: Status,
    pub session: Option<String>,
    pub title: String,
    pub working_since: Option<i64>,
    pub agent_session: Option<String>,
    pub mode: Option<String>,
    pub modes: Vec<ModeChoice>,
    pub entries: Vec<Entry>,
    pub asks: Vec<Ask>,
}

#[derive(Debug)]
pub enum Command {
    Prompt(String),
    Cancel,
    SetMode(String),
    Answer { ask: String, option: String },
    Stop,
}

#[derive(Debug, Default)]
pub struct Assistant {
    state: Mutex<PanelState>,
    commands: Mutex<Option<mpsc::UnboundedSender<Command>>>,
    agents: Mutex<Vec<AgentChoice>>,
    open: Mutex<Option<String>>,
    expanded: Mutex<HashSet<String>>,
}

pub const NEW_THREAD: &str = "New thread";

pub fn chosen_agent(app: &Session) -> String {
    let chosen = app.state::<AppState>().settings().assistant.agent;
    if chosen.is_empty() {
        BUNDLED_ID.to_owned()
    } else {
        chosen
    }
}

pub fn open_thread_id(app: &Session) -> Option<String> {
    app.state::<Assistant>().open.lock().expect("open").clone()
}

pub fn open_thread(app: &Session, thread: Option<&str>) {
    *app.state::<Assistant>().open.lock().expect("open") = thread.map(str::to_owned);
    changed(app);
}

pub fn is_expanded(app: &Session, id: &str) -> bool {
    app.state::<Assistant>()
        .expanded
        .lock()
        .expect("expanded")
        .contains(id)
}

pub fn toggle(app: &Session, id: &str) {
    {
        let mut expanded = app.state::<Assistant>().expanded.lock().expect("expanded");
        if !expanded.remove(id) {
            expanded.insert(id.to_owned());
        }
    }
    changed(app);
}

pub async fn new_thread(app: &Session) -> Result<(), String> {
    start(app, &chosen_agent(app)).await?;
    let session = panel_state(app).session;
    open_thread(app, session.as_deref());
    Ok(())
}

pub async fn threads(app: &Session) -> Vec<AssistantThread> {
    let Some(project) = crate::document::document_dir(app.state::<AppState>()) else {
        return Vec::new();
    };
    app.state::<AppState>()
        .db()
        .assistant_threads(&project.to_string_lossy())
        .await
        .unwrap_or_else(|error| {
            log::warn!("listing the assistant's threads: {error}");
            Vec::new()
        })
}

pub async fn past_entries(app: &Session, thread: &str) -> Vec<Entry> {
    let messages = app
        .state::<AppState>()
        .db()
        .assistant_messages(thread)
        .await
        .unwrap_or_else(|error| {
            log::warn!("reading an assistant thread: {error}");
            Vec::new()
        });
    messages
        .into_iter()
        .map(|message| {
            if message.role == "user" {
                Entry::User(message.text)
            } else {
                Entry::Agent(message.text)
            }
        })
        .collect()
}

fn first_line(text: &str) -> &str {
    text.lines().next().unwrap_or_default().trim()
}

pub fn agents(app: &Session) -> Vec<AgentChoice> {
    app.state::<Assistant>()
        .agents
        .lock()
        .expect("agents")
        .clone()
}

pub fn refresh_agents(app: &Session) {
    let app = app.clone();
    crate::runtime::spawn(async move {
        let found = available(&app).await;
        *app.state::<Assistant>().agents.lock().expect("agents") = found;
        changed(&app);
    });
}

pub fn open_logs(app: &Session) -> Result<(), String> {
    let logs = assistant_logs(&app.state::<AppState>().paths.user_data);
    std::fs::create_dir_all(&logs).map_err(|e| e.to_string())?;
    app.platform().open_path(&logs)
}

fn changed(app: &Session) {
    if let Err(error) = AssistantChanged.emit(app) {
        log::debug!("no window follows the assistant: {error}");
    }
}

pub fn panel_state(app: &Session) -> PanelState {
    app.state::<Assistant>()
        .state
        .lock()
        .expect("assistant")
        .clone()
}

fn edit(app: &Session, change: impl FnOnce(&mut PanelState)) {
    change(&mut app.state::<Assistant>().state.lock().expect("assistant"));
    changed(app);
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .and_then(|elapsed| i64::try_from(elapsed.as_millis()).ok())
        .unwrap_or_default()
}

#[derive(Debug, Clone)]
pub struct Context {
    pub session: String,
    pub project: PathBuf,
    pub agent: String,
}

fn record(app: &Session, context: &Context, event: AssistantEvent) {
    let recorded = Recorded {
        session: context.session.clone(),
        project: context.project.to_string_lossy().into_owned(),
        agent: context.agent.clone(),
        at: now_ms(),
        event,
    };
    let app = app.clone();
    crate::runtime::spawn(async move {
        if let Err(error) = app
            .state::<AppState>()
            .db()
            .record_assistant_event(&recorded)
            .await
        {
            log::warn!("recording an assistant event: {error}");
        }
    });
}

fn send(app: &Session, command: Command) -> bool {
    let commands = app.state::<Assistant>().commands.lock().expect("commands");
    commands
        .as_ref()
        .is_some_and(|sender| sender.send(command).is_ok())
}

pub fn prompt(app: &Session, text: &str) -> bool {
    let text = text.trim();
    !text.is_empty() && send(app, Command::Prompt(text.to_owned()))
}

pub fn cancel(app: &Session) -> bool {
    send(app, Command::Cancel)
}

pub fn set_mode(app: &Session, mode: &str) -> bool {
    send(app, Command::SetMode(mode.to_owned()))
}

pub fn answer(app: &Session, ask: &str, option: &str) -> bool {
    send(
        app,
        Command::Answer {
            ask: ask.to_owned(),
            option: option.to_owned(),
        },
    )
}

pub fn stop(app: &Session) {
    send(app, Command::Stop);
    *app.state::<Assistant>().commands.lock().expect("commands") = None;
    *app.state::<Assistant>().open.lock().expect("open") = None;
    app.state::<Assistant>()
        .expanded
        .lock()
        .expect("expanded")
        .clear();
    edit(app, |state| *state = PanelState::default());
}

pub fn opsis_mcp() -> PathBuf {
    let name = if cfg!(windows) {
        "opsis-mcp.exe"
    } else {
        "opsis-mcp"
    };
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join(name)))
        .unwrap_or_else(|| PathBuf::from(name))
}

pub fn tool_server(session: &str) -> McpServer {
    McpServer::Stdio(McpServerStdio::new("opsis", opsis_mcp()).args(vec![
        "stdio".to_owned(),
        "--session".to_owned(),
        session.to_owned(),
    ]))
}

fn text_of(content: &ContentBlock) -> Option<String> {
    match content {
        ContentBlock::Text(text) => Some(text.text.clone()),
        _ => None,
    }
}

const fn kind_name(kind: ToolKind) -> &'static str {
    match kind {
        ToolKind::Read => "read",
        ToolKind::Edit => "edit",
        ToolKind::Delete => "delete",
        ToolKind::Move => "move",
        ToolKind::Search => "search",
        ToolKind::Execute => "execute",
        ToolKind::Fetch => "fetch",
        _ => "other",
    }
}

const fn option_kind(kind: PermissionOptionKind) -> &'static str {
    match kind {
        PermissionOptionKind::AllowOnce => "allowOnce",
        PermissionOptionKind::AllowAlways => "allowAlways",
        PermissionOptionKind::RejectOnce => "rejectOnce",
        _ => "rejectAlways",
    }
}

const fn status_name(status: ToolCallStatus) -> &'static str {
    match status {
        ToolCallStatus::Pending => "pending",
        ToolCallStatus::Completed => "done",
        ToolCallStatus::Failed => "failed",
        _ => "running",
    }
}

fn apply_update(state: &mut PanelState, update: &SessionUpdate) -> Option<String> {
    match update {
        SessionUpdate::AgentMessageChunk(ContentChunk { content, .. }) => {
            let text = text_of(content)?;
            if let Some(Entry::Agent(open)) = state.entries.last_mut() {
                open.push_str(&text);
            } else {
                state.entries.push(Entry::Agent(text.clone()));
            }
            Some(text)
        }
        SessionUpdate::UserMessageChunk(ContentChunk { content, .. }) => {
            if let Some(text) = text_of(content) {
                state.entries.push(Entry::User(text));
            }
            None
        }
        SessionUpdate::ToolCall(call) => {
            state.entries.push(Entry::Tool {
                id: call.tool_call_id.0.to_string(),
                kind: kind_name(call.kind).to_owned(),
                title: call.title.clone(),
                input: call.raw_input.clone(),
                status: status_name(call.status).to_owned(),
                output: String::new(),
            });
            None
        }
        SessionUpdate::ToolCallUpdate(update) => {
            let id = update.tool_call_id.0.to_string();
            let found = state
                .entries
                .iter_mut()
                .rev()
                .find_map(|entry| match entry {
                    Entry::Tool {
                        id: tool,
                        status,
                        output,
                        title,
                        input,
                        ..
                    } if *tool == id => Some((status, output, title, input)),
                    _ => None,
                });
            if let Some((status, output, title, input)) = found {
                if let Some(new_input) = &update.fields.raw_input {
                    *input = Some(new_input.clone());
                }
                if let Some(new_status) = &update.fields.status {
                    status_name(*new_status).clone_into(status);
                }
                if let Some(new_title) = &update.fields.title {
                    new_title.clone_into(title);
                }
                if let Some(content) = &update.fields.content {
                    *output = content
                        .iter()
                        .filter_map(|block| serde_json::to_value(block).ok())
                        .filter_map(|block| {
                            block
                                .pointer("/content/text")
                                .and_then(Value::as_str)
                                .map(str::to_owned)
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                }
            }
            None
        }
        SessionUpdate::CurrentModeUpdate(mode) => {
            state.mode = Some(mode.current_mode_id.0.to_string());
            None
        }
        _ => None,
    }
}

enum Incoming {
    Update(Box<SessionUpdate>),
    Permission(
        Box<(
            RequestPermissionRequest,
            Responder<RequestPermissionResponse>,
        )>,
    ),
}

async fn incoming(
    message: SessionMessage,
) -> Result<Option<Result<Incoming, Option<StopReason>>>, Error> {
    match message {
        SessionMessage::Message(dispatch) => {
            let found = Mutex::new(None);
            MatchDispatch::new(dispatch.into_raw())
                .if_request(async |request: RequestPermissionRequest, responder| {
                    *found.lock().expect("incoming") =
                        Some(Incoming::Permission(Box::new((request, responder))));
                    Ok(())
                })
                .await
                .if_notification(
                    async |notification: agent_client_protocol::v1::schema::SessionNotification| {
                        *found.lock().expect("incoming") =
                            Some(Incoming::Update(Box::new(notification.update)));
                        Ok(())
                    },
                )
                .await
                .otherwise_ignore()?;
            Ok(found.into_inner().expect("incoming").map(Ok))
        }
        SessionMessage::PromptResponse(response) => {
            Ok(Some(Err(response.ok().map(|r| r.stop_reason))))
        }
        _ => Ok(None),
    }
}

const fn stop_name(stop: Option<&StopReason>) -> &'static str {
    match stop {
        Some(StopReason::EndTurn) => "endTurn",
        Some(StopReason::Cancelled) => "cancelled",
        Some(StopReason::Refusal) => "refusal",
        Some(StopReason::MaxTokens) => "maxTokens",
        Some(StopReason::MaxTurnRequests) => "maxTurnRequests",
        _ => "error",
    }
}

struct Talk<'a> {
    app: &'a Session,
    context: &'a Context,
    commands: &'a mut mpsc::UnboundedReceiver<Command>,
    pending: HashMap<String, Responder<RequestPermissionResponse>>,
    working: bool,
}

impl Talk<'_> {
    fn turn_ended(&mut self, stop: Option<&StopReason>) {
        if self.working {
            self.app.state::<AppState>().tasks.end(WAITING);
            self.working = false;
        }
        record(
            self.app,
            self.context,
            AssistantEvent::TurnEnded {
                stop: stop_name(stop).to_owned(),
            },
        );
        edit(self.app, |state| {
            state.status = Status::Ready;
            state.working_since = None;
            state.asks.retain(|ask| ask.sign_in);
        });
    }

    fn update(&self, update: &SessionUpdate) {
        let mut said = None;
        edit(self.app, |state| said = apply_update(state, update));
        if let Some(text) = said {
            record(self.app, self.context, AssistantEvent::AgentText { text });
        }
    }

    fn permission(
        &mut self,
        request: &RequestPermissionRequest,
        responder: Responder<RequestPermissionResponse>,
    ) {
        let id = request.tool_call.tool_call_id.0.to_string();
        let title = request
            .tool_call
            .fields
            .title
            .clone()
            .unwrap_or_else(|| "The agent asks to go on".to_owned());
        let input = request.tool_call.fields.raw_input.clone();
        let options = request
            .options
            .iter()
            .map(|option| AskOption {
                id: option.option_id.0.to_string(),
                name: option.name.clone(),
                kind: option_kind(option.kind).to_owned(),
            })
            .collect();
        record(
            self.app,
            self.context,
            AssistantEvent::RequestAsked {
                request: id.clone(),
                kind: "permission".to_owned(),
                title: title.clone(),
            },
        );
        edit(self.app, |state| {
            state.asks.push(Ask {
                id: id.clone(),
                title,
                input,
                sign_in: false,
                options,
            });
        });
        self.pending.insert(id, responder);
    }

    fn answered(&mut self, ask: &str, option: &str) {
        let Some(responder) = self.pending.remove(ask) else {
            return;
        };
        let outcome =
            RequestPermissionOutcome::Selected(SelectedPermissionOutcome::new(option.to_owned()));
        if let Err(error) = responder.respond(RequestPermissionResponse::new(outcome)) {
            log::warn!("answering the agent: {error}");
        }
        record(
            self.app,
            self.context,
            AssistantEvent::RequestAnswered {
                request: ask.to_owned(),
                answer: option.to_owned(),
            },
        );
        edit(self.app, |state| state.asks.retain(|a| a.id != ask));
    }

    async fn run(&mut self, session: &mut ActiveSession<'_, Agent>) -> Result<bool, Error> {
        loop {
            tokio::select! {
                command = self.commands.recv() => match command {
                    Some(Command::Prompt(text)) => {
                        record(self.app, self.context, AssistantEvent::UserMessage { text: text.clone() });
                        record(self.app, self.context, AssistantEvent::TurnStarted);
                        edit(self.app, |state| {
                            if state.title.is_empty() {
                                first_line(&text).clone_into(&mut state.title);
                            }
                            state.entries.push(Entry::User(text.clone()));
                            state.status = Status::Working;
                            state.working_since = Some(now_ms());
                        });
                        if !self.working {
                            self.app.state::<AppState>().tasks.begin(WAITING);
                            self.working = true;
                        }
                        session.send_prompt(text)?;
                    }
                    Some(Command::Cancel) => {
                        let id = session.session_id().clone();
                        session.connection().send_notification(CancelNotification::new(id))?;
                    }
                    Some(Command::SetMode(mode)) => {
                        let id = session.session_id().clone();
                        session
                            .connection()
                            .send_request(SetSessionModeRequest::new(id, mode.clone()))
                            .block_task()
                            .await?;
                        edit(self.app, |state| state.mode = Some(mode));
                    }
                    Some(Command::Answer { ask, option }) => self.answered(&ask, &option),
                    Some(Command::Stop) | None => return Ok(true),
                },
                message = session.read_update() => match incoming(message?).await? {
                    Some(Ok(Incoming::Update(update))) => self.update(&update),
                    Some(Ok(Incoming::Permission(asked))) => {
                        let (request, responder) = *asked;
                        self.permission(&request, responder);
                    }
                    Some(Err(stop)) => self.turn_ended(stop.as_ref()),
                    None => {}
                },
            }
        }
    }
}

fn sign_in_methods(methods: &[AuthMethod]) -> Vec<AskOption> {
    methods
        .iter()
        .filter_map(|method| match method {
            AuthMethod::Agent(agent) => Some(AskOption {
                id: agent.id.0.to_string(),
                name: agent.name.clone(),
                kind: SIGN_IN.to_owned(),
            }),
            _ => None,
        })
        .collect()
}

async fn sign_in(
    app: &Session,
    commands: &mut mpsc::UnboundedReceiver<Command>,
    methods: &[AskOption],
) -> Option<String> {
    edit(app, |state| {
        state.status = Status::SignIn;
        state.asks.push(Ask {
            id: SIGN_IN.to_owned(),
            title: "Sign in to continue".to_owned(),
            input: None,
            sign_in: true,
            options: methods.to_vec(),
        });
    });
    loop {
        match commands.recv().await? {
            Command::Answer { ask, option } if ask == SIGN_IN => {
                edit(app, |state| state.asks.retain(|a| a.id != SIGN_IN));
                return Some(option);
            }
            Command::Stop => return None,
            _ => {}
        }
    }
}

pub async fn converse(
    app: Session,
    transport: impl ConnectTo<Client> + 'static,
    context: Context,
    mut commands: mpsc::UnboundedReceiver<Command>,
) -> Result<bool, Error> {
    let request =
        NewSessionRequest::new(&context.project).mcp_servers(vec![tool_server(&context.session)]);
    Client
        .v1()
        .name("opsis")
        .connect_with(transport, async |cx| {
            let hello = cx
                .send_request(InitializeRequest::new(ProtocolVersion::V1))
                .block_task()
                .await?;
            let methods = sign_in_methods(&hello.auth_methods);
            loop {
                let started = cx
                    .build_session_from(request.clone())
                    .block_task()
                    .run_until(
                        async |SessionSetup {
                                   mut session,
                                   response,
                                   ..
                               }| {
                            let agent_session = session.session_id().0.to_string();
                            record(
                                &app,
                                &context,
                                AssistantEvent::SessionStarted {
                                    agent_session: Some(agent_session.clone()),
                                },
                            );
                            edit(&app, |state| {
                                state.status = Status::Ready;
                                state.agent_session = Some(agent_session);
                                if let Some(modes) = &response.modes {
                                    state.mode = Some(modes.current_mode_id.0.to_string());
                                    state.modes = modes
                                        .available_modes
                                        .iter()
                                        .map(|mode| ModeChoice {
                                            id: mode.id.0.to_string(),
                                            name: mode.name.clone(),
                                        })
                                        .collect();
                                }
                            });
                            let mut talk = Talk {
                                app: &app,
                                context: &context,
                                commands: &mut commands,
                                pending: HashMap::new(),
                                working: false,
                            };
                            let ended = talk.run(&mut session).await;
                            if talk.working {
                                app.state::<AppState>().tasks.end(WAITING);
                            }
                            ended
                        },
                    )
                    .await;
                match started {
                    Err(error)
                        if error.code == Error::auth_required().code && !methods.is_empty() =>
                    {
                        let Some(method) = sign_in(&app, &mut commands, &methods).await else {
                            return Ok(true);
                        };
                        record(
                            &app,
                            &context,
                            AssistantEvent::RequestAnswered {
                                request: SIGN_IN.to_owned(),
                                answer: method.clone(),
                            },
                        );
                        cx.send_request(AuthenticateRequest::new(method))
                            .block_task()
                            .await?;
                    }
                    other => return other,
                }
            }
        })
        .await
}

fn log_line(log: &RequestLog, line: &str, direction: agent_client_protocol::LineDirection) {
    let direction = match direction {
        agent_client_protocol::LineDirection::Stdin => "toAgent",
        agent_client_protocol::LineDirection::Stdout => "fromAgent",
        agent_client_protocol::LineDirection::Stderr => "stderr",
    };
    let message =
        serde_json::from_str::<Value>(line).unwrap_or_else(|_| Value::String(line.to_owned()));
    if let Err(error) = log.write(&json!({ "direction": direction, "message": message })) {
        log::debug!("writing the panel's request log: {error}");
    }
}

fn process(choice: &AgentChoice, user_data: &Path) -> AcpAgent {
    let log = RequestLog::new(
        &assistant_logs(user_data),
        &format!("panel-{}", choice.id.replace([':', '/', '\\'], "-")),
    );
    AcpAgent::new(AcpAgentConfig::new(&choice.command).args(choice.args.clone()))
        .with_debug(move |line, direction| log_line(&log, line, direction))
}

pub async fn start(app: &Session, agent: &str) -> Result<(), String> {
    if !app.state::<AppState>().settings().assistant.enabled {
        return Err(TURN_OFF.to_owned());
    }
    let project = crate::document::document_dir(app.state::<AppState>())
        .ok_or_else(|| "Open a project first.".to_owned())?;
    let choice = available(app)
        .await
        .into_iter()
        .find(|choice| choice.id == agent)
        .ok_or_else(|| "That agent isn't on this computer any more.".to_owned())?;
    stop(app);
    let context = Context {
        session: uuid::Uuid::new_v4().simple().to_string(),
        project,
        agent: choice.id.clone(),
    };
    edit(app, |state| {
        state.agent = Some(choice.id.clone());
        state.agent_name.clone_from(&choice.name);
        state.status = Status::Starting;
        state.session = Some(context.session.clone());
    });
    let app = app.clone();
    crate::runtime::spawn(async move {
        keep_talking(&app, &choice, &context).await;
    });
    Ok(())
}

async fn keep_talking(app: &Session, choice: &AgentChoice, context: &Context) {
    let user_data = app.state::<AppState>().paths.user_data.clone();
    for attempt in 0..=RESTARTS {
        let (sender, receiver) = mpsc::unbounded_channel();
        *app.state::<Assistant>().commands.lock().expect("commands") = Some(sender);
        let ended = converse(
            app.clone(),
            process(choice, &user_data),
            context.clone(),
            receiver,
        )
        .await;
        let stopped = panel_state(app).session.as_deref() != Some(context.session.as_str());
        match ended {
            Ok(true) => return,
            _ if stopped => return,
            Ok(false) => {}
            Err(error) => {
                let message = format!("The agent stopped: {error}");
                edit(app, |state| {
                    state.entries.push(Entry::Notice(message.clone()));
                    state.status = Status::Ended(message.clone());
                });
            }
        }
        if attempt == RESTARTS {
            return;
        }
        tokio::time::sleep(RESTART_AFTER).await;
        edit(app, |state| {
            state
                .entries
                .push(Entry::Notice("Starting the agent again.".to_owned()));
            state.status = Status::Starting;
        });
    }
}
