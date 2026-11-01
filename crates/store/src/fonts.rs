use crate::db::{Database, Result};

#[derive(Debug)]
pub struct FontRow {
    pub font_id: String,
    pub def: String,
}

impl Database {
    pub async fn font_rows(&self) -> Result<Vec<FontRow>> {
        Ok(
            sqlx::query_as!(FontRow, "SELECT font_id, def FROM fonts ORDER BY id")
                .fetch_all(&self.pool)
                .await?,
        )
    }

    pub async fn save_font_row(&self, font_id: &str, def: &str) -> Result<()> {
        sqlx::query!(
            "INSERT INTO fonts (font_id, def) VALUES (?, ?) ON CONFLICT (font_id) DO UPDATE SET def = excluded.def",
            font_id,
            def
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn remove_font_row(&self, font_id: &str) -> Result<()> {
        sqlx::query!("DELETE FROM fonts WHERE font_id = ?", font_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
