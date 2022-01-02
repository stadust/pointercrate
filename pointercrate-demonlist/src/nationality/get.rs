use crate::{
    demon::MinimalDemon,
    error::{DemonlistError, Result},
    nationality::{BestRecord, MiniDemon, MiniDemonWithPlayers, Nationality, NationalityRecord, Subdivision},
};
use futures::stream::StreamExt;
use sqlx::{Error, PgConnection};

impl Nationality {
    pub async fn subdivisions(&self, connection: &mut PgConnection) -> Result<Vec<Subdivision>> {
        let mut stream = sqlx::query!(
            r#"SELECT iso_code as "iso_code: String", name as "name: String" FROM subdivisions WHERE nation = $1 ORDER BY name"#,
            self.iso_country_code
        )
        .fetch(connection);
        let mut subdivisions = Vec::new();

        while let Some(row) = stream.next().await {
            let row = row?;

            subdivisions.push(Subdivision {
                name: row.name,
                iso_code: row.iso_code,
            })
        }

        Ok(subdivisions)
    }

    pub async fn by_country_code_or_name(code: &str, connection: &mut PgConnection) -> Result<Nationality> {
        sqlx::query!(
            r#"SELECT nation as "nation: String", iso_country_code as "iso_country_code: String" FROM nationalities WHERE iso_country_code = $1 or nation = $1"#,
            code.to_string() /* FIXME(sqlx 0.3) */
        )
        .fetch_one(connection)
        .await
        .map(|row| {
            Nationality {
                nation: row.nation,
                iso_country_code: row.iso_country_code,
                subdivision: None
            }
        })
        .map_err(|sqlx_error| {
            match sqlx_error {
                Error::RowNotFound =>
                    DemonlistError::NationalityNotFound {
                        iso_code: code.to_string(),
                    },
                _ => sqlx_error.into(),
            }
        })
    }

    pub async fn all(connection: &mut PgConnection) -> Result<Vec<Nationality>> {
        let mut stream =
            sqlx::query!(r#"SELECT nation as "nation: String", iso_country_code as "iso_country_code: String" FROM nationalities"#)
                .fetch(connection);
        let mut nationalities = Vec::new();

        while let Some(row) = stream.next().await {
            let row = row?;

            nationalities.push(Nationality {
                nation: row.nation,
                iso_country_code: row.iso_country_code,
                subdivision: None,
            })
        }

        Ok(nationalities)
    }

    pub async fn used(connection: &mut PgConnection) -> Result<Vec<Nationality>> {
        let mut stream = sqlx::query!(
            r#"SELECT DISTINCT nation as "nation: String", iso_country_code as "iso_country_code: String" FROM players INNER JOIN nationalities ON nationality=iso_country_code ORDER BY nation"#
        )
        .fetch(connection);
        let mut nationalities = Vec::new();

        while let Some(row) = stream.next().await {
            let row = row?;

            nationalities.push(Nationality {
                nation: row.nation,
                iso_country_code: row.iso_country_code,
                subdivision: None,
            })
        }

        Ok(nationalities)
    }

    pub async fn upgrade(self, connection: &mut PgConnection) -> Result<NationalityRecord> {
        Ok(NationalityRecord {
            best_records: best_records_in(&self, connection).await?,
            created: created_in(&self, connection).await?,
            verified: verified_in(&self, connection).await?,
            published: published_in(&self, connection).await?,
            unbeaten: unbeaten_in(&self, connection).await?,
            nation: self,
        })
    }
}

pub async fn unbeaten_in(nation: &Nationality, connection: &mut PgConnection) -> Result<Vec<MinimalDemon>> {
    let mut stream = sqlx::query!(
        r#"select name::text as "name!", id as "id!", position as "position!" from demons where position <= $1 except (select demons.name, demons.id, position from records inner join players on 
         players.id=records.player inner join demons on demons.id=records.demon where status_='APPROVED' and nationality=$2 and progress=100 union select demons.name, demons.id, demons.position from demons inner join players on players.id=verifier where players.nationality=$2)"#,
        crate::config::extended_list_size(),
        nation.iso_country_code
    )
    .fetch(connection);

    let mut unbeaten = Vec::new();

    while let Some(row) = stream.next().await {
        let row = row?;

        unbeaten.push(MinimalDemon {
            id: row.id,
            position: row.position,
            name: row.name,
        });
    }

    Ok(unbeaten)
}

pub async fn created_in(nation: &Nationality, connection: &mut PgConnection) -> Result<Vec<MiniDemonWithPlayers>> {
    let mut stream = sqlx::query!( r#"select distinct on (demon) demon, demons.name::text as "demon_name!", demons.position, players.name::text as "player_name!" from creators inner join demons on demons.id=demon inner join players on players.id=creator where nationality=$1"#, nation.iso_country_code).fetch(connection);

    let mut creations = Vec::<MiniDemonWithPlayers>::new();

    while let Some(row) = stream.next().await {
        let row = row?;

        match creations.last_mut() {
            Some(mini_demon) if mini_demon.demon == row.demon_name => mini_demon.players.push(row.player_name),
            _ =>
                creations.push(MiniDemonWithPlayers {
                    id: row.demon,
                    demon: row.demon_name,
                    position: row.position,
                    players: vec![row.player_name],
                }),
        }
    }

    Ok(creations)
}

pub async fn verified_in(nation: &Nationality, connection: &mut PgConnection) -> Result<Vec<MiniDemon>> {
    let mut stream = sqlx::query!(
        r#"select demons.id as demon, demons.name::text as "demon_name!", demons.position, players.name::text as "player_name!" from demons inner join players on players.id=verifier where nationality=$1"#, nation.iso_country_code).fetch(connection);

    let mut demons = Vec::new();

    while let Some(row) = stream.next().await {
        let row = row?;

        demons.push(MiniDemon {
            id: row.demon,
            demon: row.demon_name,
            position: row.position,
            player: row.player_name,
        });
    }

    Ok(demons)
}

pub async fn published_in(nation: &Nationality, connection: &mut PgConnection) -> Result<Vec<MiniDemon>> {
    let mut stream = sqlx::query!(
        r#"select demons.id as demon, demons.name::text as "demon_name!", demons.position, players.name::text as "player_name!" from demons inner join players on players.id=publisher where nationality=$1"#, nation.iso_country_code).fetch(connection);

    let mut demons = Vec::new();

    while let Some(row) = stream.next().await {
        let row = row?;

        demons.push(MiniDemon {
            id: row.demon,
            demon: row.demon_name,
            position: row.position,
            player: row.player_name,
        });
    }

    Ok(demons)
}

pub async fn best_records_in(nation: &Nationality, connection: &mut PgConnection) -> Result<Vec<BestRecord>> {
    let mut stream = sqlx::query!(
        r#"SELECT progress as "progress!", demons.id AS "demon_id!", demons.name as "demon_name!: String", demons.position as "position!", players.name as "player_name!: String" FROM best_records_in($1) as records INNER JOIN demons ON records.demon = demons.id INNER JOIN players ON players.id = records.player"#,
        nation.iso_country_code
    )
        .fetch(connection);

    let mut records = Vec::<BestRecord>::new();

    while let Some(row) = stream.next().await {
        let row = row?;

        match records.last_mut() {
            Some(record) if record.demon == row.demon_name => record.players.push(row.player_name),
            _ =>
                records.push(BestRecord {
                    id: row.demon_id,
                    demon: row.demon_name,
                    position: row.position,
                    progress: row.progress,
                    players: vec![row.player_name],
                }),
        }
    }

    Ok(records)
}
