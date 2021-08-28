use crate::{
    creator::creators_of,
    demon::{Demon, FullDemon, MinimalDemon, TimeShiftedDemon},
    error::{DemonlistError, Result},
    player::DatabasePlayer,
    record::approved_records_on,
};
use chrono::{DateTime, FixedOffset};
use futures::StreamExt;
use sqlx::{Error, PgConnection};

impl MinimalDemon {
    pub async fn by_id(id: i32, connection: &mut PgConnection) -> Result<MinimalDemon> {
        let row = sqlx::query!(r#"SELECT id, name as "name: String", position FROM demons WHERE id = $1"#, id)
            .fetch_one(connection)
            .await?;

        Ok(MinimalDemon {
            id,
            position: row.position,
            name: row.name,
        })
    }

    pub async fn by_name(name: &str, connection: &mut PgConnection) -> Result<MinimalDemon> {
        let mut stream = sqlx::query!(
            r#"SELECT id, name as "name: String", position FROM demons WHERE name = cast($1::text as citext)"#, // FIXME(sqlx) once CITEXT is supported
            name.to_string()
        )
        .fetch(connection);

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
                None =>
                    Err(DemonlistError::DemonNotFoundName {
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
            .map_err(|err| {
                match err {
                    Error::RowNotFound => DemonlistError::DemonNotFound { demon_id: id },
                    _ => err.into(),
                }
            })
    }

    pub async fn by_position(position: i16, connection: &mut PgConnection) -> Result<Demon> {
        sqlx::query_file_as!(FetchedDemon, "sql/demon_by_position.sql", position)
            .fetch_one(connection)
            .await
            .map(Into::into)
            .map_err(|err| {
                match err {
                    Error::RowNotFound => DemonlistError::DemonNotFoundPosition { demon_position: position },
                    _ => err.into(),
                }
            })
    }
}

macro_rules! query_many_demons {
    ($connection:expr, $query:expr, $id:expr) => {{
        let mut stream = sqlx::query!($query, $id).fetch($connection);
        let mut demons = Vec::new();

        while let Some(row) = stream.next().await {
            let row = row?;

            demons.push(MinimalDemon {
                id: row.id,
                position: row.position,
                name: row.name,
            })
        }

        Ok(demons)
    }};
}

pub async fn published_by(player: &DatabasePlayer, connection: &mut PgConnection) -> Result<Vec<MinimalDemon>> {
    query_many_demons!(
        connection,
        r#"SELECT id, name AS "name: String", position FROM demons WHERE publisher = $1"#,
        player.id
    )
}

pub async fn verified_by(player: &DatabasePlayer, connection: &mut PgConnection) -> Result<Vec<MinimalDemon>> {
    query_many_demons!(
        connection,
        r#"SELECT id, name as "name: String", position FROM demons WHERE verifier = $1"#,
        player.id
    )
}

struct FetchedDemon {
    demon_id: i32,
    demon_name: String,
    position: i16,
    requirement: i16,
    video: Option<String>,
    publisher_id: i32,
    publisher_name: String,
    publisher_banned: bool,
    verifier_id: i32,
    verifier_name: String,
    verifier_banned: bool,
    level_id: Option<i64>,
}

impl Into<Demon> for FetchedDemon {
    fn into(self) -> Demon {
        Demon {
            base: MinimalDemon {
                id: self.demon_id,
                name: self.demon_name,
                position: self.position,
            },
            requirement: self.requirement,
            video: self.video,
            publisher: DatabasePlayer {
                id: self.publisher_id,
                name: self.publisher_name,
                banned: self.publisher_banned,
            },
            verifier: DatabasePlayer {
                id: self.verifier_id,
                name: self.verifier_name,
                banned: self.verifier_banned,
            },
            level_id: self.level_id.map(|id| id as u64),
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

pub async fn list_at(connection: &mut PgConnection, at: DateTime<FixedOffset>) -> Result<Vec<TimeShiftedDemon>> {
    let mut stream = sqlx::query_file!("sql/all_demons_at.sql", at.naive_utc()).fetch(connection);
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
