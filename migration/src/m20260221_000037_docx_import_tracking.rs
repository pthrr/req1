use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let _ = db
            .execute_unprepared("ALTER TABLE object ADD COLUMN docx_source_id VARCHAR")
            .await?;

        let _ = db
            .execute_unprepared(
                "CREATE INDEX idx_object_docx_source ON object(module_id, docx_source_id) WHERE docx_source_id IS NOT NULL",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let _ = db
            .execute_unprepared("DROP INDEX IF EXISTS idx_object_docx_source")
            .await?;
        let _ = db
            .execute_unprepared("ALTER TABLE object DROP COLUMN IF EXISTS docx_source_id")
            .await?;
        Ok(())
    }
}
