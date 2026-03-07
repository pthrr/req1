use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE object ADD COLUMN source_object_id UUID REFERENCES object(id) ON DELETE SET NULL",
            )
            .await?;

        let _ = manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE object ADD COLUMN source_module_id UUID REFERENCES module(id) ON DELETE SET NULL",
            )
            .await?;

        let _ = manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE object ADD COLUMN is_placeholder BOOLEAN NOT NULL DEFAULT false",
            )
            .await?;

        let _ = manager
            .get_connection()
            .execute_unprepared(
                "CREATE INDEX idx_object_source ON object(source_object_id) WHERE source_object_id IS NOT NULL",
            )
            .await?;

        let _ = manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE object ADD CONSTRAINT chk_placeholder CHECK (NOT is_placeholder OR source_object_id IS NOT NULL)",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager
            .get_connection()
            .execute_unprepared("ALTER TABLE object DROP CONSTRAINT IF EXISTS chk_placeholder")
            .await?;

        let _ = manager
            .get_connection()
            .execute_unprepared("DROP INDEX IF EXISTS idx_object_source")
            .await?;

        let _ = manager
            .get_connection()
            .execute_unprepared("ALTER TABLE object DROP COLUMN IF EXISTS is_placeholder")
            .await?;

        let _ = manager
            .get_connection()
            .execute_unprepared("ALTER TABLE object DROP COLUMN IF EXISTS source_module_id")
            .await?;

        let _ = manager
            .get_connection()
            .execute_unprepared("ALTER TABLE object DROP COLUMN IF EXISTS source_object_id")
            .await?;

        Ok(())
    }
}
