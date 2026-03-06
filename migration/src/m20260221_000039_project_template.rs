use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let _ = db
            .execute_unprepared(
                "CREATE TABLE project_template (
                    id UUID PRIMARY KEY,
                    name VARCHAR NOT NULL UNIQUE,
                    description TEXT,
                    standard VARCHAR,
                    version VARCHAR,
                    template_data JSONB NOT NULL DEFAULT '{}'::jsonb,
                    is_builtin BOOLEAN NOT NULL DEFAULT false,
                    created_by UUID REFERENCES app_user(id) ON DELETE SET NULL,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
                )",
            )
            .await?;

        let _ = db
            .execute_unprepared(
                "CREATE INDEX idx_project_template_builtin ON project_template(is_builtin)",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS project_template")
            .await?;
        Ok(())
    }
}
