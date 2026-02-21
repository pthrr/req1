use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE attachment (
                    id UUID PRIMARY KEY,
                    object_id UUID NOT NULL REFERENCES object(id) ON DELETE CASCADE,
                    file_name VARCHAR NOT NULL,
                    content_type VARCHAR NOT NULL DEFAULT 'application/octet-stream',
                    size_bytes BIGINT NOT NULL DEFAULT 0,
                    storage_path VARCHAR NOT NULL,
                    sha256 VARCHAR,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
                )",
            )
            .await?;

        let _ = manager
            .get_connection()
            .execute_unprepared("CREATE INDEX idx_attachment_object ON attachment(object_id)")
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS attachment")
            .await?;

        Ok(())
    }
}
