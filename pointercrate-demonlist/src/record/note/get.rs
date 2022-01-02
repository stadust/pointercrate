use crate::{
    error::{DemonlistError, Result},
    record::note::Note,
};
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
             WHERE id = $1 AND content IS NOT NULL",
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
    pub async fn by_id(record_id: i32, note_id: i32, connection: &mut PgConnection) -> Result<Note> {
        // TODO: handling of deleted users
        let row = sqlx::query_as!(
            PartialNote,
            r#"SELECT id, record, content, members.name AS "author?: String", EXISTS(SELECT 1 FROM record_notes_modifications WHERE record IS NOT NULL 
             AND id = $1) AS "transferred!: bool" FROM record_notes NATURAL JOIN record_notes_additions LEFT OUTER JOIN members on 
             members.member_id = record_notes_additions.userid WHERE id = $1 and record = $2"#,
            note_id, record_id
        )
            .fetch_one(&mut *connection)
            .await;

        match row {
            Err(Error::RowNotFound) => Err(DemonlistError::NoteNotFound { note_id, record_id }),
            Err(err) => Err(err.into()),
            Ok(row) => row.upgrade(connection).await,
        }
    }
}

pub async fn notes_on(record_id: i32, connection: &mut PgConnection) -> Result<Vec<Note>> {
    let partials = sqlx::query_as!(
        PartialNote,
        r#"SELECT id, record, content, members.name AS "author?: String", EXISTS(SELECT 1 FROM record_notes_modifications WHERE record IS NOT NULL AND 
         id = $1) AS "transferred!: bool"  FROM record_notes NATURAL JOIN record_notes_additions LEFT OUTER JOIN members on members.member_id = 
         record_notes_additions.userid WHERE record = $1"#,
        record_id
    )
        .fetch_all(&mut *connection)
        .await?;

    let mut notes = Vec::new();

    for partial in partials {
        notes.push(partial.upgrade(connection).await?)
    }

    Ok(notes)
}
