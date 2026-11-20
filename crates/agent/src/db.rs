use std::path::Path;

use opsis_persistence::db::DatabaseError;
use sqlx::SqlitePool;

pub const AGENT_DB_FILE: &str = "agent.db";
pub const SCHEMA_SQL: &str = include_str!("../schema.sql");

pub async fn open(user_data: &Path) -> Result<SqlitePool, DatabaseError> {
    opsis_persistence::db::open(Some(&user_data.join(AGENT_DB_FILE)), SCHEMA_SQL).await
}
