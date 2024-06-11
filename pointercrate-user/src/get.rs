use crate::{
    error::{Result, UserError},
    User,
};
use sqlx::{Error, PgConnection};

macro_rules! construct_from_row {
    ($row:expr) => {
        User {
            id: $row.member_id,
            name: $row.name,
            permissions: $row.permissions.unwrap() as u16,
            display_name: $row.display_name,
            youtube_channel: $row.youtube_channel,
        }
    };
}

impl User {
    pub async fn by_id(id: i32, connection: &mut PgConnection) -> Result<User> {
        let row = sqlx::query!(
            r#"SELECT member_id, members.name, permissions::integer, display_name, youtube_channel::text FROM members WHERE member_id = $1"#,
            id
        )
        .fetch_one(connection)
        .await;

        match row {
            Err(Error::RowNotFound) => Err(UserError::UserNotFound { user_id: id }),
            Err(err) => Err(err.into()),
            Ok(row) => Ok(construct_from_row!(row)),
        }
    }

    pub async fn by_google_account(id: &str, connection: &mut PgConnection) -> Result<User> {
        let row = sqlx::query!(
            r#"SELECT member_id, members.name, CAST(permissions AS integer), display_name, youtube_channel::text FROM members WHERE google_account_id = $1"#,
            id
        )
        .fetch_one(connection)
        .await;

        match row {
            Err(Error::RowNotFound) => Err(UserError::UserNotFoundGoogleAccount {
                google_account_id: id.to_string(),
            }),
            Err(err) => Err(err.into()),
            Ok(row) => Ok(construct_from_row!(row)),
        }
    }

    pub async fn by_name(name: &str, connection: &mut PgConnection) -> Result<User> {
        let row = sqlx::query!(
            r#"SELECT member_id, members.name, CAST(permissions AS integer), display_name, youtube_channel::text FROM members WHERE members.name = $1"#,
            name
        )
        .fetch_one(connection)
        .await;

        match row {
            Err(Error::RowNotFound) => Err(UserError::UserNotFoundName {
                user_name: name.to_string(),
            }),
            Err(err) => Err(err.into()),
            Ok(row) => Ok(construct_from_row!(row)),
        }
    }
}
