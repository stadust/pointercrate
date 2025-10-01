use crate::{
    error::Result,
    list::List,
    nationality::{Continent, Nationality},
};
use futures::StreamExt;
use pointercrate_core::util::non_nullable;
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, Row};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NationalityRankingPagination {
    #[serde(default, deserialize_with = "non_nullable")]
    list: Option<List>,

    #[serde(default, deserialize_with = "non_nullable")]
    continent: Option<Continent>,

    #[serde(default, deserialize_with = "non_nullable")]
    name_contains: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RankedNation {
    pub rank: i64,
    pub score: f64,
    #[serde(flatten)]
    pub nationality: Nationality,
}

impl NationalityRankingPagination {
    pub async fn page(&self, connection: &mut PgConnection) -> Result<Vec<RankedNation>> {
        let mut stream = sqlx::query(
            match self.list.unwrap_or_default() {
                List::Demonlist => 
                    r#"SELECT rank, score, nation, iso_country_code FROM ranked_nations WHERE rank IS NOT NULL AND (STRPOS(nation, $1::CITEXT) > 0 OR $1 is NULL) AND (continent::text = $2 OR $2 IS NULL)"#,
                List::RatedPlus => 
                    r#"SELECT unrated_rank as rank, unrated_score as score, nation, iso_country_code FROM ranked_nations WHERE unrated_rank IS NOT NULL AND (STRPOS(nation, $1::CITEXT) > 0 OR $1 is NULL) AND (continent::text = $2 OR $2 IS NULL)"#
            }
        )
        .bind(&self.name_contains)
        .bind(self.continent.map(|c| c.to_sql()))
        .fetch(connection);

        let mut nations = Vec::new();

        while let Some(row) = stream.next().await {
            let row = row?;

            nations.push(RankedNation {
                rank: row.get("rank"),
                score: row.get("score"),
                nationality: Nationality {
                    iso_country_code: row.get("iso_country_code"),
                    nation: row.get("nation"),
                    subdivision: None,
                },
            })
        }

        Ok(nations)
    }
}
