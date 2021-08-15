use crate::{
    error::Result,
    nationality::{Continent, Nationality},
    player::{DatabasePlayer, Player, RankedPlayer},
};
use futures::StreamExt;
use pointercrate_core::{
    error::CoreError,
    util::{non_nullable, nullable},
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgConnection, Row};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerPagination {
    #[serde(default, deserialize_with = "non_nullable")]
    #[serde(rename = "before")]
    pub before_id: Option<i32>,

    #[serde(default, deserialize_with = "non_nullable")]
    #[serde(rename = "after")]
    pub after_id: Option<i32>,

    #[serde(default, deserialize_with = "non_nullable")]
    pub limit: Option<u8>,

    #[serde(default, deserialize_with = "non_nullable")]
    name: Option<String>,

    #[serde(default, deserialize_with = "non_nullable")]
    name_contains: Option<String>,

    #[serde(default, deserialize_with = "non_nullable")]
    pub banned: Option<bool>,

    #[serde(default, deserialize_with = "nullable")]
    nation: Option<Option<String>>,
}

impl PlayerPagination {
    pub async fn page(&self, connection: &mut PgConnection) -> Result<Vec<Player>> {
        if let Some(limit) = self.limit {
            if limit < 1 || limit > 100 {
                Err(CoreError::InvalidPaginationLimit)?
            }
        }

        let order = if self.after_id.is_none() && self.before_id.is_some() {
            "DESC"
        } else {
            "ASC"
        };

        let query = format!(include_str!("../../sql/paginate_players_by_id.sql"), order);

        // FIXME(sqlx) once CITEXT is supported
        let mut stream = sqlx::query(&query)
            .bind(self.before_id)
            .bind(self.after_id)
            .bind(self.name.as_ref().map(|s| s.as_str()))
            .bind(self.name_contains.as_ref().map(|s| s.as_str()))
            .bind(self.banned)
            .bind(&self.nation)
            .bind(self.nation == Some(None))
            .bind(self.limit.unwrap_or(50) as i32 + 1)
            .fetch(connection);

        let mut players = Vec::new();

        while let Some(row) = stream.next().await {
            let row = row?;

            let nationality = match (row.get("nation"), row.get("iso_country_code")) {
                (Some(nation), Some(country_code)) =>
                    Some(Nationality {
                        iso_country_code: country_code,
                        nation,
                        subdivision: None, // dont include subdivision in pagination data
                    }),
                _ => None,
            };

            players.push(Player {
                base: DatabasePlayer {
                    id: row.get("id"),
                    name: row.get("name"),
                    banned: row.get("banned"),
                },
                nationality,
            })
        }

        Ok(players)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RankingPagination {
    #[serde(default, deserialize_with = "non_nullable")]
    #[serde(rename = "before")]
    pub before_index: Option<i64>,

    #[serde(default, deserialize_with = "non_nullable")]
    #[serde(rename = "after")]
    pub after_index: Option<i64>,

    #[serde(default, deserialize_with = "non_nullable")]
    pub limit: Option<u8>,

    #[serde(default, deserialize_with = "nullable")]
    nation: Option<Option<String>>,

    #[serde(default, deserialize_with = "non_nullable")]
    continent: Option<Continent>,

    #[serde(default, deserialize_with = "non_nullable")]
    subdivision: Option<String>,

    #[serde(default, deserialize_with = "non_nullable")]
    name_contains: Option<String>,
}

impl RankingPagination {
    pub async fn page(&self, connection: &mut PgConnection) -> Result<Vec<RankedPlayer>> {
        if let Some(limit) = self.limit {
            if limit < 1 || limit > 100 {
                Err(CoreError::InvalidPaginationLimit)?
            }
        }

        let order = if self.before_index.is_some() && self.after_index.is_none() {
            "DESC"
        } else {
            "ASC"
        };

        let query = format!(include_str!("../../sql/paginate_player_ranking.sql"), order);

        let mut stream = sqlx::query(&query)
            .bind(self.before_index)
            .bind(self.after_index)
            .bind(self.name_contains.as_ref().map(|s| s.as_str()))
            .bind(&self.nation)
            .bind(self.nation == Some(None))
            .bind(self.continent.as_ref().map(|c| c.to_sql()))
            .bind(&self.subdivision)
            .bind(self.limit.unwrap_or(50) as i32 + 1)
            .fetch(connection);

        let mut players = Vec::new();

        while let Some(row) = stream.next().await {
            let row = row?;

            let nationality = match (row.get("nation"), row.get("iso_country_code")) {
                (Some(nation), Some(country_code)) =>
                    Some(Nationality {
                        iso_country_code: country_code,
                        nation,
                        subdivision: None, // dont include subdivision in pagination data
                    }),
                _ => None,
            };

            players.push(RankedPlayer {
                id: row.get("id"),
                name: row.get("name"),
                rank: row.get("rank"),
                nationality,
                score: row.get("score"),
                index: row.get("index"),
            })
        }

        Ok(players)
    }
}
