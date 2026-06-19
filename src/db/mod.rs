mod migrations;
mod models;

pub use models::ClipboardEntry;

use crate::{
    config::Config,
    utils::{hashing, paths},
};
use anyhow::Result;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct Database {
    path: PathBuf,
}

impl Database {
    pub fn open_default() -> Result<Self> {
        Self::open(paths::database_file()?)
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let db = Self {
            path: path.as_ref().to_path_buf(),
        };
        migrations::run(&db.connection()?)?;
        Ok(db)
    }

    fn connection(&self) -> Result<Connection> {
        let connection = Connection::open(&self.path)?;
        connection.busy_timeout(std::time::Duration::from_secs(2))?;
        Ok(connection)
    }

    pub fn insert_text(&self, text: &str, config: &Config) -> Result<bool> {
        let hash = hashing::sha256(text);
        let connection = self.connection()?;
        let latest: Option<String> = connection.query_row(
            "SELECT content_hash FROM clipboard_entries ORDER BY created_at DESC, id DESC LIMIT 1", [], |row| row.get(0)
        ).optional()?;
        if latest.as_deref() == Some(&hash) {
            return Ok(false);
        }

        connection.execute(
            "INSERT INTO clipboard_entries (content_type, text_content, created_at, content_hash)
             VALUES ('text/plain', ?1, ?2, ?3)
             ON CONFLICT(content_hash) DO UPDATE SET created_at=excluded.created_at",
            params![text, Utc::now().to_rfc3339(), hash],
        )?;
        self.prune(config.max_history_items)?;
        Ok(true)
    }

    pub fn list(&self, query: Option<&str>, limit: usize) -> Result<Vec<ClipboardEntry>> {
        let connection = self.connection()?;
        let pattern = format!("%{}%", query.unwrap_or_default());
        let mut statement = connection.prepare(
            "SELECT id, content_type, text_content, created_at, is_pinned
             FROM clipboard_entries WHERE text_content LIKE ?1
             ORDER BY is_pinned DESC, created_at DESC, id DESC LIMIT ?2",
        )?;
        let rows = statement.query_map(params![pattern, limit as i64], ClipboardEntry::from_row)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn get(&self, id: i64) -> Result<Option<ClipboardEntry>> {
        Ok(self.connection()?.query_row(
            "SELECT id, content_type, text_content, created_at, is_pinned FROM clipboard_entries WHERE id=?1",
            [id], ClipboardEntry::from_row).optional()?)
    }

    pub fn mark_used(&self, id: i64) -> Result<()> {
        self.connection()?.execute(
            "UPDATE clipboard_entries SET last_used_at=?1, use_count=use_count+1 WHERE id=?2",
            params![Utc::now().to_rfc3339(), id],
        )?;
        Ok(())
    }

    pub fn toggle_pin(&self, id: i64) -> Result<()> {
        self.connection()?.execute("UPDATE clipboard_entries SET is_pinned=CASE is_pinned WHEN 0 THEN 1 ELSE 0 END WHERE id=?1", [id])?;
        Ok(())
    }

    pub fn delete(&self, id: i64) -> Result<()> {
        self.connection()?
            .execute("DELETE FROM clipboard_entries WHERE id=?1", [id])?;
        Ok(())
    }
    pub fn clear_unpinned(&self) -> Result<()> {
        self.connection()?
            .execute("DELETE FROM clipboard_entries WHERE is_pinned=0", [])?;
        Ok(())
    }
    pub fn clear_all(&self) -> Result<()> {
        self.connection()?
            .execute("DELETE FROM clipboard_entries", [])?;
        Ok(())
    }

    pub fn prune(&self, max: usize) -> Result<()> {
        self.connection()?.execute(
            "DELETE FROM clipboard_entries WHERE is_pinned=0 AND id NOT IN
             (SELECT id FROM clipboard_entries WHERE is_pinned=0 ORDER BY created_at DESC, id DESC LIMIT ?1)",
            [max as i64])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stores_deduplicates_and_preserves_pins() {
        let path = std::env::temp_dir().join(format!("lch-{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let db = Database::open(&path).unwrap();
        let mut config = Config::default();
        config.max_history_items = 1;
        assert!(db.insert_text("one", &config).unwrap());
        assert!(!db.insert_text("one", &config).unwrap());
        let id = db.list(None, 10).unwrap()[0].id;
        db.toggle_pin(id).unwrap();
        db.insert_text("two", &config).unwrap();
        assert_eq!(db.list(None, 10).unwrap().len(), 2);
        db.clear_unpinned().unwrap();
        assert!(db.list(None, 10).unwrap()[0].is_pinned);
        let _ = std::fs::remove_file(path);
    }
}
