use crate::model::nationality::Subdivision;
use crate::{
    cistring::{CiStr, CiString},
    error::PointercrateError,
    model::nationality::Nationality,
    Result,
};
use futures::stream::StreamExt;
use sqlx::{Error, PgConnection};

impl Nationality {
    pub async fn subdivisions(&self, connection: &mut PgConnection) -> Result<Vec<Subdivision>> {
        let mut stream = sqlx::query!(
            r#"SELECT iso_code as "iso_code: String", name as "name: String" FROM subdivisions WHERE nation = $1 ORDER BY name"#,
            self.iso_country_code
        )
        .fetch(connection);
        let mut subdivisions = Vec::new();

        while let Some(row) = stream.next().await {
            let row = row?;

            subdivisions.push(Subdivision {
                name: CiString::from(row.name),
                iso_code: row.iso_code,
            })
        }

        Ok(subdivisions)
    }

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
                subdivision: None
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
                subdivision: None,
            })
        }

        Ok(nationalities)
    }
}
