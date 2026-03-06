use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE review_comment (
                    id UUID PRIMARY KEY,
                    package_id UUID NOT NULL REFERENCES review_package(id) ON DELETE CASCADE,
                    author_id UUID,
                    body TEXT NOT NULL,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
                )",
            )
            .await?;

        let _ = manager
            .get_connection()
            .execute_unprepared(
                "CREATE INDEX idx_review_comment_package ON review_comment(package_id)",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS review_comment")
            .await?;

        Ok(())
    }
}
