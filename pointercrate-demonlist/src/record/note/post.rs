use crate::{
    error::{DemonlistError, Result},
    record::{note::Note, FullRecord},
};
use serde::Deserialize;
use sqlx::PgConnection;

#[derive(Deserialize, Debug)]
pub struct NewNote {
    content: String,

    #[serde(default)]
    is_public: bool,
}

impl Note {
    /// Creates a new note on the given records
    ///
    /// This does **not** insert the note into the records `notes` vector! Also doesn't set the
    /// `author` field!
    pub async fn create_on(record: &FullRecord, new_note: NewNote, connection: &mut PgConnection) -> Result<Note> {
        if new_note.content.trim().is_empty() {
            return Err(DemonlistError::NoteEmpty);
        }

        let note_id = sqlx::query!(
            "INSERT INTO record_notes (record, content, is_public) VALUES ($1, $2, $3) RETURNING id",
            record.id,
            new_note.content,
            new_note.is_public,
        )
        .fetch_one(connection)
        .await?
        .id;

        Ok(Note {
            id: note_id,
            record: record.id,
            content: new_note.content,
            is_public: new_note.is_public,
            transferred: false,
            author: None,
            editors: vec![],
        })
    }
}
