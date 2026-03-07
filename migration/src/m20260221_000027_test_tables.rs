use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE test_case (
                    id UUID PRIMARY KEY,
                    module_id UUID NOT NULL REFERENCES module(id) ON DELETE CASCADE,
                    name VARCHAR NOT NULL,
                    description TEXT,
                    preconditions TEXT,
                    expected_result TEXT,
                    test_type VARCHAR NOT NULL DEFAULT 'manual',
                    priority VARCHAR NOT NULL DEFAULT 'medium',
                    status VARCHAR NOT NULL DEFAULT 'draft',
                    requirement_ids JSONB NOT NULL DEFAULT '[]'::jsonb,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
                )",
            )
            .await?;

        let _ = manager
            .get_connection()
            .execute_unprepared("CREATE INDEX idx_test_case_module ON test_case(module_id)")
            .await?;

        let _ = manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE test_execution (
                    id UUID PRIMARY KEY,
                    test_case_id UUID NOT NULL REFERENCES test_case(id) ON DELETE CASCADE,
                    status VARCHAR NOT NULL DEFAULT 'not_run',
                    executor VARCHAR,
                    executed_at TIMESTAMPTZ,
                    duration_ms BIGINT,
                    evidence TEXT,
                    environment VARCHAR,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
                )",
            )
            .await?;

        let _ = manager
            .get_connection()
            .execute_unprepared(
                "CREATE INDEX idx_test_execution_test_case ON test_execution(test_case_id)",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS test_execution")
            .await?;

        let _ = manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS test_case")
            .await?;

        Ok(())
    }
}
