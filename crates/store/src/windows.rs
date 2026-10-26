use serde::{Deserialize, Serialize};

use crate::db::{Database, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct Bounds {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Database {
    pub async fn saved_bounds(&self, window: &str) -> Result<Option<Bounds>> {
        Ok(sqlx::query_as!(
            Bounds,
            r#"SELECT x AS "x: i32", y AS "y: i32", width AS "width: i32", height AS "height: i32" FROM window_bounds WHERE window = ?"#,
            window
        )
        .fetch_optional(&self.pool)
        .await?)
    }

    pub async fn save_bounds(&self, window: &str, b: Bounds) -> Result<()> {
        sqlx::query!(
            "INSERT INTO window_bounds (window, x, y, width, height) VALUES (?, ?, ?, ?, ?) ON CONFLICT (window) DO UPDATE SET x = excluded.x, y = excluded.y, width = excluded.width, height = excluded.height",
            window,
            b.x,
            b.y,
            b.width,
            b.height
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn remembers_the_last_bounds_of_each_window() {
        let db = Database::in_memory().await.unwrap();
        assert_eq!(db.saved_bounds("settings").await.unwrap(), None);
        db.save_bounds(
            "settings",
            Bounds {
                x: 1,
                y: 2,
                width: 300,
                height: 400,
            },
        )
        .await
        .unwrap();
        db.save_bounds(
            "settings",
            Bounds {
                x: 5,
                y: 6,
                width: 700,
                height: 800,
            },
        )
        .await
        .unwrap();
        assert_eq!(
            db.saved_bounds("settings").await.unwrap(),
            Some(Bounds {
                x: 5,
                y: 6,
                width: 700,
                height: 800
            })
        );
    }
}
