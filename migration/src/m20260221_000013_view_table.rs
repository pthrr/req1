use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE view (
                    id UUID PRIMARY KEY,
                    module_id UUID NOT NULL REFERENCES module(id) ON DELETE CASCADE,
                    name VARCHAR NOT NULL,
                    is_default BOOLEAN NOT NULL DEFAULT false,
                    column_config JSONB NOT NULL DEFAULT '[]'::jsonb,
                    filter_config JSONB NOT NULL DEFAULT '{}'::jsonb,
                    sort_config JSONB NOT NULL DEFAULT '{}'::jsonb,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
                )",
            )
            .await?;

        let _ = manager
            .get_connection()
            .execute_unprepared("CREATE INDEX idx_view_module ON view(module_id)")
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS view")
            .await?;

        Ok(())
    }
}
