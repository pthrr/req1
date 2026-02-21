use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE app_user (
                    id UUID PRIMARY KEY,
                    email VARCHAR NOT NULL UNIQUE,
                    display_name VARCHAR NOT NULL,
                    role VARCHAR NOT NULL DEFAULT 'editor',
                    active BOOLEAN NOT NULL DEFAULT true,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                    CONSTRAINT chk_user_role CHECK (role IN ('admin', 'editor', 'viewer'))
                )",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS app_user")
            .await?;

        Ok(())
    }
}
