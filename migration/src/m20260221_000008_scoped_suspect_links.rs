use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add source_fingerprint column
        let _ = manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE link ADD COLUMN source_fingerprint VARCHAR NOT NULL DEFAULT ''",
            )
            .await?;

        // Add target_fingerprint column
        let _ = manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE link ADD COLUMN target_fingerprint VARCHAR NOT NULL DEFAULT ''",
            )
            .await?;

        // Populate from current object content_fingerprint
        let _ = manager
            .get_connection()
            .execute_unprepared(
                "UPDATE link SET source_fingerprint = o.content_fingerprint
                 FROM object o WHERE link.source_object_id = o.id",
            )
            .await?;

        let _ = manager
            .get_connection()
            .execute_unprepared(
                "UPDATE link SET target_fingerprint = o.content_fingerprint
                 FROM object o WHERE link.target_object_id = o.id",
            )
            .await?;

        // Reset all links to non-suspect
        let _ = manager
            .get_connection()
            .execute_unprepared("UPDATE link SET suspect = false")
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager
            .get_connection()
            .execute_unprepared("ALTER TABLE link DROP COLUMN source_fingerprint")
            .await?;

        let _ = manager
            .get_connection()
            .execute_unprepared("ALTER TABLE link DROP COLUMN target_fingerprint")
            .await?;

        Ok(())
    }
}
