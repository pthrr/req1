use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE comment (
                    id UUID PRIMARY KEY,
                    object_id UUID NOT NULL REFERENCES object(id) ON DELETE CASCADE,
                    author_id UUID,
                    body TEXT NOT NULL,
                    resolved BOOLEAN NOT NULL DEFAULT false,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
                )",
            )
            .await?;

        let _ = manager
            .get_connection()
            .execute_unprepared("CREATE INDEX idx_comment_object ON comment(object_id)")
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS comment")
            .await?;

        Ok(())
    }
}
