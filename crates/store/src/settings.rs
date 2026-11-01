use garde::Validate;
use opsis_types::settings::{AppSettings, SETTINGS_SECTIONS};
use serde_json::{Map, Value};

use crate::db::{Database, Result};

pub type SettingsPatch = Map<String, Value>;

#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error(transparent)]
    Unreadable(#[from] serde_json::Error),
    #[error(transparent)]
    Invalid(#[from] garde::Report),
}

fn parsed(object: Map<String, Value>) -> std::result::Result<AppSettings, SettingsError> {
    let settings: AppSettings = serde_json::from_value(Value::Object(object))?;
    settings.validate()?;
    Ok(settings)
}

pub fn merged_settings(current: &AppSettings, patch: &SettingsPatch) -> Result<AppSettings> {
    let values = serde_json::to_value(current)?;
    let mut merged = Map::new();
    for section in SETTINGS_SECTIONS {
        let mut fields = values[section].as_object().cloned().unwrap_or_default();
        if let Some(Value::Object(changes)) = patch.get(section) {
            fields.extend(changes.clone());
        }
        merged.insert(section.to_owned(), Value::Object(fields));
    }
    let mut next = parsed(merged)?;
    if next.projects.default_dir != current.projects.default_dir
        && !next.projects.last_dir.is_empty()
    {
        next.projects.last_dir.clear();
    }
    Ok(next)
}

impl Database {
    pub async fn read_settings(&self) -> Result<AppSettings> {
        let rows = sqlx::query!("SELECT section, value FROM settings")
            .fetch_all(&self.pool)
            .await?;
        let mut object = Map::new();
        for row in rows {
            let read = serde_json::from_str(&row.value).unwrap_or_else(|error| {
                log::warn!("the {} settings couldn't be read: {error}", row.section);
                Value::Null
            });
            object.insert(row.section, read);
        }
        Ok(parsed(object).unwrap_or_else(|problem| {
            log::warn!("{problem}");
            AppSettings::default()
        }))
    }

    pub async fn write_settings(&self, settings: &AppSettings) -> Result<()> {
        let values = serde_json::to_value(settings)?;
        let mut tx = self.pool.begin().await?;
        for section in SETTINGS_SECTIONS {
            let value = values[section].to_string();
            sqlx::query!(
                "INSERT INTO settings (section, value) VALUES (?, ?) ON CONFLICT (section) DO UPDATE SET value = excluded.value",
                section,
                value
            )
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn patch(value: Value) -> SettingsPatch {
        serde_json::from_value(value).unwrap()
    }

    #[tokio::test]
    async fn starts_with_the_defaults() {
        let db = Database::in_memory().await.unwrap();
        assert_eq!(db.read_settings().await.unwrap(), AppSettings::default());
    }

    #[tokio::test]
    async fn keeps_what_was_written() {
        let db = Database::in_memory().await.unwrap();
        let next = merged_settings(
            &AppSettings::default(),
            &patch(json!({ "app": { "confirmExit": false } })),
        )
        .unwrap();
        db.write_settings(&next).await.unwrap();
        assert!(!db.read_settings().await.unwrap().app.confirm_exit);
    }

    #[test]
    fn forgets_the_last_folder_when_the_default_folder_changes() {
        let with_last = merged_settings(
            &AppSettings::default(),
            &patch(json!({ "projects": { "lastDir": "/a" } })),
        )
        .unwrap();
        let projects = merged_settings(
            &with_last,
            &patch(json!({ "projects": { "defaultDir": "/b" } })),
        )
        .unwrap()
        .projects;
        assert_eq!(
            (projects.default_dir.as_str(), projects.last_dir.as_str()),
            ("/b", "")
        );
    }

    #[tokio::test]
    async fn falls_back_to_the_defaults_when_a_saved_section_cant_be_read() {
        let db = Database::in_memory().await.unwrap();
        sqlx::query!(
            "INSERT INTO settings (section, value) VALUES ('app', '{\"onStart\":\"nowhere\"}')"
        )
        .execute(&db.pool)
        .await
        .unwrap();
        assert_eq!(db.read_settings().await.unwrap(), AppSettings::default());
    }

    #[test]
    fn refuses_a_patch_out_of_range() {
        assert!(
            merged_settings(
                &AppSettings::default(),
                &patch(json!({ "traces": { "keepLast": 0 } }))
            )
            .is_err()
        );
    }
}
