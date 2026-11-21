use serde::{Deserialize, Serialize};
use sqlx::{Sqlite, Transaction};

use crate::db::{Database, Result};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AssistantEvent {
    #[serde(rename_all = "camelCase")]
    SessionStarted {
        agent_session: Option<String>,
    },
    UserMessage {
        text: String,
    },
    AgentText {
        text: String,
    },
    TurnStarted,
    TurnEnded {
        stop: String,
    },
    RequestAsked {
        request: String,
        kind: String,
        title: String,
    },
    RequestAnswered {
        request: String,
        answer: String,
    },
    ToolCalled {
        tool: String,
        kind: String,
        title: String,
        input: Option<String>,
    },
    ToolFailed {
        tool: String,
    },
}

impl AssistantEvent {
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::SessionStarted { .. } => "sessionStarted",
            Self::UserMessage { .. } => "userMessage",
            Self::AgentText { .. } => "agentText",
            Self::TurnStarted => "turnStarted",
            Self::TurnEnded { .. } => "turnEnded",
            Self::RequestAsked { .. } => "requestAsked",
            Self::RequestAnswered { .. } => "requestAnswered",
            Self::ToolCalled { .. } => "toolCalled",
            Self::ToolFailed { .. } => "toolFailed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recorded {
    pub session: String,
    pub project: String,
    pub agent: String,
    pub at: i64,
    pub event: AssistantEvent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantSession {
    pub id: String,
    pub project: String,
    pub agent: String,
    pub agent_session: Option<String>,
    pub started_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantThread {
    pub id: String,
    pub title: String,
    pub updated_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantMessage {
    pub seq: i64,
    pub role: String,
    pub text: String,
    pub at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantRequest {
    pub request: String,
    pub kind: String,
    pub title: String,
    pub answer: Option<String>,
    pub asked_at: i64,
    pub answered_at: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantTurn {
    pub seq: i64,
    pub started_at: i64,
    pub ended_at: Option<i64>,
    pub stop: Option<String>,
}

async fn project(tx: &mut Transaction<'_, Sqlite>, seq: i64, recorded: &Recorded) -> Result<()> {
    let Recorded {
        session,
        project,
        agent,
        at,
        event,
    } = recorded;
    if !matches!(event, AssistantEvent::SessionStarted { .. }) {
        sqlx::query!(
            "UPDATE assistant_sessions SET updated_at = ? WHERE id = ?",
            at,
            session
        )
        .execute(&mut **tx)
        .await?;
    }
    match event {
        AssistantEvent::SessionStarted { agent_session } => {
            sqlx::query!(
                "INSERT INTO assistant_sessions (id, project, agent, agent_session, started_at, updated_at, open_message)
                 VALUES (?, ?, ?, ?, ?, ?, NULL)
                 ON CONFLICT (id) DO UPDATE SET agent_session = excluded.agent_session, updated_at = excluded.updated_at, open_message = NULL",
                session,
                project,
                agent,
                agent_session,
                at,
                at
            )
            .execute(&mut **tx)
            .await?;
        }
        AssistantEvent::UserMessage { text } => {
            sqlx::query!(
                "INSERT INTO assistant_messages (session_id, seq, role, text, at) VALUES (?, ?, 'user', ?, ?)",
                session,
                seq,
                text,
                at
            )
            .execute(&mut **tx)
            .await?;
            close_message(tx, session).await?;
        }
        AssistantEvent::AgentText { text } => {
            let open = sqlx::query_scalar!(
                "SELECT open_message FROM assistant_sessions WHERE id = ?",
                session
            )
            .fetch_optional(&mut **tx)
            .await?
            .flatten();
            if let Some(open) = open {
                sqlx::query!(
                    "UPDATE assistant_messages SET text = text || ? WHERE session_id = ? AND seq = ?",
                    text,
                    session,
                    open
                )
                .execute(&mut **tx)
                .await?;
            } else {
                sqlx::query!(
                    "INSERT INTO assistant_messages (session_id, seq, role, text, at) VALUES (?, ?, 'agent', ?, ?)",
                    session,
                    seq,
                    text,
                    at
                )
                .execute(&mut **tx)
                .await?;
                sqlx::query!(
                    "UPDATE assistant_sessions SET open_message = ? WHERE id = ?",
                    seq,
                    session
                )
                .execute(&mut **tx)
                .await?;
            }
        }
        AssistantEvent::TurnStarted => {
            sqlx::query!(
                "INSERT INTO assistant_turns (session_id, seq, started_at) VALUES (?, ?, ?)",
                session,
                seq,
                at
            )
            .execute(&mut **tx)
            .await?;
            close_message(tx, session).await?;
        }
        AssistantEvent::TurnEnded { stop } => {
            sqlx::query!(
                "UPDATE assistant_turns SET ended_at = ?, stop = ? WHERE session_id = ? AND ended_at IS NULL",
                at,
                stop,
                session
            )
            .execute(&mut **tx)
            .await?;
            close_message(tx, session).await?;
        }
        AssistantEvent::RequestAsked {
            request,
            kind,
            title,
        } => {
            sqlx::query!(
                "INSERT INTO assistant_requests (session_id, request, kind, title, asked_at) VALUES (?, ?, ?, ?, ?)
                 ON CONFLICT (session_id, request) DO UPDATE SET kind = excluded.kind, title = excluded.title, asked_at = excluded.asked_at, answer = NULL, answered_at = NULL",
                session,
                request,
                kind,
                title,
                at
            )
            .execute(&mut **tx)
            .await?;
        }
        AssistantEvent::RequestAnswered { request, answer } => {
            sqlx::query!(
                "UPDATE assistant_requests SET answer = ?, answered_at = ? WHERE session_id = ? AND request = ?",
                answer,
                at,
                session,
                request
            )
            .execute(&mut **tx)
            .await?;
        }
        AssistantEvent::ToolCalled {
            tool,
            kind,
            title,
            input,
        } => {
            let call = serde_json::json!({
                "tool": tool,
                "kind": kind,
                "title": title,
                "input": input,
                "failed": false,
            })
            .to_string();
            sqlx::query!(
                "INSERT INTO assistant_messages (session_id, seq, role, text, at) VALUES (?, ?, 'tool', ?, ?)",
                session,
                seq,
                call,
                at
            )
            .execute(&mut **tx)
            .await?;
            close_message(tx, session).await?;
        }
        AssistantEvent::ToolFailed { tool } => {
            sqlx::query!(
                "UPDATE assistant_messages SET text = json_set(text, '$.failed', json('true'))
                 WHERE session_id = ? AND role = 'tool' AND json_extract(text, '$.tool') = ?",
                session,
                tool
            )
            .execute(&mut **tx)
            .await?;
        }
    }
    Ok(())
}

async fn close_message(tx: &mut Transaction<'_, Sqlite>, session: &str) -> Result<()> {
    sqlx::query!(
        "UPDATE assistant_sessions SET open_message = NULL WHERE id = ?",
        session
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

impl Database {
    pub async fn record_assistant_event(&self, recorded: &Recorded) -> Result<i64> {
        let payload = serde_json::to_string(&recorded.event)?;
        let kind = recorded.event.kind();
        let mut tx = self.pool.begin().await?;
        let seq = sqlx::query!(
            "INSERT INTO assistant_events (session_id, project, agent, kind, at, payload) VALUES (?, ?, ?, ?, ?, ?)",
            recorded.session,
            recorded.project,
            recorded.agent,
            kind,
            recorded.at,
            payload
        )
        .execute(&mut *tx)
        .await?
        .last_insert_rowid();
        project(&mut tx, seq, recorded).await?;
        tx.commit().await?;
        Ok(seq)
    }

    pub async fn rebuild_assistant_projections(&self) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        for table in [
            "DELETE FROM assistant_sessions",
            "DELETE FROM assistant_messages",
            "DELETE FROM assistant_turns",
            "DELETE FROM assistant_requests",
        ] {
            sqlx::query(table).execute(&mut *tx).await?;
        }
        let rows = sqlx::query!(
            "SELECT seq, session_id, project, agent, at, payload FROM assistant_events ORDER BY seq"
        )
        .fetch_all(&mut *tx)
        .await?;
        for row in rows {
            let event: AssistantEvent = serde_json::from_str(&row.payload)?;
            let recorded = Recorded {
                session: row.session_id,
                project: row.project,
                agent: row.agent,
                at: row.at,
                event,
            };
            project(&mut tx, row.seq, &recorded).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn assistant_sessions(&self, project: Option<&str>) -> Result<Vec<AssistantSession>> {
        Ok(sqlx::query_as!(
            AssistantSession,
            "SELECT id AS \"id!\", project, agent, agent_session, started_at, updated_at FROM assistant_sessions
             WHERE ?1 IS NULL OR project = ?1 ORDER BY updated_at DESC, id",
            project
        )
        .fetch_all(&self.pool)
        .await?)
    }

    pub async fn assistant_threads(&self, project: &str) -> Result<Vec<AssistantThread>> {
        Ok(sqlx::query_as!(
            AssistantThread,
            "SELECT s.id AS \"id!\", m.text AS \"title!\", s.updated_at AS \"updated_at!\"
             FROM assistant_sessions s
             JOIN assistant_messages m ON m.session_id = s.id AND m.seq = (
               SELECT MIN(seq) FROM assistant_messages WHERE session_id = s.id AND role = 'user'
             )
             WHERE s.project = ? ORDER BY s.updated_at DESC, s.id",
            project
        )
        .fetch_all(&self.pool)
        .await?)
    }

    pub async fn assistant_messages(&self, session: &str) -> Result<Vec<AssistantMessage>> {
        Ok(sqlx::query_as!(
            AssistantMessage,
            "SELECT seq, role, text, at FROM assistant_messages WHERE session_id = ? ORDER BY seq",
            session
        )
        .fetch_all(&self.pool)
        .await?)
    }

    pub async fn assistant_turns(&self, session: &str) -> Result<Vec<AssistantTurn>> {
        Ok(sqlx::query_as!(
            AssistantTurn,
            "SELECT seq, started_at, ended_at, stop FROM assistant_turns WHERE session_id = ? ORDER BY seq",
            session
        )
        .fetch_all(&self.pool)
        .await?)
    }

    pub async fn assistant_requests(&self, session: &str) -> Result<Vec<AssistantRequest>> {
        Ok(sqlx::query_as!(
            AssistantRequest,
            "SELECT request, kind, title, answer, asked_at, answered_at FROM assistant_requests
             WHERE session_id = ? ORDER BY asked_at DESC, request",
            session
        )
        .fetch_all(&self.pool)
        .await?)
    }

    pub async fn clear_assistant_log(&self) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        for table in [
            "DELETE FROM assistant_events",
            "DELETE FROM assistant_sessions",
            "DELETE FROM assistant_messages",
            "DELETE FROM assistant_turns",
            "DELETE FROM assistant_requests",
        ] {
            sqlx::query(table).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }
}
