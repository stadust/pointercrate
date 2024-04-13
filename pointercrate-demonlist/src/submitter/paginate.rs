use crate::submitter::Submitter;
use futures::StreamExt;
use pointercrate_core::{
    first_and_last, pagination::{Pagination, PaginationParameters}, util::non_nullable
};
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, Row};

#[derive(Deserialize, Debug, Clone, Copy, Serialize)]
pub struct SubmitterPagination {
    #[serde(flatten)]
    pub params: PaginationParameters,

    #[serde(default, deserialize_with = "non_nullable")]
    banned: Option<bool>,
}

impl Pagination for SubmitterPagination {
    type Item = Submitter; 

    fn parameters(&self) -> PaginationParameters {
        self.params
    }

    fn with_parameters(&self, parameters: PaginationParameters) -> Self {
        Self {
            params: parameters,
            ..*self
        }
    }

    first_and_last!("submitters", "submitter_id");
    
    async fn page(&self, connection: &mut PgConnection) -> Result<Vec<Submitter>, sqlx::Error> {
        let order = self.params.order();

        let query = format!("SELECT submitter_id, banned FROM submitters WHERE (submitter_id < $1 OR $1 IS NULL) AND (submitter_id > $2 OR $2 IS NULL) AND (banned = $3 OR $3 IS NULL) ORDER BY submitter_id {} LIMIT $4", order);

        let mut stream = sqlx::query(&query)
            .bind(self.params.before)
            .bind(self.params.after)
            .bind(self.banned)
            .bind(self.params.limit + 1)
            .fetch(connection);

        let mut submitters = Vec::new();

        while let Some(row) = stream.next().await {
            let row = row?;

            submitters.push(Submitter {
                id: row.get("submitter_id"),
                banned: row.get("banned"),
            })
        }

        Ok(submitters)
    }
}
