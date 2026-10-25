use crate::db::{Database, Result};

impl Database {
    pub fn font_rows(&self) -> Result<Vec<(String, String)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT fontId, def FROM fonts ORDER BY id")?;
        let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
        Ok(rows.collect::<std::result::Result<_, _>>()?)
    }

    pub fn save_font_row(&self, font_id: &str, def: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO fonts (fontId, def) VALUES (?1, ?2) ON CONFLICT(fontId) DO UPDATE SET def = excluded.def",
            rusqlite::params![font_id, def],
        )?;
        Ok(())
    }

    pub fn remove_font_row(&self, font_id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM fonts WHERE fontId = ?1", [font_id])?;
        Ok(())
    }
}
