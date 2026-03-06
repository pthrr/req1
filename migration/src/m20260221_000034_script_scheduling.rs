use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let _ = db
            .execute_unprepared("ALTER TABLE script ADD COLUMN cron_expression VARCHAR")
            .await?;

        let _ = db
            .execute_unprepared("ALTER TABLE script ADD COLUMN last_run_at TIMESTAMPTZ")
            .await?;

        let _ = db
            .execute_unprepared("ALTER TABLE script ADD COLUMN next_run_at TIMESTAMPTZ")
            .await?;

        let _ = db
            .execute_unprepared(
                "CREATE TABLE script_execution (
                    id UUID PRIMARY KEY,
                    script_id UUID NOT NULL REFERENCES script(id) ON DELETE CASCADE,
                    status VARCHAR NOT NULL DEFAULT 'running',
                    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                    finished_at TIMESTAMPTZ,
                    duration_ms BIGINT,
                    output TEXT,
                    error_message TEXT,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
                )",
            )
            .await?;

        let _ = db
            .execute_unprepared(
                "CREATE INDEX idx_script_execution_script ON script_execution(script_id)",
            )
            .await?;

        let _ = db
            .execute_unprepared(
                "CREATE INDEX idx_script_execution_started ON script_execution(started_at)",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let _ = db
            .execute_unprepared("DROP TABLE IF EXISTS script_execution")
            .await?;

        let _ = db
            .execute_unprepared("ALTER TABLE script DROP COLUMN IF EXISTS next_run_at")
            .await?;

        let _ = db
            .execute_unprepared("ALTER TABLE script DROP COLUMN IF EXISTS last_run_at")
            .await?;

        let _ = db
            .execute_unprepared("ALTER TABLE script DROP COLUMN IF EXISTS cron_expression")
            .await?;

        Ok(())
    }
}
