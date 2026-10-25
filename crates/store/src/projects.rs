use std::path::Path;

use opsis_types::workflow::WORKFLOW_FILE;
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::db::{Database, Result};

pub const MAX_RECENT: usize = 20;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct RecentProject {
    pub dir: String,
    pub name: String,
    pub opened_at: i64,
}

pub type ViewPrefs = Map<String, Value>;

impl Database {
    pub fn list_recent(&self) -> Result<Vec<RecentProject>> {
        let mut stmt = self
            .conn
            .prepare("SELECT dir, name, openedAt FROM projects ORDER BY openedAt DESC")?;
        let rows = stmt.query_map([], |r| {
            Ok(RecentProject {
                dir: r.get(0)?,
                name: r.get(1)?,
                opened_at: r.get(2)?,
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            let project = row?;
            if Path::new(&project.dir).join(WORKFLOW_FILE).exists() {
                out.push(project);
            }
            if out.len() == MAX_RECENT {
                break;
            }
        }
        Ok(out)
    }

    pub fn touch_recent(&self, dir: &str, name: &str, opened_at: i64) -> Result<()> {
        self.conn.execute(
            "INSERT INTO projects (dir, name, openedAt) VALUES (?1, ?2, ?3) ON CONFLICT(dir) DO UPDATE SET name = excluded.name, openedAt = excluded.openedAt",
            rusqlite::params![dir, name, opened_at],
        )?;
        Ok(())
    }

    pub fn project_count(&self) -> Result<usize> {
        Ok(self
            .conn
            .query_row("SELECT count(*) FROM projects", [], |r| r.get::<_, i64>(0))?
            as usize)
    }

    fn project_id(&self, dir: &str) -> Result<Option<i64>> {
        Ok(self
            .conn
            .query_row("SELECT id FROM projects WHERE dir = ?1", [dir], |r| {
                r.get(0)
            })
            .optional()?)
    }

    pub fn read_view_state(&self, dir: &str) -> Result<ViewPrefs> {
        let Some(id) = self.project_id(dir)? else {
            return Ok(ViewPrefs::new());
        };
        let mut stmt = self
            .conn
            .prepare("SELECT key, value FROM view_state WHERE projectId = ?1 ORDER BY id")?;
        let rows = stmt.query_map([id], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })?;
        let mut prefs = ViewPrefs::new();
        for row in rows {
            let (key, value) = row?;
            prefs.insert(key, serde_json::from_str(&value)?);
        }
        Ok(prefs)
    }

    pub fn write_view_state(&self, dir: &str, key: &str, value: &Value) -> Result<bool> {
        let Some(id) = self.project_id(dir)? else {
            return Ok(false);
        };
        self.conn.execute(
            "INSERT INTO view_state (projectId, key, value) VALUES (?1, ?2, ?3) ON CONFLICT(projectId, key) DO UPDATE SET value = excluded.value",
            rusqlite::params![id, key, value.to_string()],
        )?;
        Ok(true)
    }

    pub fn forget_view_state(&self, dir: &str) -> Result<()> {
        if let Some(id) = self.project_id(dir)? {
            self.conn
                .execute("DELETE FROM view_state WHERE projectId = ?1", [id])?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn project(root: &Path, name: &str) -> String {
        let dir = root.join(name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(WORKFLOW_FILE), "{}").unwrap();
        dir.to_string_lossy().into_owned()
    }

    #[test]
    fn lists_the_last_opened_first_and_skips_folders_without_a_project() {
        let root = tempfile::tempdir().unwrap();
        let db = Database::in_memory().unwrap();
        let (first, second, gone) = (
            project(root.path(), "first"),
            project(root.path(), "second"),
            project(root.path(), "gone"),
        );
        db.touch_recent(&first, "First", 1).unwrap();
        db.touch_recent(&second, "Second", 2).unwrap();
        db.touch_recent(&gone, "Gone", 3).unwrap();
        std::fs::remove_dir_all(&gone).unwrap();
        db.touch_recent(&first, "First", 4).unwrap();
        let dirs: Vec<String> = db
            .list_recent()
            .unwrap()
            .into_iter()
            .map(|p| p.dir)
            .collect();
        assert_eq!(dirs, [first, second]);
    }

    #[test]
    fn keeps_every_project_it_has_seen_beyond_the_recent_list() {
        let root = tempfile::tempdir().unwrap();
        let db = Database::in_memory().unwrap();
        for i in 0..21 {
            db.touch_recent(&project(root.path(), &format!("p{i}")), &format!("P{i}"), i)
                .unwrap();
        }
        assert_eq!(db.list_recent().unwrap().len(), MAX_RECENT);
        assert_eq!(db.project_count().unwrap(), 21);
    }

    #[test]
    fn returns_what_was_saved_for_the_project() {
        let root = tempfile::tempdir().unwrap();
        let db = Database::in_memory().unwrap();
        let dir = project(root.path(), "views");
        db.touch_recent(&dir, "Views", 1).unwrap();
        assert!(
            db.write_view_state(&dir, "dockTab", &json!("mixer"))
                .unwrap()
        );
        db.write_view_state(&dir, "dockTab", &json!("timeline"))
            .unwrap();
        db.write_view_state(&dir, "mixerCollapsed", &json!(["music"]))
            .unwrap();
        assert_eq!(
            Value::Object(db.read_view_state(&dir).unwrap()),
            json!({ "dockTab": "timeline", "mixerCollapsed": ["music"] })
        );
    }

    #[test]
    fn saves_nothing_for_a_folder_that_was_never_opened() {
        let db = Database::in_memory().unwrap();
        assert!(
            !db.write_view_state("/nowhere", "dockTab", &json!("mixer"))
                .unwrap()
        );
        assert!(db.read_view_state("/nowhere").unwrap().is_empty());
    }

    #[test]
    fn forgets_a_projects_view() {
        let root = tempfile::tempdir().unwrap();
        let db = Database::in_memory().unwrap();
        let dir = project(root.path(), "again");
        db.touch_recent(&dir, "Again", 1).unwrap();
        db.write_view_state(&dir, "dockTab", &json!("mixer"))
            .unwrap();
        db.forget_view_state(&dir).unwrap();
        assert!(db.read_view_state(&dir).unwrap().is_empty());
    }
}
