use std::path::{Path, PathBuf};

use rig_core::message::Message;
use sqlx::SqlitePool;

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error(transparent)]
    Sql(#[from] sqlx::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredSession {
    pub id: String,
    pub folder: PathBuf,
    pub mode: String,
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .and_then(|elapsed| i64::try_from(elapsed.as_millis()).ok())
        .unwrap_or_default()
}

pub fn new_id() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}

pub async fn create(
    pool: &SqlitePool,
    id: &str,
    folder: &Path,
    mode: &str,
) -> Result<(), SessionError> {
    let folder = folder.to_string_lossy().into_owned();
    let now = now_ms();
    sqlx::query!(
        "INSERT INTO sessions (id, folder, mode, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
        id,
        folder,
        mode,
        now,
        now
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn find(pool: &SqlitePool, id: &str) -> Result<Option<StoredSession>, SessionError> {
    let row = sqlx::query!("SELECT id, folder, mode FROM sessions WHERE id = ?", id)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|row| StoredSession {
        id: row.id.unwrap_or_default(),
        folder: PathBuf::from(row.folder),
        mode: row.mode,
    }))
}

pub async fn set_mode(pool: &SqlitePool, id: &str, mode: &str) -> Result<(), SessionError> {
    let now = now_ms();
    sqlx::query!(
        "UPDATE sessions SET mode = ?, updated_at = ? WHERE id = ?",
        mode,
        now,
        id
    )
    .execute(pool)
    .await?;
    Ok(())
}

const fn role(message: &Message) -> &'static str {
    match message {
        Message::System { .. } => "system",
        Message::User { .. } => "user",
        Message::Assistant(_) => "assistant",
    }
}

pub async fn append(pool: &SqlitePool, id: &str, message: &Message) -> Result<(), SessionError> {
    let content = serde_json::to_string(message)?;
    let role = role(message);
    let now = now_ms();
    let mut tx = pool.begin().await?;
    sqlx::query!(
        "INSERT INTO messages (session_id, seq, role, content)
         VALUES (?, (SELECT COALESCE(MAX(seq), 0) + 1 FROM messages WHERE session_id = ?), ?, ?)",
        id,
        id,
        role,
        content
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!("UPDATE sessions SET updated_at = ? WHERE id = ?", now, id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn messages(pool: &SqlitePool, id: &str) -> Result<Vec<Message>, SessionError> {
    let rows = sqlx::query_scalar!(
        "SELECT content FROM messages WHERE session_id = ? ORDER BY seq",
        id
    )
    .fetch_all(pool)
    .await?;
    rows.iter()
        .map(|content| Ok(serde_json::from_str(content)?))
        .collect()
}

pub async fn allowed(pool: &SqlitePool, id: &str) -> Result<Vec<String>, SessionError> {
    Ok(sqlx::query_scalar!(
        "SELECT kind FROM allowed WHERE session_id = ? ORDER BY kind",
        id
    )
    .fetch_all(pool)
    .await?)
}

pub async fn allow(pool: &SqlitePool, id: &str, kind: &str) -> Result<(), SessionError> {
    sqlx::query!(
        "INSERT INTO allowed (session_id, kind) VALUES (?, ?) ON CONFLICT DO NOTHING",
        id,
        kind
    )
    .execute(pool)
    .await?;
    Ok(())
}
