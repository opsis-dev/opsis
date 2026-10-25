use garde::Validate;
use opsis_types::settings::{AppSettings, SETTINGS_SECTIONS};
use serde_json::{Map, Value};

use crate::db::{Database, Result, StoreError};

pub type SettingsPatch = Map<String, Value>;

fn parsed(object: Map<String, Value>) -> std::result::Result<AppSettings, String> {
    let settings: AppSettings =
        serde_json::from_value(Value::Object(object)).map_err(|e| e.to_string())?;
    settings.validate().map_err(|e| e.to_string())?;
    Ok(settings)
}

impl Database {
    pub fn read_settings(&self) -> Result<AppSettings> {
        let mut stmt = self.conn.prepare("SELECT section, value FROM settings")?;
        let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
        let mut object = Map::new();
        for row in rows {
            let (section, value) = row?;
            object.insert(section, serde_json::from_str(&value).unwrap_or(Value::Null));
        }
        Ok(parsed(object).unwrap_or_else(|problem| {
            log::warn!("{problem}");
            AppSettings::default()
        }))
    }

    pub fn patch_settings(&mut self, patch: &SettingsPatch) -> Result<(AppSettings, AppSettings)> {
        let previous = self.read_settings()?;
        let current = serde_json::to_value(&previous)?;
        let mut merged = Map::new();
        for section in SETTINGS_SECTIONS {
            let mut fields = current[section].as_object().cloned().unwrap_or_default();
            if let Some(Value::Object(changes)) = patch.get(section) {
                fields.extend(changes.clone());
            }
            merged.insert(section.to_owned(), Value::Object(fields));
        }
        let mut next = parsed(merged).map_err(StoreError::Invalid)?;
        if next.projects.default_dir != previous.projects.default_dir
            && !next.projects.last_dir.is_empty()
        {
            next.projects.last_dir.clear();
        }
        let values = serde_json::to_value(&next)?;
        let tx = self.conn.transaction()?;
        for section in SETTINGS_SECTIONS {
            tx.execute(
                "INSERT INTO settings (section, value) VALUES (?1, ?2) ON CONFLICT(section) DO UPDATE SET value = excluded.value",
                rusqlite::params![section, values[section].to_string()],
            )?;
        }
        tx.commit()?;
        Ok((next, previous))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn patch(value: Value) -> SettingsPatch {
        value.as_object().cloned().unwrap()
    }

    #[test]
    fn starts_with_the_defaults() {
        assert_eq!(
            Database::in_memory().unwrap().read_settings().unwrap(),
            AppSettings::default()
        );
    }

    #[test]
    fn keeps_a_patch_and_returns_what_changed() {
        let mut db = Database::in_memory().unwrap();
        let (next, previous) = db
            .patch_settings(&patch(json!({ "app": { "confirmExit": false } })))
            .unwrap();
        assert!(previous.app.confirm_exit);
        assert!(!next.app.confirm_exit);
        assert_eq!(db.read_settings().unwrap().app, next.app);
    }

    #[test]
    fn forgets_the_last_folder_when_the_default_folder_changes() {
        let mut db = Database::in_memory().unwrap();
        db.patch_settings(&patch(json!({ "projects": { "lastDir": "/a" } })))
            .unwrap();
        db.patch_settings(&patch(json!({ "projects": { "defaultDir": "/b" } })))
            .unwrap();
        let projects = db.read_settings().unwrap().projects;
        assert_eq!(
            (projects.default_dir.as_str(), projects.last_dir.as_str()),
            ("/b", "")
        );
    }

    #[test]
    fn falls_back_to_the_defaults_when_a_saved_section_cant_be_read() {
        let db = Database::in_memory().unwrap();
        db.conn
            .execute(
                "INSERT INTO settings (section, value) VALUES ('app', '{\"onStart\":\"nowhere\"}')",
                [],
            )
            .unwrap();
        assert_eq!(db.read_settings().unwrap(), AppSettings::default());
    }

    #[test]
    fn refuses_a_patch_out_of_range() {
        let mut db = Database::in_memory().unwrap();
        assert!(
            db.patch_settings(&patch(json!({ "traces": { "keepLast": 0 } })))
                .is_err()
        );
    }
}
