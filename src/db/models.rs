use rusqlite::{Result, Row};

#[derive(Clone, Debug)]
pub struct ClipboardEntry {
    pub id: i64,
    pub content_type: String,
    pub text_content: Option<String>,
    pub created_at: String,
    pub is_pinned: bool,
}

impl ClipboardEntry {
    pub(crate) fn from_row(row: &Row<'_>) -> Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            content_type: row.get(1)?,
            text_content: row.get(2)?,
            created_at: row.get(3)?,
            is_pinned: row.get(4)?,
        })
    }

    pub fn preview(&self) -> String {
        let text = self
            .text_content
            .as_deref()
            .unwrap_or_default()
            .replace(['\n', '\r'], " ");
        if text.chars().count() > 100 {
            format!("{}…", text.chars().take(100).collect::<String>())
        } else {
            text
        }
    }
}
