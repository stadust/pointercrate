use crate::{
    creator::creators_of,
    demon::{Demon, FullDemon, MinimalDemon, TimeShiftedDemon},
    error::{DemonlistError, Result},
    player::DatabasePlayer,
    record::approved_records_on,
};
use chrono::NaiveDateTime;
use futures::StreamExt;
use sqlx::{Error, PgConnection};

impl MinimalDemon {
    pub async fn by_id(id: i32, connection: &mut PgConnection) -> Result<MinimalDemon> {
        sqlx::query_as!(MinimalDemon, r#"SELECT id, name, position FROM demons WHERE id = $1"#, id)
            .fetch_one(connection)
            .await
            .map_err(|err| match err {
                Error::RowNotFound => DemonlistError::DemonNotFound { demon_id: id },
                _ => err.into(),
            })
    }

    pub async fn by_name(name: &str, connection: &mut PgConnection) -> Result<MinimalDemon> {
        let mut stream = sqlx::query!(r#"SELECT id, name, position FROM demons WHERE name = $1"#, name.to_string()).fetch(connection);

        let mut demon = None;
        let mut further_demons = Vec::new();

        while let Some(row) = stream.next().await {
            let row = row?;

            let current_demon = MinimalDemon {
                id: row.id,
                position: row.position,
                name: row.name,
            };

            if demon.is_none() {
                demon = Some(current_demon)
            } else {
                further_demons.push(current_demon)
            }
        }

        if further_demons.is_empty() {
            match demon {
                Some(demon) => Ok(demon),
                None => Err(DemonlistError::DemonNotFoundName {
                    demon_name: name.to_string(),
                }),
            }
        } else {
            further_demons.extend(demon);

            Err(DemonlistError::DemonNameNotUnique { demons: further_demons })
        }
    }
}

impl FullDemon {
    pub async fn by_id(id: i32, connection: &mut PgConnection) -> Result<FullDemon> {
        Demon::by_id(id, connection).await?.upgrade(connection).await
    }

    pub async fn by_position(position: i16, connection: &mut PgConnection) -> Result<FullDemon> {
        Demon::by_position(position, connection).await?.upgrade(connection).await
    }
}

// FIXME: optimally, we want to only have one of these
impl Demon {
    async fn upgrade(self, connection: &mut PgConnection) -> Result<FullDemon> {
        let creators = creators_of(&self.base, connection).await?;
        let records = approved_records_on(&self.base, connection).await?;

        Ok(FullDemon {
            demon: self,
            creators,
            records,
        })
    }

    pub async fn by_id(id: i32, connection: &mut PgConnection) -> Result<Demon> {
        sqlx::query_file_as!(FetchedDemon, "sql/demon_by_id.sql", id)
            .fetch_one(connection)
            .await
            .map(Into::into)
            .map_err(|err| match err {
                Error::RowNotFound => DemonlistError::DemonNotFound { demon_id: id },
                _ => err.into(),
            })
    }

    pub async fn by_position(position: i16, connection: &mut PgConnection) -> Result<Demon> {
        sqlx::query_file_as!(FetchedDemon, "sql/demon_by_position.sql", position)
            .fetch_one(connection)
            .await
            .map(Into::into)
            .map_err(|err| match err {
                Error::RowNotFound => DemonlistError::DemonNotFoundPosition { demon_position: position },
                _ => err.into(),
            })
    }
}

macro_rules! query_many_demons {
    ($connection:expr, $query:expr, $id:expr) => {{
        let mut stream = sqlx::query_as!(MinimalDemon, $query, $id).fetch($connection);
        let mut demons = Vec::new();

        while let Some(row) = stream.next().await {
            demons.push(row?)
        }

        Ok(demons)
    }};
}

pub async fn published_by(player: &DatabasePlayer, connection: &mut PgConnection) -> Result<Vec<MinimalDemon>> {
    query_many_demons!(
        connection,
        r#"SELECT id, name, position FROM demons WHERE publisher = $1"#,
        player.id
    )
}

pub async fn verified_by(player: &DatabasePlayer, connection: &mut PgConnection) -> Result<Vec<MinimalDemon>> {
    query_many_demons!(
        connection,
        r#"SELECT id, name, position FROM demons WHERE verifier = $1"#,
        player.id
    )
}

struct FetchedDemon {
    demon_id: i32,
    demon_name: String,
    position: i16,
    requirement: i16,
    video: Option<String>,
    thumbnail: String,
    publisher_id: i32,
    publisher_name: String,
    publisher_banned: bool,
    verifier_id: i32,
    verifier_name: String,
    verifier_banned: bool,
    level_id: Option<i64>,
}

impl From<FetchedDemon> for Demon {
    fn from(fetched: FetchedDemon) -> Self {
        Demon {
            base: MinimalDemon {
                id: fetched.demon_id,
                name: fetched.demon_name,
                position: fetched.position,
            },
            requirement: fetched.requirement,
            video: fetched.video,
            thumbnail: fetched.thumbnail,
            publisher: DatabasePlayer {
                id: fetched.publisher_id,
                name: fetched.publisher_name,
                banned: fetched.publisher_banned,
            },
            verifier: DatabasePlayer {
                id: fetched.verifier_id,
                name: fetched.verifier_name,
                banned: fetched.verifier_banned,
            },
            level_id: fetched.level_id.map(|id| id as u64),
        }
    }
}

pub async fn current_list(connection: &mut PgConnection) -> Result<Vec<Demon>> {
    Ok(sqlx::query_file_as!(FetchedDemon, "sql/all_demons.sql")
        .fetch_all(connection)
        .await?
        .into_iter()
        .map(Into::into)
        .collect())
}

pub async fn list_at(connection: &mut PgConnection, at: NaiveDateTime) -> Result<Vec<TimeShiftedDemon>> {
    let mut stream = sqlx::query_file!("sql/all_demons_at.sql", at).fetch(connection);
    let mut demons = Vec::new();

    while let Some(row) = stream.next().await {
        let row = row?;

        demons.push(TimeShiftedDemon {
            current_demon: Demon {
                base: MinimalDemon {
                    id: row.demon_id,
                    position: row.position,
                    name: row.demon_name,
                },
                requirement: row.requirement,
                video: row.video,
                thumbnail: row.thumbnail,
                publisher: DatabasePlayer {
                    id: row.publisher_id,
                    name: row.publisher_name,
                    banned: row.publisher_banned,
                },
                verifier: DatabasePlayer {
                    id: row.verifier_id,
                    name: row.verifier_name,
                    banned: row.verifier_banned,
                },
                level_id: row.level_id.map(|i| i as u64),
            },
            position_now: row.current_position,
        })
    }

    Ok(demons)
}
