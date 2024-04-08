use crate::{
    error::Result,
    nationality::{Continent, Nationality},
    player::{DatabasePlayer, Player, RankedPlayer},
};
use futures::StreamExt;
use pointercrate_core::{
    pagination::{Pagination, PaginationParameters},
    util::{non_nullable, nullable},
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgConnection, Row};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerPagination {
    #[serde(flatten)]
    pub params: PaginationParameters,

    #[serde(default, deserialize_with = "non_nullable")]
    name: Option<String>,

    #[serde(default, deserialize_with = "non_nullable")]
    name_contains: Option<String>,

    #[serde(default, deserialize_with = "non_nullable")]
    pub banned: Option<bool>,

    #[serde(default, deserialize_with = "nullable")]
    nation: Option<Option<String>>,
}

impl Pagination for PlayerPagination {
    fn parameters(&self) -> PaginationParameters {
        self.params
    }

    fn with_parameters(&self, parameters: PaginationParameters) -> Self {
        Self {
            params: parameters,
            ..self.clone()
        }
    }
}

impl PlayerPagination {
    pub async fn page(&self, connection: &mut PgConnection) -> Result<Vec<Player>> {
        self.params.validate()?;

        let order = self.params.order();

        let query = format!(include_str!("../../sql/paginate_players_by_id.sql"), order);

        // FIXME(sqlx) once CITEXT is supported
        let mut stream = sqlx::query(&query)
            .bind(self.params.before)
            .bind(self.params.after)
            .bind(self.name.as_deref())
            .bind(self.name_contains.as_deref())
            .bind(self.banned)
            .bind(&self.nation)
            .bind(self.nation == Some(None))
            .bind(self.params.limit + 1)
            .fetch(connection);

        let mut players = Vec::new();

        while let Some(row) = stream.next().await {
            let row = row?;

            let nationality = match (row.get("nation"), row.get("iso_country_code")) {
                (Some(nation), Some(country_code)) => Some(Nationality {
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
    #[serde(flatten)]
    pub params: PaginationParameters,

    #[serde(default, deserialize_with = "nullable")]
    nation: Option<Option<String>>,

    #[serde(default, deserialize_with = "non_nullable")]
    continent: Option<Continent>,

    #[serde(default, deserialize_with = "non_nullable")]
    subdivision: Option<String>,

    #[serde(default, deserialize_with = "non_nullable")]
    name_contains: Option<String>,
}

impl Pagination for RankingPagination {
    fn parameters(&self) -> PaginationParameters {
        self.params
    }

    fn with_parameters(&self, parameters: PaginationParameters) -> Self {
        Self {
            params: parameters,
            ..self.clone()
        }
    }
}

impl RankingPagination {
    pub async fn page(&self, connection: &mut PgConnection) -> Result<Vec<RankedPlayer>> {
        self.params.validate()?;

        let order = self.params.order();

        let query = format!(include_str!("../../sql/paginate_player_ranking.sql"), order);

        let mut stream = sqlx::query(&query)
            .bind(self.params.before)
            .bind(self.params.after)
            .bind(self.name_contains.as_deref())
            .bind(&self.nation)
            .bind(self.nation == Some(None))
            .bind(self.continent.as_ref().map(|c| c.to_sql()))
            .bind(&self.subdivision)
            .bind(self.params.limit + 1)
            .fetch(connection);

        let mut players = Vec::new();

        while let Some(row) = stream.next().await {
            let row = row?;

            let nationality = match (row.get("nation"), row.get("iso_country_code")) {
                (Some(nation), Some(country_code)) => Some(Nationality {
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
