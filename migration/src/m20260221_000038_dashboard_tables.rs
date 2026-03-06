use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let _ = db
            .execute_unprepared(
                "CREATE TABLE dashboard (
                    id UUID PRIMARY KEY,
                    workspace_id UUID NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
                    name VARCHAR NOT NULL,
                    description TEXT,
                    layout JSONB NOT NULL DEFAULT '[]'::jsonb,
                    created_by UUID REFERENCES app_user(id) ON DELETE SET NULL,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
                )",
            )
            .await?;

        let _ = db
            .execute_unprepared("CREATE INDEX idx_dashboard_workspace ON dashboard(workspace_id)")
            .await?;

        let _ = db
            .execute_unprepared(
                "CREATE TABLE dashboard_widget (
                    id UUID PRIMARY KEY,
                    dashboard_id UUID NOT NULL REFERENCES dashboard(id) ON DELETE CASCADE,
                    widget_type VARCHAR NOT NULL,
                    title VARCHAR NOT NULL,
                    config JSONB NOT NULL DEFAULT '{}'::jsonb,
                    position_x INTEGER NOT NULL DEFAULT 0,
                    position_y INTEGER NOT NULL DEFAULT 0,
                    width INTEGER NOT NULL DEFAULT 4,
                    height INTEGER NOT NULL DEFAULT 3,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
                )",
            )
            .await?;

        let _ = db
            .execute_unprepared(
                "CREATE INDEX idx_dashboard_widget_dashboard ON dashboard_widget(dashboard_id)",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let _ = db
            .execute_unprepared("DROP TABLE IF EXISTS dashboard_widget")
            .await?;
        let _ = db
            .execute_unprepared("DROP TABLE IF EXISTS dashboard")
            .await?;
        Ok(())
    }
}
