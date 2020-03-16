use crate::{
    cistring::{CiStr, CiString},
    error::PointercrateError,
    model::{
        demonlist::{
            creator::created_by,
            demon::{published_by, verified_by},
            player::{DatabasePlayer, FullPlayer, Player},
            record::approved_records_by,
        },
        nationality::Nationality,
    },
    Result,
};
use sqlx::{Error, PgConnection};

// Required until https://github.com/launchbadge/sqlx/pull/108 is merged
struct FetchedPlayer {
    id: i32,
    name: String,
    banned: bool,
    nation: Option<String>,
    iso_country_code: Option<String>,
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
            "SELECT id, name::text, banned, nation::text, iso_country_code::text FROM players LEFT OUTER JOIN nationalities ON \
             players.nationality = nationalities.iso_country_code WHERE id = $1",
            id
        )
        .fetch_one(connection)
        .await;

        match result {
            Ok(row) => {
                let nationality = if let (Some(nation), Some(iso_country_code)) = (row.nation, row.iso_country_code) {
                    Some(Nationality {
                        iso_country_code,
                        nation: CiString(nation),
                    })
                } else {
                    None
                };
                Ok(Player {
                    base: DatabasePlayer {
                        id: row.id,
                        name: CiString(row.name),
                        banned: row.banned,
                    },
                    nationality,
                })
            },
            Err(Error::NotFound) =>
                Err(PointercrateError::ModelNotFound {
                    model: "Player",
                    identified_by: id.to_string(),
                }),
            Err(err) => Err(err.into()),
        }
    }
}

impl DatabasePlayer {
    pub async fn by_name(name: &CiStr, connection: &mut PgConnection) -> Result<DatabasePlayer> {
        let name = CiStr::from_str(name.trim());

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
                    name: CiString(row.name),
                    banned: row.banned,
                }),
            Err(Error::NotFound) =>
                Err(PointercrateError::ModelNotFound {
                    model: "Player",
                    identified_by: name.to_string(),
                }),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn by_id(id: i32, connection: &mut PgConnection) -> Result<DatabasePlayer> {
        let result = sqlx::query!("SELECT id, name::text, banned FROM players WHERE id = $1", id)
            .fetch_one(connection)
            .await;

        match result {
            Ok(row) =>
                Ok(DatabasePlayer {
                    id: row.id,
                    name: CiString(row.name),
                    banned: row.banned,
                }),
            Err(Error::NotFound) =>
                Err(PointercrateError::ModelNotFound {
                    model: "Player",
                    identified_by: id.to_string(),
                }),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn by_name_or_create(name: &CiStr, connection: &mut PgConnection) -> Result<DatabasePlayer> {
        let name = CiStr::from_str(name.trim());

        match Self::by_name(name, connection).await {
            Err(PointercrateError::ModelNotFound { .. }) => {
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
