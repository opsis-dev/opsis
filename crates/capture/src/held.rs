use std::fmt;
use std::sync::{Arc, Mutex};

use opsis_types::values::{ActionResult, ActionState, BrowserSessionValue, SessionStep};
use tokio::sync::{mpsc, oneshot};

use crate::take::{StepStatus, TakeEnv, TakeRun, step_name};

type Seen = Arc<Mutex<Vec<(String, StepStatus, Option<String>)>>>;

enum Command {
    Check {
        steps: Vec<SessionStep>,
        reply: oneshot::Sender<Vec<ActionResult>>,
    },
    Page {
        reply: oneshot::Sender<String>,
    },
}

pub struct HeldBrowser {
    commands: mpsc::UnboundedSender<Command>,
    address: String,
}

impl fmt::Debug for HeldBrowser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HeldBrowser")
            .field("address", &self.address)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{message}")]
pub struct HoldFailed {
    pub message: String,
    pub actions: Vec<ActionResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("The browser has closed.")]
pub struct BrowserClosed;

fn results(steps: &[SessionStep], seen: &Seen) -> Vec<ActionResult> {
    let seen = seen.lock().expect("seen steps");
    steps
        .iter()
        .map(|step| {
            let last = seen.iter().rev().find(|(id, status, _)| {
                *id == step.id && matches!(status, StepStatus::Done | StepStatus::Error)
            });
            let (state, message) = match last {
                Some((_, StepStatus::Done, _)) => (ActionState::Passed, None),
                Some((_, _, message)) => (ActionState::Failed, message.clone()),
                None => (ActionState::NotReached, None),
            };
            ActionResult {
                node_id: step.id.clone(),
                label: step_name(step, None),
                state,
                message,
            }
        })
        .collect()
}

fn first_failure(actions: &[ActionResult]) -> Option<String> {
    actions
        .iter()
        .find(|action| action.state == ActionState::Failed)
        .map(|action| {
            format!(
                "{}: {}",
                action.label,
                action.message.as_deref().unwrap_or("it failed.")
            )
        })
}

pub async fn hold(
    mut env: TakeEnv,
    session: BrowserSessionValue,
) -> Result<HeldBrowser, HoldFailed> {
    let seen: Seen = Arc::default();
    let inner = Arc::clone(&env.step);
    let noted = Arc::clone(&seen);
    env.step = Arc::new(move |id, status, message| {
        noted
            .lock()
            .expect("seen steps")
            .push((id.to_owned(), status, message.map(str::to_owned)));
        inner(id, status, message);
    });
    let (ready, opened) = oneshot::channel();
    let (commands, mut received) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        let mut take = match TakeRun::open(&env, &session, None).await {
            Ok(take) => take,
            Err(error) => {
                if ready
                    .send(Err(HoldFailed {
                        message: error.to_string(),
                        actions: Vec::new(),
                    }))
                    .is_err()
                {
                    log::debug!("no one waits for the browser to open");
                }
                return;
            }
        };
        seen.lock().expect("seen steps").clear();
        if let Err(error) = take.play_chain(&session.steps).await {
            let actions = results(&session.steps, &seen);
            take.close().await;
            let message = first_failure(&actions).unwrap_or_else(|| error.to_string());
            if ready.send(Err(HoldFailed { message, actions })).is_err() {
                log::debug!("no one waits for the browser to open");
            }
            return;
        }
        if ready.send(Ok(take.address())).is_err() {
            take.close().await;
            return;
        }
        while let Some(command) = received.recv().await {
            match command {
                Command::Check { steps, reply } => {
                    seen.lock().expect("seen steps").clear();
                    if let Err(error) = take.play_steps(&steps).await {
                        log::info!("a browser check stopped: {error}");
                    }
                    if reply.send(results(&steps, &seen)).is_err() {
                        log::debug!("no one waits for the browser check");
                    }
                }
                Command::Page { reply } => {
                    if reply.send(take.page_url()).is_err() {
                        log::debug!("no one waits for the page address");
                    }
                }
            }
        }
        take.close().await;
    });
    let address = opened.await.map_err(|_| HoldFailed {
        message: BrowserClosed.to_string(),
        actions: Vec::new(),
    })??;
    Ok(HeldBrowser { commands, address })
}

impl HeldBrowser {
    pub fn address(&self) -> &str {
        &self.address
    }

    pub async fn check(&self, steps: Vec<SessionStep>) -> Result<Vec<ActionResult>, BrowserClosed> {
        let (reply, answer) = oneshot::channel();
        self.commands
            .send(Command::Check { steps, reply })
            .map_err(|_| BrowserClosed)?;
        answer.await.map_err(|_| BrowserClosed)
    }

    pub async fn page(&self) -> Result<String, BrowserClosed> {
        let (reply, answer) = oneshot::channel();
        self.commands
            .send(Command::Page { reply })
            .map_err(|_| BrowserClosed)?;
        answer.await.map_err(|_| BrowserClosed)
    }
}
