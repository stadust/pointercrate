use crate::{
    cistring::{CiStr, CiString},
    error::PointercrateError,
    model::demonlist::{
        creator::creators_of,
        demon::{Demon, FullDemon, MinimalDemon},
        player::DatabasePlayer,
        record::approved_records_on,
    },
    Result,
};
use futures::StreamExt;
use sqlx::{Error, PgConnection};

impl MinimalDemon {
    pub async fn by_id(id: i32, connection: &mut PgConnection) -> Result<MinimalDemon> {
        let row = sqlx::query!("SELECT id, name::text, position FROM demons WHERE id = $1", id)
            .fetch_one(connection)
            .await?;

        Ok(MinimalDemon {
            id,
            position: row.position,
            name: CiString(row.name),
        })
    }

    pub async fn by_name(name: &CiStr, connection: &mut PgConnection) -> Result<MinimalDemon> {
        let mut stream = sqlx::query!(
            "SELECT id, name::text, position FROM demons WHERE name = cast($1::text as citext)", // FIXME(sqlx) once CITEXT is supported
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
                name: CiString(row.name),
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
                    Err(PointercrateError::ModelNotFound {
                        model: "Demon",
                        identified_by: name.to_string(),
                    }),
            }
        } else {
            further_demons.extend(demon);

            Err(PointercrateError::DemonNameNotUnique { demons: further_demons })
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
                    Error::NotFound =>
                        PointercrateError::ModelNotFound {
                            model: "Demon",
                            identified_by: id.to_string(),
                        },
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
                    Error::NotFound =>
                        PointercrateError::ModelNotFound {
                            model: "Demon",
                            identified_by: position.to_string(),
                        },
                    _ => err.into(),
                }
            })
    }
}

pub async fn published_by(player: &DatabasePlayer, connection: &mut PgConnection) -> Result<Vec<MinimalDemon>> {
    let mut stream = sqlx::query!("SELECT id, name::text, position FROM demons WHERE publisher = $1", player.id).fetch(connection);

    let mut demons = Vec::new();

    while let Some(row) = stream.next().await {
        let row = row?;

        demons.push(MinimalDemon {
            id: row.id,
            position: row.position,
            name: CiString(row.name),
        })
    }

    Ok(demons)
}

pub async fn verified_by(player: &DatabasePlayer, connection: &mut PgConnection) -> Result<Vec<MinimalDemon>> {
    let mut stream = sqlx::query!("SELECT id, name::text, position FROM demons WHERE verifier = $1", player.id).fetch(connection);

    let mut demons = Vec::new();

    while let Some(row) = stream.next().await {
        let row = row?;

        demons.push(MinimalDemon {
            id: row.id,
            position: row.position,
            name: CiString(row.name),
        })
    }

    Ok(demons)
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
}

impl Into<Demon> for FetchedDemon {
    fn into(self) -> Demon {
        Demon {
            base: MinimalDemon {
                id: self.demon_id,
                name: CiString(self.demon_name),
                position: self.position,
            },
            requirement: self.requirement,
            video: self.video,
            publisher: DatabasePlayer {
                id: self.publisher_id,
                name: CiString(self.publisher_name),
                banned: self.publisher_banned,
            },
            verifier: DatabasePlayer {
                id: self.verifier_id,
                name: CiString(self.verifier_name),
                banned: self.verifier_banned,
            },
        }
    }
}
