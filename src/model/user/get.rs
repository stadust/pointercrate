use crate::{error::PointercrateError, model::user::User, permissions::Permissions, Result};
use sqlx::{Error, PgConnection};

macro_rules! construct_from_row {
    ($row:expr) => {
        User {
            id: $row.member_id,
            name: $row.name,
            permissions: Permissions::from_bits_truncate($row.permissions.unwrap() as u16),
            display_name: $row.display_name,
            youtube_channel: $row.youtube_channel,
            claimed_player: match ($row.cp_id, $row.cp_name, $row.cp_banned) {
                (Some(id), Some(name), Some(banned)) =>
                    Some(crate::model::user::DatabasePlayer {
                        id,
                        name: crate::cistring::CiString(name),
                        banned,
                    }),
                _ => None,
            },
        }
    };
}

macro_rules! query_user {
    ($connection: expr, $query:expr, $id: expr, $($param: expr),*) => {{
        let row = sqlx::query!($query, $id, $($param),*).fetch_one($connection).await;

        match row {
            Err(Error::RowNotFound) =>
                Err(PointercrateError::ModelNotFound {
                    model: "User",
                    identified_by: $id.to_string(),
                }),
            Err(err) => Err(err.into()),
            Ok(row) => Ok(construct_from_row!(row)),
        }
    }};
}

impl User {
    pub async fn by_id(id: i32, connection: &mut PgConnection) -> Result<User> {
        query_user!(
            connection,
            r#"SELECT member_id, members.name, permissions::integer, display_name, youtube_channel::text, players.id as "cp_id?", 
             players.name::text as cp_name, players.banned as "cp_banned?" FROM members LEFT OUTER JOIN players ON players.id = 
             members.claimed_player WHERE member_id = $1"#,
            id,
        )
    }

    pub async fn by_name(name: &str, connection: &mut PgConnection) -> Result<User> {
        query_user!(
            connection,
            r#"SELECT member_id, members.name, CAST(permissions AS integer), display_name, youtube_channel::text, players.id as "cp_id?", 
             players.name::text as cp_name, players.banned as "cp_banned?" FROM members LEFT OUTER JOIN players ON players.id = 
             members.claimed_player WHERE members.name = $1"#,
            name,
        )
    }
}
