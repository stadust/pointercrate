use crate::{
    error::Result,
    nationality::{Continent, Nationality},
};
use futures::StreamExt;
use pointercrate_core::util::non_nullable;
use serde::{Deserialize, Serialize};
use sqlx::PgConnection;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NationalityRankingPagination {
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
        let mut stream = sqlx::query!(
            r#"SELECT rank as "rank!", score as "score!", nation as "nation!", iso_country_code as "iso_country_code!" FROM ranked_nations WHERE (STRPOS(nation, $1::CITEXT) > 
             0 OR $1 is NULL) AND (continent::text = $2 OR $2 IS NULL)"#,
            self.name_contains,
            self.continent.map(|c| c.to_sql())
        )
        .fetch(connection);

        let mut nations = Vec::new();

        while let Some(row) = stream.next().await {
            let row = row?;

            nations.push(RankedNation {
                rank: row.rank,
                score: row.score,
                nationality: Nationality {
                    iso_country_code: row.iso_country_code,
                    nation: row.nation,
                    subdivision: None,
                },
            })
        }

        Ok(nations)
    }
}
