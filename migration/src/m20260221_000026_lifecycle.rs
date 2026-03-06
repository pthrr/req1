use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create lifecycle_model table
        let _ = manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE lifecycle_model (
                    id UUID PRIMARY KEY,
                    module_id UUID NOT NULL REFERENCES module(id) ON DELETE CASCADE,
                    name VARCHAR NOT NULL,
                    description TEXT,
                    initial_state VARCHAR NOT NULL DEFAULT 'new',
                    states JSONB NOT NULL DEFAULT '[]'::jsonb,
                    transitions JSONB NOT NULL DEFAULT '[]'::jsonb,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
                )",
            )
            .await?;

        let _ = manager
            .get_connection()
            .execute_unprepared(
                "CREATE INDEX idx_lifecycle_model_module ON lifecycle_model(module_id)",
            )
            .await?;

        // Add lifecycle columns to object
        let _ = manager
            .get_connection()
            .execute_unprepared("ALTER TABLE object ADD COLUMN lifecycle_state VARCHAR")
            .await?;

        let _ = manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE object ADD COLUMN lifecycle_model_id UUID REFERENCES lifecycle_model(id) ON DELETE SET NULL",
            )
            .await?;

        // Add default lifecycle model to module
        let _ = manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE module ADD COLUMN default_lifecycle_model_id UUID REFERENCES lifecycle_model(id) ON DELETE SET NULL",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE module DROP COLUMN IF EXISTS default_lifecycle_model_id",
            )
            .await?;

        let _ = manager
            .get_connection()
            .execute_unprepared("ALTER TABLE object DROP COLUMN IF EXISTS lifecycle_model_id")
            .await?;

        let _ = manager
            .get_connection()
            .execute_unprepared("ALTER TABLE object DROP COLUMN IF EXISTS lifecycle_state")
            .await?;

        let _ = manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS lifecycle_model")
            .await?;

        Ok(())
    }
}
