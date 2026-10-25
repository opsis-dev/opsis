use std::path::Path;

use rusqlite::{Connection, OptionalExtension};
use sha2::{Digest, Sha256};

pub const DB_FILE: &str = "opsis.db";
pub const SCHEMA_SQL: &str = include_str!("../../../src/main/db/schema.generated.sql");
const META_ID: i64 = 1;
const BUSY_TIMEOUT_MS: u32 = 5000;

pub fn schema_hash() -> String {
    hex::encode(Sha256::digest(SCHEMA_SQL.as_bytes()))
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error(transparent)]
    Sql(#[from] rusqlite::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Invalid(String),
}

pub type Result<T> = std::result::Result<T, StoreError>;

pub struct Database {
    pub(crate) conn: Connection,
}

fn connect(file: Option<&Path>) -> Result<Connection> {
    let conn = match file {
        Some(file) => {
            if let Some(dir) = file.parent() {
                std::fs::create_dir_all(dir)?;
            }
            Connection::open(file)?
        }
        None => Connection::open_in_memory()?,
    };
    conn.execute_batch(&format!(
        "PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON; PRAGMA busy_timeout = {BUSY_TIMEOUT_MS};"
    ))?;
    Ok(conn)
}

fn stored_hash(conn: &Connection) -> Option<String> {
    conn.query_row(
        "SELECT schemaHash FROM meta WHERE id = ?1",
        [META_ID],
        |r| r.get(0),
    )
    .optional()
    .ok()
    .flatten()
}

fn has_tables(conn: &Connection) -> Result<bool> {
    Ok(conn
        .query_row("SELECT 1 FROM sqlite_schema LIMIT 1", [], |_| Ok(()))
        .optional()?
        .is_some())
}

fn create_schema(mut conn: Connection) -> Result<Connection> {
    let tx = conn.transaction()?;
    tx.execute_batch(SCHEMA_SQL)?;
    tx.execute(
        "INSERT INTO meta (id, schemaHash) VALUES (?1, ?2)",
        rusqlite::params![META_ID, schema_hash()],
    )?;
    tx.commit()?;
    Ok(conn)
}

fn remove_files(file: &Path) {
    for suffix in ["", "-wal", "-shm"] {
        let _ = std::fs::remove_file(format!("{}{suffix}", file.display()));
    }
}

impl Database {
    pub fn open(file: &Path) -> Result<Self> {
        Self::open_at(Some(file))
    }

    pub fn in_memory() -> Result<Self> {
        Self::open_at(None)
    }

    fn open_at(file: Option<&Path>) -> Result<Self> {
        let conn = connect(file)?;
        if stored_hash(&conn).as_deref() == Some(schema_hash().as_str()) {
            return Ok(Self { conn });
        }
        if !has_tables(&conn)? {
            return Ok(Self {
                conn: create_schema(conn)?,
            });
        }
        let Some(file) = file else {
            return Err(StoreError::Invalid(
                "An in-memory database can't hold another schema".into(),
            ));
        };
        log::warn!(
            "{} was made for another schema and was replaced",
            file.display()
        );
        drop(conn);
        remove_files(file);
        Ok(Self {
            conn: create_schema(connect(Some(file))?)?,
        })
    }

    pub fn table_names(&self) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT name FROM sqlite_schema WHERE type = 'table' ORDER BY name")?;
        let names = stmt
            .query_map([], |r| r.get(0))?
            .collect::<std::result::Result<Vec<String>, _>>()?;
        Ok(names)
    }

    pub fn stored_schema_hash(&self) -> Option<String> {
        stored_hash(&self.conn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_every_table_of_the_schema() {
        let db = Database::in_memory().unwrap();
        assert_eq!(
            db.table_names().unwrap(),
            [
                "fonts",
                "meta",
                "projects",
                "settings",
                "view_state",
                "window_bounds"
            ]
        );
        assert_eq!(db.stored_schema_hash(), Some(schema_hash()));
    }

    fn count_settings(db: &Database) -> i64 {
        db.conn
            .query_row("SELECT count(*) FROM settings", [], |r| r.get(0))
            .unwrap()
    }

    #[test]
    fn keeps_a_database_made_for_this_schema() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join(DB_FILE);
        let db = Database::open(&file).unwrap();
        db.conn
            .execute(
                "INSERT INTO settings (section, value) VALUES ('app', '{}')",
                [],
            )
            .unwrap();
        drop(db);
        assert_eq!(count_settings(&Database::open(&file).unwrap()), 1);
    }

    #[test]
    fn replaces_a_database_made_for_another_schema() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join(DB_FILE);
        let db = Database::open(&file).unwrap();
        db.conn
            .execute(
                "INSERT INTO settings (section, value) VALUES ('app', '{}')",
                [],
            )
            .unwrap();
        db.conn
            .execute("UPDATE meta SET schemaHash = 'older'", [])
            .unwrap();
        drop(db);
        let db = Database::open(&file).unwrap();
        assert_eq!(count_settings(&db), 0);
        assert_eq!(db.stored_schema_hash(), Some(schema_hash()));
    }

    #[test]
    fn leaves_nothing_behind_when_a_transaction_fails() {
        let mut db = Database::in_memory().unwrap();
        let tx = db.conn.transaction().unwrap();
        tx.execute(
            "INSERT INTO settings (section, value) VALUES ('app', '{}')",
            [],
        )
        .unwrap();
        assert!(
            tx.execute(
                "INSERT INTO settings (section, value) VALUES ('app', '{}')",
                []
            )
            .is_err()
        );
        drop(tx);
        assert_eq!(count_settings(&db), 0);
    }
}
