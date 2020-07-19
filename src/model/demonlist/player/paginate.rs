use crate::{
    cistring::CiString,
    model::{
        demonlist::player::{DatabasePlayer, Player, RankedPlayer},
        nationality::Nationality,
    },
    util::{non_nullable, nullable},
    Result,
};
use futures::StreamExt;
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
    name: Option<CiString>,

    #[serde(default, deserialize_with = "non_nullable")]
    name_contains: Option<CiString>,

    #[serde(default, deserialize_with = "non_nullable")]
    banned: Option<bool>,

    #[serde(default, deserialize_with = "nullable")]
    nation: Option<Option<String>>,
}

impl PlayerPagination {
    pub async fn page(&self, connection: &mut PgConnection) -> Result<Vec<Player>> {
        let order = if self.after_id.is_none() && self.before_id.is_some() {
            "DESC"
        } else {
            "ASC"
        };

        let query = format!(include_str!("../../../../sql/paginate_players_by_id.sql"), order);

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
                        nation: CiString(nation),
                    }),
                _ => None,
            };

            players.push(Player {
                base: DatabasePlayer {
                    id: row.get("id"),
                    name: CiString(row.get("name")),
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
    name_contains: Option<CiString>,
}

impl RankingPagination {
    pub async fn page(&self, connection: &mut PgConnection) -> Result<Vec<RankedPlayer>> {
        let order = if self.before_index.is_some() && self.after_index.is_none() {
            "DESC"
        } else {
            "ASC"
        };

        let query = format!(include_str!("../../../../sql/paginate_player_ranking.sql"), order);

        let mut stream = sqlx::query(&query)
            .bind(self.before_index)
            .bind(self.after_index)
            .bind(self.name_contains.as_ref().map(|s| s.as_str()))
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
                        nation: CiString(nation),
                    }),
                _ => None,
            };

            players.push(RankedPlayer {
                id: row.get("id"),
                name: CiString(row.get("name")),
                rank: row.get("rank"),
                nationality,
                score: row.get("score"),
                index: row.get("index"),
            })
        }

        Ok(players)
    }
}
