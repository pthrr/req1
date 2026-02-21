use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager
            .get_connection()
            .execute_unprepared("ALTER TABLE object ADD COLUMN deleted_at TIMESTAMPTZ")
            .await?;

        let _ = manager
            .get_connection()
            .execute_unprepared(
                "CREATE INDEX idx_object_deleted ON object(deleted_at) WHERE deleted_at IS NULL",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager
            .get_connection()
            .execute_unprepared("DROP INDEX IF EXISTS idx_object_deleted")
            .await?;

        let _ = manager
            .get_connection()
            .execute_unprepared("ALTER TABLE object DROP COLUMN IF EXISTS deleted_at")
            .await?;

        Ok(())
    }
}
