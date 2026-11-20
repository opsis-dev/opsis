use std::path::Path;
use std::time::Duration;

use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};

pub const META_SQL: &str = include_str!("../meta.sql");
const META_ID: i64 = 1;
const BUSY_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error(transparent)]
    Sql(#[from] sqlx::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("An in-memory database can't hold another schema")]
    InMemorySchema,
}

pub fn schema_hash(schema: &str) -> String {
    hex::encode(Sha256::digest(schema.as_bytes()))
}

fn options() -> SqliteConnectOptions {
    SqliteConnectOptions::new()
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .foreign_keys(true)
        .busy_timeout(BUSY_TIMEOUT)
}

async fn connect(file: Option<&Path>) -> Result<SqlitePool, DatabaseError> {
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

pub async fn stored_schema_hash(pool: &SqlitePool) -> Option<String> {
    sqlx::query_scalar::<_, String>("SELECT schema_hash FROM meta WHERE id = ?")
        .bind(META_ID)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
}

async fn has_tables(pool: &SqlitePool) -> Result<bool, DatabaseError> {
    Ok(
        sqlx::query_scalar::<_, i64>("SELECT 1 FROM sqlite_schema LIMIT 1")
            .fetch_optional(pool)
            .await?
            .is_some(),
    )
}

async fn create_schema(pool: &SqlitePool, schema: &'static str) -> Result<(), DatabaseError> {
    let mut tx = pool.begin().await?;
    sqlx::raw_sql(META_SQL).execute(&mut *tx).await?;
    sqlx::raw_sql(schema).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO meta (id, schema_hash) VALUES (?, ?)")
        .bind(META_ID)
        .bind(schema_hash(schema))
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

pub async fn open(file: Option<&Path>, schema: &'static str) -> Result<SqlitePool, DatabaseError> {
    let pool = connect(file).await?;
    if stored_schema_hash(&pool).await.as_deref() == Some(schema_hash(schema).as_str()) {
        return Ok(pool);
    }
    if !has_tables(&pool).await? {
        create_schema(&pool, schema).await?;
        return Ok(pool);
    }
    let Some(file) = file else {
        return Err(DatabaseError::InMemorySchema);
    };
    log::warn!(
        "{} was made for another schema and was replaced",
        file.display()
    );
    pool.close().await;
    remove_files(file)?;
    let pool = connect(Some(file)).await?;
    create_schema(&pool, schema).await?;
    Ok(pool)
}

pub async fn table_names(pool: &SqlitePool) -> Result<Vec<String>, DatabaseError> {
    Ok(sqlx::query_scalar::<_, String>(
        "SELECT name FROM sqlite_schema WHERE type = 'table' ORDER BY name",
    )
    .fetch_all(pool)
    .await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOTES: &str = "CREATE TABLE notes (id INTEGER PRIMARY KEY, text TEXT NOT NULL);";
    const TASKS: &str = "CREATE TABLE tasks (id INTEGER PRIMARY KEY);";

    async fn count_notes(pool: &SqlitePool) -> i64 {
        sqlx::query_scalar("SELECT count(*) FROM notes")
            .fetch_one(pool)
            .await
            .unwrap()
    }

    async fn add_note(pool: &SqlitePool) {
        sqlx::query("INSERT INTO notes (text) VALUES ('kept')")
            .execute(pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn makes_the_schema_beside_its_own_meta_table() {
        let pool = open(None, NOTES).await.unwrap();
        assert_eq!(table_names(&pool).await.unwrap(), ["meta", "notes"]);
        assert_eq!(stored_schema_hash(&pool).await, Some(schema_hash(NOTES)));
    }

    #[tokio::test]
    async fn keeps_a_database_made_for_its_schema() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("notes.db");
        let pool = open(Some(&file), NOTES).await.unwrap();
        add_note(&pool).await;
        pool.close().await;
        assert_eq!(
            count_notes(&open(Some(&file), NOTES).await.unwrap()).await,
            1
        );
    }

    #[tokio::test]
    async fn replaces_a_database_made_for_another_schema() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("notes.db");
        let pool = open(Some(&file), NOTES).await.unwrap();
        add_note(&pool).await;
        pool.close().await;
        let pool = open(Some(&file), TASKS).await.unwrap();
        assert_eq!(table_names(&pool).await.unwrap(), ["meta", "tasks"]);
        assert_eq!(stored_schema_hash(&pool).await, Some(schema_hash(TASKS)));
    }

    #[tokio::test]
    async fn two_databases_in_one_folder_stay_apart() {
        let dir = tempfile::tempdir().unwrap();
        let notes = open(Some(&dir.path().join("a.db")), NOTES).await.unwrap();
        let tasks = open(Some(&dir.path().join("b.db")), TASKS).await.unwrap();
        add_note(&notes).await;
        assert_eq!(table_names(&tasks).await.unwrap(), ["meta", "tasks"]);
        assert_eq!(count_notes(&notes).await, 1);
    }
}
