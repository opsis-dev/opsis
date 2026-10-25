use rusqlite::OptionalExtension;
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
    pub fn saved_bounds(&self, window: &str) -> Result<Option<Bounds>> {
        Ok(self
            .conn
            .query_row(
                "SELECT x, y, width, height FROM window_bounds WHERE window = ?1",
                [window],
                |r| {
                    Ok(Bounds {
                        x: r.get(0)?,
                        y: r.get(1)?,
                        width: r.get(2)?,
                        height: r.get(3)?,
                    })
                },
            )
            .optional()?)
    }

    pub fn save_bounds(&self, window: &str, b: Bounds) -> Result<()> {
        self.conn.execute(
            "INSERT INTO window_bounds (window, x, y, width, height) VALUES (?1, ?2, ?3, ?4, ?5) ON CONFLICT(window) DO UPDATE SET x = excluded.x, y = excluded.y, width = excluded.width, height = excluded.height",
            rusqlite::params![window, b.x, b.y, b.width, b.height],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remembers_the_last_bounds_of_each_window() {
        let db = Database::in_memory().unwrap();
        assert_eq!(db.saved_bounds("settings").unwrap(), None);
        db.save_bounds(
            "settings",
            Bounds {
                x: 1,
                y: 2,
                width: 300,
                height: 400,
            },
        )
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
        .unwrap();
        assert_eq!(
            db.saved_bounds("settings").unwrap(),
            Some(Bounds {
                x: 5,
                y: 6,
                width: 700,
                height: 800
            })
        );
    }
}
