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
    before_id: Option<i32>,

    #[serde(default, deserialize_with = "non_nullable")]
    #[serde(rename = "after")]
    after_id: Option<i32>,

    #[serde(default, deserialize_with = "non_nullable")]
    limit: Option<u8>,

    #[serde(default, deserialize_with = "non_nullable")]
    name: Option<CiString>,
    #[serde(default, deserialize_with = "non_nullable")]
    banned: Option<bool>,

    #[serde(default, deserialize_with = "nullable")]
    nation: Option<Option<String>>,
}

impl PlayerPagination {
    pub async fn page(&self, connection: &mut PgConnection) -> Result<Vec<Player>> {
        let mut stream = sqlx::query(
            "SELECT id, name::TEXT, banned, nation::TEXT, iso_country_code::TEXT FROM players INNER JOIN nationalities ON nationality = \
             iso_country_code WHERE (id < $1 OR $1 IS NULL) AND (id > $2 OR $2 IS NULL) AND (name = $3 OR $3 is NULL) AND (banned = $4 OR \
             $4 IS NULL) AND (nation = $5 OR iso_country_code = $5 OR (nationality IS NULL AND $6) OR ($5 IS NULL AND NOT $6)) ORDER BY \
             id LIMIT $7",
        )
        .bind(self.before_id)
        .bind(self.after_id)
        .bind(&self.name)
        .bind(self.banned)
        .bind(&self.nation)
        .bind(self.nation == Some(None))
        .bind(self.limit.unwrap_or(50) as i32)
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
    before_index: Option<i64>,

    #[serde(default, deserialize_with = "non_nullable")]
    #[serde(rename = "after")]
    after_index: Option<i64>,

    #[serde(default, deserialize_with = "non_nullable")]
    limit: Option<u8>,

    #[serde(default, deserialize_with = "nullable")]
    nation: Option<Option<String>>,
    #[serde(default, deserialize_with = "non_nullable")]
    name_contains: Option<CiString>,
}

impl RankingPagination {
    pub async fn page(&self, connection: &mut PgConnection) -> Result<Vec<RankedPlayer>> {
        let mut stream = sqlx::query(
            "SELECT id, name::TEXT, banned, rank, score, index, nation::TEXT, iso_country_code::TEXT FROM players_with_score WHERE (index \
             < $1 OR $1 IS NULL) AND (index > $2 OR $2 IS NULL) AND (STRPOS(name, $3) > 0 OR $3 is NULL) AND (nation = $4 OR \
             iso_country_code = $4 OR (nationality IS NULL AND $5) OR ($4 IS NULL AND NOT $5)) ORDER BY id LIMIT $6",
        )
        .bind(self.before_index)
        .bind(self.after_index)
        .bind(&self.name_contains)
        .bind(&self.nation)
        .bind(self.nation == Some(None))
        .bind(self.limit.unwrap_or(50) as i32)
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
            })
        }

        Ok(players)
    }
}
/*
impl TablePaginator for RankingPagination {
    type ColumnType = i64;
    type PaginationColumn = players_with_score::index;
    type Table = players_with_score::table;

    fn query(&self, _: RequestContext) -> PaginatorQuery<players_with_score::table> {
        let mut query = RankedPlayer::boxed_all();

        if let Some(ref nation) = self.nation {
            query = query.filter(
                players_with_score::iso_country_code
                    .eq(nation.to_uppercase())
                    .or(players_with_score::nation.eq(Some(CiStr::from_str(nation)))), // okay?
            );
        }

        if let Some(ref like_name) = self.name_contains {
            query = query.filter(
                sql("STRPOS(name, ")
                    .bind::<CiText, _>(like_name)
                    .sql(") > 0"),
            );
        }

        query
    }
}*/
