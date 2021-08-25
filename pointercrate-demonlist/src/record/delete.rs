use crate::{error::Result, record::FullRecord};
use log::info;
use sqlx::PgConnection;

impl FullRecord {
    pub async fn delete(self, connection: &mut PgConnection) -> Result<()> {
        info!("Deleting record {}", self);

        FullRecord::delete_by_id(self.id, connection).await
    }

    /// `FullRecord::delete` should be preferred
    pub async fn delete_by_id(record_id: i32, connection: &mut PgConnection) -> Result<()> {
        // Associated notes get deleted due to the ON DELETE CASCADE on record_notes.record

        sqlx::query!("DELETE FROM records WHERE id = $1", record_id)
            .execute(connection)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::model::demonlist::record::FullRecord;

    #[actix_rt::test]
    async fn test_delete_record() {
        let mut connection = crate::test::test_setup().await;

        // Get the first demon of the test set
        let id = sqlx::query!("SELECT id FROM records WHERE status_ = 'SUBMITTED'")
            .fetch_one(&mut connection)
            .await
            .unwrap()
            .id;

        let record = FullRecord::by_id(id, &mut connection).await.unwrap();

        let result = record.delete(&mut connection).await;

        assert!(result.is_ok(), "{:?}", result.unwrap_err());
        assert!(FullRecord::by_id(id, &mut connection).await.is_err());
    }

    #[actix_rt::test]
    async fn test_delete_record_with_notes() {
        let mut connection = crate::test::test_setup().await;

        // Get the second demon of the test set
        let id = sqlx::query!("SELECT id FROM records WHERE status_='REJECTED'")
            .fetch_one(&mut connection)
            .await
            .unwrap()
            .id;

        let record = FullRecord::by_id(id, &mut connection).await.unwrap();

        let result = record.delete(&mut connection).await;

        assert!(result.is_ok(), "{:?}", result.unwrap_err());
        assert!(FullRecord::by_id(id, &mut connection).await.is_err());
    }
}
