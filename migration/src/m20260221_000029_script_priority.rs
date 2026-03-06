use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE script ADD COLUMN priority INTEGER NOT NULL DEFAULT 100",
            )
            .await?;

        let _ = manager
            .get_connection()
            .execute_unprepared(
                "CREATE INDEX idx_script_priority ON script(module_id, hook_point, priority)",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager
            .get_connection()
            .execute_unprepared("DROP INDEX IF EXISTS idx_script_priority")
            .await?;

        let _ = manager
            .get_connection()
            .execute_unprepared("ALTER TABLE script DROP COLUMN IF EXISTS priority")
            .await?;

        Ok(())
    }
}
