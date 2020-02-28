use crate::{error::PointercrateError, model::demonlist::record::note::Note, Result};
use futures::StreamExt;
use sqlx::{Error, PgConnection};

struct PartialNote {
    id: i32,
    record: i32,
    content: String,
    author: Option<String>,
    transferred: bool,
}

impl PartialNote {
    async fn upgrade(self, connection: &mut PgConnection) -> Result<Note> {
        let mut stream = sqlx::query!(
            "SELECT members.name AS name FROM record_notes_modifications AS rnm INNER JOIN members ON members.member_id = rnm.userid \
             WHERE id = $1",
            self.id
        )
        .fetch(connection);

        let mut editors = Vec::new();

        while let Some(row) = stream.next().await {
            editors.push(row?.name)
        }

        Ok(Note {
            id: self.id,
            record: self.record,
            content: self.content,
            author: self.author,
            transferred: self.transferred,
            editors,
        })
    }
}

impl Note {
    pub async fn by_id(note_id: i32, connection: &mut PgConnection) -> Result<Note> {
        // TODO: handling of deleted users
        let row = sqlx::query_as!(
            PartialNote,
            "SELECT id, record, content, members.name AS author, EXISTS(SELECT 1 FROM record_notes_modifications WHERE record IS NOT NULL \
             AND id = $1) AS transferred FROM record_notes NATURAL JOIN record_notes_additions LEFT OUTER JOIN members on \
             members.member_id = record_notes_additions.userid WHERE id = $1",
            note_id
        )
        .fetch_one(connection)
        .await;

        match row {
            Err(Error::NotFound) =>
                Err(PointercrateError::ModelNotFound {
                    model: "Note",
                    identified_by: note_id.to_string(),
                }),
            Err(err) => Err(err.into()),
            Ok(row) => row.upgrade(connection).await,
        }
    }
}

pub async fn notes_on(record_id: i32, connection: &mut PgConnection) -> Result<Vec<Note>> {
    let partials = sqlx::query_as!(
        PartialNote,
        "SELECT id, record, content, members.name AS author, EXISTS(SELECT 1 FROM record_notes_modifications WHERE record IS NOT NULL AND \
         id = $1) AS transferred  FROM record_notes NATURAL JOIN record_notes_additions LEFT OUTER JOIN members on members.member_id = \
         record_notes_additions.userid WHERE record = $1",
        record_id
    )
    .fetch_all(connection)
    .await?;

    let mut notes = Vec::new();

    for partial in partials {
        notes.push(partial.upgrade(connection).await?)
    }

    Ok(notes)
}
