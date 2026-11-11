use std::path::Path;
use std::time::Duration;

use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};

pub const DB_FILE: &str = "opsis.db";
pub const SCHEMA_SQL: &str = include_str!("../schema.sql");
const META_ID: i64 = 1;
const BUSY_TIMEOUT: Duration = Duration::from_secs(5);

pub fn schema_hash() -> String {
    hex::encode(Sha256::digest(SCHEMA_SQL.as_bytes()))
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error(transparent)]
    Sql(#[from] sqlx::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Settings(#[from] crate::settings::SettingsError),
    #[error("An in-memory database can't hold another schema")]
    InMemorySchema,
}

pub type Result<T> = std::result::Result<T, StoreError>;

#[derive(Debug)]
pub struct Database {
    pub(crate) pool: SqlitePool,
}

fn options() -> SqliteConnectOptions {
    SqliteConnectOptions::new()
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .foreign_keys(true)
        .busy_timeout(BUSY_TIMEOUT)
}

async fn connect(file: Option<&Path>) -> Result<SqlitePool> {
    let pool = match file {
        Some(file) => {
            if let Some(dir) = file.parent() {
                std::fs::create_dir_all(dir)?;
            }
            SqlitePool::connect_with(options().filename(file)).await?
        }
        None => {
            SqlitePoolOptions::new()
                .max_connections(1)
                .idle_timeout(None)
                .max_lifetime(None)
                .connect_with(options().in_memory(true))
                .await?
        }
    };
    Ok(pool)
}

async fn stored_hash(pool: &SqlitePool) -> Option<String> {
    sqlx::query_scalar!("SELECT schema_hash FROM meta WHERE id = ?", META_ID)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
}

async fn has_tables(pool: &SqlitePool) -> Result<bool> {
    Ok(
        sqlx::query_scalar!("SELECT 1 AS present FROM sqlite_schema LIMIT 1")
            .fetch_optional(pool)
            .await?
            .is_some(),
    )
}

async fn create_schema(pool: &SqlitePool) -> Result<()> {
    let mut tx = pool.begin().await?;
    sqlx::raw_sql(SCHEMA_SQL).execute(&mut *tx).await?;
    let hash = schema_hash();
    sqlx::query!(
        "INSERT INTO meta (id, schema_hash) VALUES (?, ?)",
        META_ID,
        hash
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

fn remove_files(file: &Path) -> std::io::Result<()> {
    for suffix in ["", "-wal", "-shm"] {
        match std::fs::remove_file(format!("{}{suffix}", file.display())) {
            Err(error) if error.kind() != std::io::ErrorKind::NotFound => return Err(error),
            _ => {}
        }
    }
    Ok(())
}

impl Database {
    pub async fn open(file: &Path) -> Result<Self> {
        Self::open_at(Some(file)).await
    }

    pub async fn in_memory() -> Result<Self> {
        Self::open_at(None).await
    }

    async fn open_at(file: Option<&Path>) -> Result<Self> {
        let pool = connect(file).await?;
        if stored_hash(&pool).await.as_deref() == Some(schema_hash().as_str()) {
            return Ok(Self { pool });
        }
        if !has_tables(&pool).await? {
            create_schema(&pool).await?;
            return Ok(Self { pool });
        }
        let Some(file) = file else {
            return Err(StoreError::InMemorySchema);
        };
        log::warn!(
            "{} was made for another schema and was replaced",
            file.display()
        );
        pool.close().await;
        remove_files(file)?;
        let pool = connect(Some(file)).await?;
        create_schema(&pool).await?;
        Ok(Self { pool })
    }

    pub async fn table_names(&self) -> Result<Vec<String>> {
        let names = sqlx::query_scalar!(
            "SELECT name FROM sqlite_schema WHERE type = 'table' ORDER BY name"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(names.into_iter().flatten().collect())
    }

    pub async fn stored_schema_hash(&self) -> Option<String> {
        stored_hash(&self.pool).await
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
            ["fonts", "meta", "projects", "settings", "view_state"]
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
    async fn replaces_a_database_made_for_another_schema() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join(DB_FILE);
        let db = Database::open(&file).await.unwrap();
        add_setting(&db).await;
        sqlx::query!("UPDATE meta SET schema_hash = 'older'")
            .execute(&db.pool)
            .await
            .unwrap();
        db.pool.close().await;
        let db = Database::open(&file).await.unwrap();
        assert_eq!(count_settings(&db).await, 0);
        assert_eq!(db.stored_schema_hash().await, Some(schema_hash()));
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
