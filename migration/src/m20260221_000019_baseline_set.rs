use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE baseline_set (
                    id UUID PRIMARY KEY,
                    name VARCHAR NOT NULL,
                    version VARCHAR NOT NULL DEFAULT '1.0',
                    description TEXT,
                    created_by UUID,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
                )",
            )
            .await?;

        let _ = manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE baseline ADD COLUMN baseline_set_id UUID REFERENCES baseline_set(id) ON DELETE SET NULL",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager
            .get_connection()
            .execute_unprepared("ALTER TABLE baseline DROP COLUMN IF EXISTS baseline_set_id")
            .await?;

        let _ = manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS baseline_set")
            .await?;

        Ok(())
    }
}
