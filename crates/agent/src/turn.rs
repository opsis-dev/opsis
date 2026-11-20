use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Mutex;

use futures::StreamExt;
use rig_core::DynModel;
use rig_core::completion::CompletionRequest;
use rig_core::message::{AssistantContent, Message, ToolCall, ToolResult, ToolResultContent};
use rig_core::operation::Completion;
use rig_core::streaming::StreamEvent;
use serde_json::Value;
use tokio_util::sync::CancellationToken;

use crate::mcp::ToolServers;
use crate::{shell, tools};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Ask,
    Full,
}

impl Mode {
    pub const ALL: [Self; 2] = [Self::Ask, Self::Full];

    pub const fn id(self) -> &'static str {
        match self {
            Self::Ask => "ask",
            Self::Full => "full",
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Ask => "Ask",
            Self::Full => "Full",
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            Self::Ask => {
                "Reads, searches and read-only commands run at once; everything else asks first."
            }
            Self::Full => "Nothing asks first.",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|mode| mode.id() == id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Once,
    Always,
    Reject,
}

pub fn allowance(name: &str, input: &Value) -> String {
    if name == shell::SHELL {
        let program = input
            .get("command")
            .and_then(Value::as_str)
            .and_then(|line| line.split_whitespace().next())
            .unwrap_or_default();
        return format!("{}:{program}", shell::SHELL);
    }
    name.to_owned()
}

#[derive(Debug, Clone, PartialEq)]
pub enum Update {
    Text(String),
    ToolStarted {
        id: String,
        title: String,
        reads: bool,
        input: Value,
    },
    ToolEnded {
        id: String,
        ok: bool,
        output: Vec<ToolResultContent>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ending {
    Done,
    Cancelled,
    Refused,
}

pub trait Peer: Clone + Send + Sync + 'static {
    fn update(&self, update: Update);
    fn ask(&self, id: &str, title: &str, input: &Value) -> impl Future<Output = Decision> + Send;
    fn allow_always(&self, allowance: &str);
    fn background_ended(&self, command: &str, report: String);
}

#[derive(Debug, thiserror::Error)]
pub enum TurnError {
    #[error("The model service answered with an error: {0}")]
    Model(String),
}

#[derive(Debug)]
pub struct Turn<'a> {
    pub folder: PathBuf,
    pub system: String,
    pub model: &'a DynModel<Completion>,
    pub servers: &'a ToolServers,
    pub cancel: CancellationToken,
    pub mode: Mode,
    pub allowed: Mutex<HashSet<String>>,
    pub depth: u8,
}

pub const SUBAGENT: &str = "agent";
const SUBAGENT_BRIEF: &str = "You are a subagent: another agent gave you this task. Do it with the tools, then answer with what you found or did; your answer goes back to that agent, not to a person.";

fn subagent_definition() -> Option<rig_core::completion::ToolDefinition> {
    Some(rig_core::completion::ToolDefinition {
        name: rig_core::message::ToolName::new(SUBAGENT).ok()?,
        description: "Hand a task to a subagent: another session of this agent with the same tools and a fresh history. It works through the task and answers with what it found or did. Use it for a search or a piece of work whose steps you don't need to see.".to_owned(),
        parameters: serde_json::json!({ "type": "object", "properties": {
            "task": { "type": "string", "description": "What the subagent should do, with everything it needs to know." }
        }, "required": ["task"] }),
        annotations: rig_core::completion::ToolAnnotations::default(),
    })
}

fn last_text(history: &[Message]) -> String {
    history
        .iter()
        .rev()
        .find_map(|message| match message {
            Message::Assistant(turn) => {
                let text: Vec<&str> = turn
                    .content
                    .iter()
                    .filter_map(AssistantContent::as_text)
                    .collect();
                (!text.is_empty()).then(|| text.join(""))
            }
            _ => None,
        })
        .unwrap_or_default()
}

fn request(
    system: &str,
    history: &[Message],
    tools: &[rig_core::completion::ToolDefinition],
) -> Option<CompletionRequest> {
    let (last, earlier) = history.split_last()?;
    Some(
        CompletionRequest::new(last.clone())
            .messages(earlier.iter().cloned())
            .preamble(system)
            .tools(tools.iter().cloned()),
    )
}

fn calls(message: &Message) -> Vec<ToolCall> {
    match message {
        Message::Assistant(turn) => turn
            .content
            .iter()
            .filter_map(|block| match block {
                AssistantContent::ToolCall(call) => Some(call.clone()),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

impl Turn<'_> {
    fn needs_asking(&self, name: &str, input: &Value) -> bool {
        if self.mode == Mode::Full || name == SUBAGENT {
            return false;
        }
        if self
            .allowed
            .lock()
            .expect("allowed")
            .contains(&allowance(name, input))
        {
            return false;
        }
        if name == shell::SHELL {
            let line = input
                .get("command")
                .and_then(Value::as_str)
                .unwrap_or_default();
            return !shell::reads_only(line);
        }
        if tools::is_local(name) {
            tools::effect(name) == tools::Effect::Changes
        } else {
            !self.servers.reads_only(name)
        }
    }

    async fn run_shell(
        &self,
        input: &serde_json::Map<String, Value>,
        peer: &impl Peer,
    ) -> (bool, Vec<ToolResultContent>) {
        let Some(line) = input
            .get("command")
            .and_then(Value::as_str)
            .map(str::to_owned)
        else {
            return (
                false,
                vec![ToolResultContent::text("command is needed, as a string.")],
            );
        };
        let timeout = shell::timeout_of(input.get("timeout_seconds").and_then(Value::as_u64));
        if input.get("background").and_then(Value::as_bool) == Some(true) {
            let folder = self.folder.clone();
            let (sender, receiver) = tokio::sync::oneshot::channel();
            let watched = line.clone();
            tokio::spawn(async move {
                let report = match shell::run(&watched, &folder, timeout).await {
                    Ok(ran) => ran.report(),
                    Err(error) => error.to_string(),
                };
                if sender.send(report).is_err() {
                    log::debug!("no one waits for the background command any more");
                }
            });
            Self::deliver_later(line, receiver, peer);
            return (
                true,
                vec![ToolResultContent::text(
                    "Started in the background. Its result comes back as a message when it ends.",
                )],
            );
        }
        match shell::run(&line, &self.folder, timeout).await {
            Ok(ran) => (ran.ok(), vec![ToolResultContent::text(ran.report())]),
            Err(error) => (false, vec![ToolResultContent::text(error.to_string())]),
        }
    }

    fn deliver_later(
        line: String,
        receiver: tokio::sync::oneshot::Receiver<String>,
        peer: &impl Peer,
    ) {
        let peer = peer.clone();
        tokio::spawn(async move {
            if let Ok(report) = receiver.await {
                peer.background_ended(&line, report);
            }
        });
    }

    async fn run_tool(&self, call: &ToolCall, peer: &impl Peer) -> (bool, Vec<ToolResultContent>) {
        let name = call.function.name.as_ref();
        let arguments = call.function.arguments.clone();
        if name == shell::SHELL {
            return self.run_shell(&arguments, peer).await;
        }
        if name == SUBAGENT && self.depth == 0 {
            return Box::pin(self.run_subagent(&arguments, peer)).await;
        }
        if tools::is_local(name) {
            match tools::run(&self.folder, name, &arguments) {
                Ok(content) => (true, content),
                Err(error) => (false, vec![ToolResultContent::text(error.to_string())]),
            }
        } else {
            match self.servers.call(name, arguments).await {
                Ok(text) => (true, vec![ToolResultContent::text(text)]),
                Err(text) => (false, vec![ToolResultContent::text(text)]),
            }
        }
    }

    async fn run_subagent(
        &self,
        input: &serde_json::Map<String, Value>,
        peer: &impl Peer,
    ) -> (bool, Vec<ToolResultContent>) {
        let Some(task) = input.get("task").and_then(Value::as_str) else {
            return (
                false,
                vec![ToolResultContent::text("task is needed, as a string.")],
            );
        };
        let sub = Turn {
            folder: self.folder.clone(),
            system: format!("{}\n\n{SUBAGENT_BRIEF}", self.system),
            model: self.model,
            servers: self.servers,
            cancel: self.cancel.child_token(),
            mode: self.mode,
            allowed: Mutex::new(self.allowed.lock().expect("allowed").clone()),
            depth: self.depth + 1,
        };
        let mut history = vec![Message::user(task)];
        let ended = Box::pin(sub.run(&mut history, peer, |_| {})).await;
        let learned = sub.allowed.into_inner().expect("allowed");
        self.allowed.lock().expect("allowed").extend(learned);
        match ended {
            Ok(Ending::Done) => (true, vec![ToolResultContent::text(last_text(&history))]),
            Ok(Ending::Cancelled) => (
                false,
                vec![ToolResultContent::text("The subagent was stopped.")],
            ),
            Ok(Ending::Refused) => (
                false,
                vec![ToolResultContent::text(
                    "The person didn't allow what the subagent asked for.",
                )],
            ),
            Err(error) => (false, vec![ToolResultContent::text(error.to_string())]),
        }
    }

    async fn answer(&self, call: &ToolCall, peer: &impl Peer) -> Result<ToolResult, Ending> {
        let id = if self.depth == 0 {
            call.id.to_string()
        } else {
            format!("{}-{}", self.depth, call.id)
        };
        let name = call.function.name.to_string();
        let input = Value::Object(call.function.arguments.clone());
        let asks = self.needs_asking(&name, &input);
        peer.update(Update::ToolStarted {
            id: id.clone(),
            title: name.clone(),
            reads: !asks,
            input: input.clone(),
        });
        if asks {
            let decision = tokio::select! {
                decision = peer.ask(&id, &name, &input) => decision,
                () = self.cancel.cancelled() => return Err(Ending::Cancelled),
            };
            match decision {
                Decision::Once => {}
                Decision::Always => {
                    let allowance = allowance(&name, &input);
                    peer.allow_always(&allowance);
                    self.allowed.lock().expect("allowed").insert(allowance);
                }
                Decision::Reject => {
                    let refused = vec![ToolResultContent::text("The person didn't allow this.")];
                    peer.update(Update::ToolEnded {
                        id,
                        ok: false,
                        output: refused,
                    });
                    return Err(Ending::Refused);
                }
            }
        }
        let (ok, output) = tokio::select! {
            ran = self.run_tool(call, peer) => ran,
            () = self.cancel.cancelled() => return Err(Ending::Cancelled),
        };
        peer.update(Update::ToolEnded {
            id,
            ok,
            output: output.clone(),
        });
        let mut result = call.result(output);
        result.is_error = !ok;
        Ok(result)
    }

    async fn reply(&self, history: &[Message], peer: &impl Peer) -> Result<Message, TurnError> {
        let mut definitions = tools::definitions();
        definitions.extend(shell::definition());
        if self.depth == 0 {
            definitions.extend(subagent_definition());
        }
        definitions.extend(self.servers.definitions());
        let request = request(&self.system, history, &definitions)
            .ok_or_else(|| TurnError::Model("nothing to answer".to_owned()))?;
        let mut stream = self
            .model
            .stream(request)
            .map_err(|e| TurnError::Model(e.to_string()))?;
        while let Some(item) = stream.next().await {
            let item = item.map_err(|e| TurnError::Model(e.to_string()))?;
            if let rig_core::streaming::Item::Event(StreamEvent::Text { text, .. }) = item
                && self.depth == 0
            {
                peer.update(Update::Text(text));
            }
        }
        let response = stream
            .finish()
            .await
            .map_err(|e| TurnError::Model(e.to_string()))?;
        if let Some(error) = response.error {
            return Err(TurnError::Model(error));
        }
        Ok(Message::Assistant(
            rig_core::message::AssistantMessage::new(response.choice),
        ))
    }

    pub async fn run(
        &self,
        history: &mut Vec<Message>,
        peer: &impl Peer,
        mut saved: impl FnMut(&Message),
    ) -> Result<Ending, TurnError> {
        loop {
            if self.cancel.is_cancelled() {
                return Ok(Ending::Cancelled);
            }
            let reply = tokio::select! {
                reply = self.reply(history, peer) => reply?,
                () = self.cancel.cancelled() => return Ok(Ending::Cancelled),
            };
            let calls = calls(&reply);
            saved(&reply);
            history.push(reply);
            if calls.is_empty() {
                return Ok(Ending::Done);
            }
            let mut results = Vec::new();
            let mut ended = None;
            for call in &calls {
                if ended.is_some() {
                    let mut skipped = call.result(vec![ToolResultContent::text(
                        "Not run: the turn ended first.",
                    )]);
                    skipped.is_error = true;
                    results.push(skipped);
                    continue;
                }
                match self.answer(call, peer).await {
                    Ok(result) => results.push(result),
                    Err(ending) => {
                        ended = Some(ending);
                        let mut stopped = call.result(vec![ToolResultContent::text("Not run.")]);
                        stopped.is_error = true;
                        results.push(stopped);
                    }
                }
            }
            let message = Message::tool_results(results);
            saved(&message);
            history.push(message);
            if let Some(ending) = ended {
                return Ok(ending);
            }
        }
    }
}
