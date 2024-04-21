use futures::StreamExt;
use pointercrate_core::{
    audit::NamedId,
    first_and_last,
    pagination::{PageContext, Paginatable, PaginationParameters, PaginationQuery, __pagination_compat},
    util::non_nullable,
};
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, Row};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PlayerClaimPagination {
    #[serde(flatten)]
    pub params: PaginationParameters,

    #[serde(default, deserialize_with = "non_nullable")]
    any_name_contains: Option<String>,

    #[serde(default, deserialize_with = "non_nullable")]
    verified: Option<bool>,
}

#[derive(Serialize)]
pub struct ListedClaim {
    #[serde(skip)]
    pub id: i32,
    user: NamedId,
    player: NamedId,
    verified: bool,
}

impl PaginationQuery for PlayerClaimPagination {
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

impl Paginatable<PlayerClaimPagination> for ListedClaim {
    first_and_last!("player_claims");

    async fn page(query: &PlayerClaimPagination, connection: &mut PgConnection) -> Result<(Vec<ListedClaim>, PageContext), sqlx::Error> {
        let order = query.params.order();

        let sql_query = format!(include_str!("../../../sql/paginate_claims.sql"), order);

        let mut stream = sqlx::query(&sql_query)
            .bind(query.params.before)
            .bind(query.params.after)
            .bind(query.any_name_contains.as_ref())
            .bind(query.verified)
            .bind(query.params.limit + 1)
            .fetch(connection);

        let mut claims = Vec::new();

        while let Some(row) = stream.next().await {
            let row = row?;

            claims.push(ListedClaim {
                id: row.get("id"),
                user: NamedId {
                    id: row.get("mid"),
                    name: Some(row.get("mname")),
                },
                player: NamedId {
                    id: row.get("pid"),
                    name: Some(row.get("pname")),
                },
                verified: row.get("verified"),
            })
        }

        Ok(__pagination_compat(&query.params, claims))
    }

    fn pagination_id(&self) -> i32 {
        self.id
    }
}
