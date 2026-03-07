use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let _ = db
            .execute_unprepared(
                "ALTER TABLE comment ADD COLUMN mentioned_user_ids JSONB NOT NULL DEFAULT '[]'::jsonb",
            )
            .await?;

        let _ = db
            .execute_unprepared(
                "ALTER TABLE review_comment ADD COLUMN mentioned_user_ids JSONB NOT NULL DEFAULT '[]'::jsonb",
            )
            .await?;

        let _ = db
            .execute_unprepared(
                "CREATE TABLE notification (
                    id UUID PRIMARY KEY,
                    user_id UUID NOT NULL REFERENCES app_user(id) ON DELETE CASCADE,
                    notification_type VARCHAR NOT NULL DEFAULT 'mention',
                    title VARCHAR NOT NULL,
                    body TEXT NOT NULL,
                    entity_type VARCHAR NOT NULL,
                    entity_id UUID,
                    read BOOLEAN NOT NULL DEFAULT FALSE,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
                )",
            )
            .await?;

        let _ = db
            .execute_unprepared("CREATE INDEX idx_notification_user ON notification(user_id)")
            .await?;

        let _ = db
            .execute_unprepared(
                "CREATE INDEX idx_notification_user_read ON notification(user_id, read)",
            )
            .await?;

        let _ = db
            .execute_unprepared("CREATE INDEX idx_notification_created ON notification(created_at)")
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let _ = db
            .execute_unprepared("DROP TABLE IF EXISTS notification")
            .await?;
        let _ = db
            .execute_unprepared(
                "ALTER TABLE review_comment DROP COLUMN IF EXISTS mentioned_user_ids",
            )
            .await?;
        let _ = db
            .execute_unprepared("ALTER TABLE comment DROP COLUMN IF EXISTS mentioned_user_ids")
            .await?;
        Ok(())
    }
}
