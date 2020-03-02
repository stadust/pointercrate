use crate::{model::demonlist::record::FullRecord, Result};
use log::info;
use sqlx::PgConnection;

impl FullRecord {
    pub async fn delete(self, connection: &mut PgConnection) -> Result<()> {
        info!("Deleting record {}", self);

        /*sqlx::query!("DELETE FROM record_notes WHERE record = $1", self.id)
            .execute(connection)
            .await?;

        info!("Deleted associated notes!");*/

        sqlx::query!("DELETE FROM records WHERE id = $1", self.id)
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
