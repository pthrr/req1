use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let _ = db
            .execute_unprepared(
                "CREATE TABLE diagram (
                    id UUID PRIMARY KEY,
                    module_id UUID NOT NULL REFERENCES module(id) ON DELETE CASCADE,
                    name VARCHAR NOT NULL,
                    description TEXT,
                    diagram_type VARCHAR NOT NULL DEFAULT 'use_case',
                    source_code TEXT NOT NULL DEFAULT '',
                    linked_object_ids JSONB NOT NULL DEFAULT '[]'::jsonb,
                    created_by UUID REFERENCES app_user(id) ON DELETE SET NULL,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
                )",
            )
            .await?;

        let _ = db
            .execute_unprepared("CREATE INDEX idx_diagram_module ON diagram(module_id)")
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS diagram")
            .await?;
        Ok(())
    }
}
