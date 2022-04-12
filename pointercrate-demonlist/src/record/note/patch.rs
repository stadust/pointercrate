use crate::{
    error::{DemonlistError, Result},
    record::note::Note,
};
use pointercrate_core::util::non_nullable;
use serde::Deserialize;
use sqlx::PgConnection;

#[derive(Debug, Deserialize)]
pub struct PatchNote {
    #[serde(default, deserialize_with = "non_nullable")]
    pub content: Option<String>,

    #[serde(default, deserialize_with = "non_nullable")]
    pub is_public: Option<bool>,
}

impl Note {
    pub async fn apply_patch(mut self, patch: PatchNote, connection: &mut PgConnection) -> Result<Note> {
        if let Some(content) = patch.content {
            if content.trim().is_empty() {
                return Err(DemonlistError::NoteEmpty)
            }

            sqlx::query!("UPDATE record_notes SET content = $1 WHERE id = $2", content, self.id)
                .execute(&mut *connection)
                .await?;

            self.content = content;
        }

        if let Some(is_public) = patch.is_public {
            sqlx::query!("UPDATE record_notes SET is_public = $1 WHERE id = $2", is_public, self.id)
                .execute(connection)
                .await?;

            self.is_public = is_public;
        }

        Ok(self)
    }
}
