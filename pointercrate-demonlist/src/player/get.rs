use crate::{
    creator::created_by,
    demon::{published_by, verified_by},
    error::{DemonlistError, Result},
    nationality::{Nationality, Subdivision},
    player::{DatabasePlayer, FullPlayer, Player},
    record::approved_records_by,
};
use sqlx::{Error, PgConnection};

// Required until https://github.com/launchbadge/sqlx/pull/108 is merged
struct FetchedPlayer {
    id: i32,
    name: String,
    banned: bool,
    nation: Option<String>,
    iso_country_code: Option<String>,
    subdivision_name: Option<String>,
    subdivision_code: Option<String>,
}

impl Player {
    pub async fn upgrade(self, connection: &mut PgConnection) -> Result<FullPlayer> {
        let records = approved_records_by(&self.base, connection).await?;
        let published = published_by(&self.base, connection).await?;
        let verified = verified_by(&self.base, connection).await?;
        let created = created_by(self.base.id, connection).await?;

        Ok(FullPlayer {
            player: self,
            records,
            created,
            verified,
            published,
        })
    }

    pub async fn by_id(id: i32, connection: &mut PgConnection) -> Result<Player> {
        let result = sqlx::query_as!(
            FetchedPlayer,
            r#"SELECT id, players.name AS "name: String", banned, nationalities.nation::text, iso_country_code::text, iso_code::text as subdivision_code, subdivisions.name::text as subdivision_name FROM players LEFT OUTER JOIN nationalities ON 
             players.nationality = nationalities.iso_country_code LEFT OUTER JOIN subdivisions ON players.subdivision = subdivisions.iso_code WHERE id = $1 AND (subdivisions.nation=nationalities.iso_country_code or players.subdivision is null)"#,
            id
        )
        .fetch_one(connection)
        .await;

        match result {
            Ok(row) => {
                let nationality = if let (Some(nation), Some(iso_country_code)) = (row.nation, row.iso_country_code) {
                    Some(Nationality {
                        iso_country_code,
                        nation,
                        subdivision: if let (Some(subdivision), Some(subdivision_code)) = (row.subdivision_name, row.subdivision_code) {
                            Some(Subdivision {
                                iso_code: subdivision_code,
                                name: subdivision,
                            })
                        } else {
                            None
                        },
                    })
                } else {
                    None
                };
                Ok(Player {
                    base: DatabasePlayer {
                        id: row.id,
                        name: row.name,
                        banned: row.banned,
                    },
                    nationality,
                })
            },
            Err(Error::RowNotFound) => Err(DemonlistError::PlayerNotFound { player_id: id }),
            Err(err) => Err(err.into()),
        }
    }
}

impl DatabasePlayer {
    pub async fn by_name(name: &str, connection: &mut PgConnection) -> Result<DatabasePlayer> {
        let name = name.trim();

        let result = sqlx::query!(
            "SELECT id, name::text, banned FROM players WHERE name = cast($1::text as citext)",
            name.to_string()
        ) // FIXME(sqlx) once CITEXT is supported
        .fetch_one(connection)
        .await;

        match result {
            Ok(row) =>
                Ok(DatabasePlayer {
                    id: row.id,
                    name: row.name.unwrap(), // FIXME(sqlx) casted columns interpreted as nullable
                    banned: row.banned,
                }),
            Err(Error::RowNotFound) =>
                Err(DemonlistError::PlayerNotFoundName {
                    player_name: name.to_string(),
                }),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn by_id(id: i32, connection: &mut PgConnection) -> Result<DatabasePlayer> {
        let result = sqlx::query!(r#"SELECT id, name as "name: String", banned FROM players WHERE id = $1"#, id)
            .fetch_one(connection)
            .await;

        match result {
            Ok(row) =>
                Ok(DatabasePlayer {
                    id: row.id,
                    name: row.name,
                    banned: row.banned,
                }),
            Err(Error::RowNotFound) => Err(DemonlistError::PlayerNotFound { player_id: id }),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn by_name_or_create(name: &str, connection: &mut PgConnection) -> Result<DatabasePlayer> {
        let name = name.trim();

        match Self::by_name(name, connection).await {
            Err(DemonlistError::PlayerNotFoundName { .. }) => {
                let id = sqlx::query!("INSERT INTO players (name) VALUES ($1::text) RETURNING id", name.to_string())
                    .fetch_one(connection)
                    .await?
                    .id;

                Ok(DatabasePlayer {
                    id,
                    name: name.to_owned(),
                    banned: false,
                })
            },
            result => result,
        }
    }
}
