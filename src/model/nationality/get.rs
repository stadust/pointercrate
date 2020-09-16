use crate::{
    cistring::{CiStr, CiString},
    error::PointercrateError,
    model::nationality::Nationality,
    Result,
};
use futures::stream::StreamExt;
use sqlx::{Error, PgConnection};

impl Nationality {
    pub async fn by_country_code_or_name(code: &CiStr, connection: &mut PgConnection) -> Result<Nationality> {
        sqlx::query!(
            r#"SELECT nation as "nation: String", iso_country_code as "iso_country_code: String" FROM nationalities WHERE iso_country_code = $1 or nation = $1"#,
            code.to_string() /* FIXME(sqlx 0.3) */
        )
        .fetch_one(connection)
        .await
        .map(|row| {
            Nationality {
                nation: CiString::from(row.nation),
                iso_country_code: row.iso_country_code,
            }
        })
        .map_err(|sqlx_error| {
            match sqlx_error {
                Error::RowNotFound =>
                    PointercrateError::ModelNotFound {
                        model: "Nationality",
                        identified_by: code.to_string(),
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
                nation: CiString::from(row.nation),
                iso_country_code: row.iso_country_code,
            })
        }

        Ok(nationalities)
    }
}
