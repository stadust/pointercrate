use crate::{
    nationality::{Continent, Nationality},
    player::{DatabasePlayer, Player},
};
use futures::StreamExt;
use pointercrate_core::{
    first_and_last,
    pagination::{PageContext, Paginatable, PaginationParameters, PaginationQuery, __pagination_compat},
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

impl PaginationQuery for PlayerPagination {
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

impl Paginatable<PlayerPagination> for Player {
    first_and_last!("players");

    async fn page(query: &PlayerPagination, connection: &mut PgConnection) -> Result<(Vec<Player>, PageContext), sqlx::Error> {
        let order = query.params.order();

        let sql_query = format!(include_str!("../../sql/paginate_players_by_id.sql"), order);

        // FIXME(sqlx) once CITEXT is supported
        let mut stream = sqlx::query(&sql_query)
            .bind(query.params.before)
            .bind(query.params.after)
            .bind(query.name.as_deref())
            .bind(query.name_contains.as_deref())
            .bind(query.banned)
            .bind(&query.nation)
            .bind(query.nation == Some(None))
            .bind(query.params.limit + 1)
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
                score: row.get("score"),
                nationality,
            })
        }

        Ok(__pagination_compat(&query.params, players))
    }

    fn pagination_id(&self) -> i32 {
        self.base.id
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

impl PaginationQuery for RankingPagination {
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

#[derive(Debug, Serialize)]
pub struct RankedPlayer {
    rank: i64,
    #[serde(skip)]
    index: i64,
    #[serde(flatten)]
    player: Player,
}

impl Paginatable<RankingPagination> for RankedPlayer {
    async fn first_and_last(connection: &mut PgConnection) -> Result<Option<(i32, i32)>, sqlx::Error> {
        Ok(sqlx::query!("SELECT COUNT(*) FROM players WHERE NOT banned AND score > 0.0")
            .fetch_one(connection)
            .await?
            .count
            .map(|max| (1, max as i32)))
    }

    async fn page(query: &RankingPagination, connection: &mut PgConnection) -> Result<(Vec<RankedPlayer>, PageContext), sqlx::Error> {
        let order = query.params.order();

        let sql_query = format!(include_str!("../../sql/paginate_player_ranking.sql"), order);

        let mut stream = sqlx::query(&sql_query)
            .bind(query.params.before)
            .bind(query.params.after)
            .bind(query.name_contains.as_deref())
            .bind(&query.nation)
            .bind(query.nation == Some(None))
            .bind(query.continent.as_ref().map(|c| c.to_sql()))
            .bind(&query.subdivision)
            .bind(query.params.limit + 1)
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

            let player = Player {
                base: DatabasePlayer {
                    id: row.get("id"),
                    name: row.get("name"),
                    banned: false,
                },
                score: row.get("score"),
                nationality,
            };

            players.push(RankedPlayer {
                rank: row.get("rank"),
                index: row.get("index"),
                player,
            })
        }

        Ok(__pagination_compat(&query.params, players))
    }

    fn pagination_id(&self) -> i32 {
        self.index as i32
    }
}
