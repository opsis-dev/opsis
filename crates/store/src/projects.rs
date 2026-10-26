use std::path::Path;

use opsis_types::workflow::WORKFLOW_FILE;
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
    pub async fn list_recent(&self) -> Result<Vec<RecentProject>> {
        let rows = sqlx::query_as!(
            RecentProject,
            "SELECT dir, name, opened_at FROM projects ORDER BY opened_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .filter(|p| Path::new(&p.dir).join(WORKFLOW_FILE).exists())
            .take(MAX_RECENT)
            .collect())
    }

    pub async fn touch_recent(&self, dir: &str, name: &str, opened_at: i64) -> Result<()> {
        sqlx::query!(
            "INSERT INTO projects (dir, name, opened_at) VALUES (?, ?, ?) ON CONFLICT (dir) DO UPDATE SET name = excluded.name, opened_at = excluded.opened_at",
            dir,
            name,
            opened_at
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn project_count(&self) -> Result<i64> {
        Ok(sqlx::query_scalar!("SELECT count(*) FROM projects")
            .fetch_one(&self.pool)
            .await?)
    }

    async fn project_id(&self, dir: &str) -> Result<Option<i64>> {
        Ok(
            sqlx::query_scalar!("SELECT id FROM projects WHERE dir = ?", dir)
                .fetch_optional(&self.pool)
                .await?,
        )
    }

    pub async fn read_view_state(&self, dir: &str) -> Result<ViewPrefs> {
        let Some(id) = self.project_id(dir).await? else {
            return Ok(ViewPrefs::new());
        };
        let rows = sqlx::query!(
            "SELECT key, value FROM view_state WHERE project_id = ? ORDER BY id",
            id
        )
        .fetch_all(&self.pool)
        .await?;
        let mut prefs = ViewPrefs::new();
        for row in rows {
            prefs.insert(row.key, serde_json::from_str(&row.value)?);
        }
        Ok(prefs)
    }

    pub async fn write_view_state(&self, dir: &str, key: &str, value: &Value) -> Result<bool> {
        let Some(id) = self.project_id(dir).await? else {
            return Ok(false);
        };
        let text = value.to_string();
        sqlx::query!(
            "INSERT INTO view_state (project_id, key, value) VALUES (?, ?, ?) ON CONFLICT (project_id, key) DO UPDATE SET value = excluded.value",
            id,
            key,
            text
        )
        .execute(&self.pool)
        .await?;
        Ok(true)
    }

    pub async fn forget_view_state(&self, dir: &str) -> Result<()> {
        if let Some(id) = self.project_id(dir).await? {
            sqlx::query!("DELETE FROM view_state WHERE project_id = ?", id)
                .execute(&self.pool)
                .await?;
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

    #[tokio::test]
    async fn lists_the_last_opened_first_and_skips_folders_without_a_project() {
        let root = tempfile::tempdir().unwrap();
        let db = Database::in_memory().await.unwrap();
        let (first, second, gone) = (
            project(root.path(), "first"),
            project(root.path(), "second"),
            project(root.path(), "gone"),
        );
        db.touch_recent(&first, "First", 1).await.unwrap();
        db.touch_recent(&second, "Second", 2).await.unwrap();
        db.touch_recent(&gone, "Gone", 3).await.unwrap();
        std::fs::remove_dir_all(&gone).unwrap();
        db.touch_recent(&first, "First", 4).await.unwrap();
        let dirs: Vec<String> = db
            .list_recent()
            .await
            .unwrap()
            .into_iter()
            .map(|p| p.dir)
            .collect();
        assert_eq!(dirs, [first, second]);
    }

    #[tokio::test]
    async fn keeps_every_project_it_has_seen_beyond_the_recent_list() {
        let root = tempfile::tempdir().unwrap();
        let db = Database::in_memory().await.unwrap();
        for i in 0..21 {
            db.touch_recent(&project(root.path(), &format!("p{i}")), &format!("P{i}"), i)
                .await
                .unwrap();
        }
        assert_eq!(db.list_recent().await.unwrap().len(), MAX_RECENT);
        assert_eq!(db.project_count().await.unwrap(), 21);
    }

    #[tokio::test]
    async fn returns_what_was_saved_for_the_project() {
        let root = tempfile::tempdir().unwrap();
        let db = Database::in_memory().await.unwrap();
        let dir = project(root.path(), "views");
        db.touch_recent(&dir, "Views", 1).await.unwrap();
        assert!(
            db.write_view_state(&dir, "dockTab", &json!("mixer"))
                .await
                .unwrap()
        );
        db.write_view_state(&dir, "dockTab", &json!("timeline"))
            .await
            .unwrap();
        db.write_view_state(&dir, "mixerCollapsed", &json!(["music"]))
            .await
            .unwrap();
        assert_eq!(
            Value::Object(db.read_view_state(&dir).await.unwrap()),
            json!({ "dockTab": "timeline", "mixerCollapsed": ["music"] })
        );
    }

    #[tokio::test]
    async fn saves_nothing_for_a_folder_that_was_never_opened() {
        let db = Database::in_memory().await.unwrap();
        assert!(
            !db.write_view_state("/nowhere", "dockTab", &json!("mixer"))
                .await
                .unwrap()
        );
        assert!(db.read_view_state("/nowhere").await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn forgets_a_projects_view() {
        let root = tempfile::tempdir().unwrap();
        let db = Database::in_memory().await.unwrap();
        let dir = project(root.path(), "again");
        db.touch_recent(&dir, "Again", 1).await.unwrap();
        db.write_view_state(&dir, "dockTab", &json!("mixer"))
            .await
            .unwrap();
        db.forget_view_state(&dir).await.unwrap();
        assert!(db.read_view_state(&dir).await.unwrap().is_empty());
    }
}
