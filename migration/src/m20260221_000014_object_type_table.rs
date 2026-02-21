use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE object_type (
                    id UUID PRIMARY KEY,
                    module_id UUID NOT NULL REFERENCES module(id) ON DELETE CASCADE,
                    name VARCHAR NOT NULL,
                    description TEXT,
                    default_classification VARCHAR NOT NULL DEFAULT 'normative',
                    required_attributes JSONB NOT NULL DEFAULT '[]'::jsonb,
                    attribute_schema JSONB NOT NULL DEFAULT '{}'::jsonb,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                    UNIQUE(module_id, name)
                )",
            )
            .await?;

        let _ = manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE object ADD COLUMN object_type_id UUID REFERENCES object_type(id) ON DELETE SET NULL",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager
            .get_connection()
            .execute_unprepared("ALTER TABLE object DROP COLUMN IF EXISTS object_type_id")
            .await?;

        let _ = manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS object_type")
            .await?;

        Ok(())
    }
}
