use std::path::Path;

use opsis_persistence::db::DatabaseError;
use sqlx::SqlitePool;

pub const DB_FILE: &str = "opsis.db";
pub const SCHEMA_SQL: &str = include_str!("../schema.sql");

pub fn schema_hash() -> String {
    opsis_persistence::db::schema_hash(SCHEMA_SQL)
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error(transparent)]
    Sql(#[from] sqlx::Error),
    #[error(transparent)]
    Database(#[from] DatabaseError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Settings(#[from] crate::settings::SettingsError),
}

pub type Result<T> = std::result::Result<T, StoreError>;

#[derive(Debug)]
pub struct Database {
    pub(crate) pool: SqlitePool,
}

impl Database {
    pub async fn open(file: &Path) -> Result<Self> {
        Ok(Self {
            pool: opsis_persistence::db::open(Some(file), SCHEMA_SQL).await?,
        })
    }

    pub async fn in_memory() -> Result<Self> {
        Ok(Self {
            pool: opsis_persistence::db::open(None, SCHEMA_SQL).await?,
        })
    }

    pub async fn table_names(&self) -> Result<Vec<String>> {
        Ok(opsis_persistence::db::table_names(&self.pool).await?)
    }

    pub async fn stored_schema_hash(&self) -> Option<String> {
        opsis_persistence::db::stored_schema_hash(&self.pool).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn creates_every_table_of_the_schema() {
        let db = Database::in_memory().await.unwrap();
        assert_eq!(
            db.table_names().await.unwrap(),
            [
                "assistant_events",
                "assistant_messages",
                "assistant_requests",
                "assistant_sessions",
                "assistant_turns",
                "fonts",
                "meta",
                "projects",
                "settings",
                "view_state"
            ]
        );
        assert_eq!(db.stored_schema_hash().await, Some(schema_hash()));
    }

    async fn count_settings(db: &Database) -> i64 {
        sqlx::query_scalar!("SELECT count(*) FROM settings")
            .fetch_one(&db.pool)
            .await
            .unwrap()
    }

    async fn add_setting(db: &Database) {
        sqlx::query!("INSERT INTO settings (section, value) VALUES ('app', '{}')")
            .execute(&db.pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn keeps_a_database_made_for_this_schema() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join(DB_FILE);
        let db = Database::open(&file).await.unwrap();
        add_setting(&db).await;
        db.pool.close().await;
        assert_eq!(
            count_settings(&Database::open(&file).await.unwrap()).await,
            1
        );
    }

    #[tokio::test]
    async fn leaves_nothing_behind_when_a_transaction_fails() {
        let db = Database::in_memory().await.unwrap();
        let mut tx = db.pool.begin().await.unwrap();
        sqlx::query!("INSERT INTO settings (section, value) VALUES ('app', '{}')")
            .execute(&mut *tx)
            .await
            .unwrap();
        assert!(
            sqlx::query!("INSERT INTO settings (section, value) VALUES ('app', '{}')")
                .execute(&mut *tx)
                .await
                .is_err()
        );
        drop(tx);
        assert_eq!(count_settings(&db).await, 0);
    }
}
