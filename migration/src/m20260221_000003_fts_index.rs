use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let _ = db
            .execute_unprepared(
                "CREATE INDEX idx_object_fts ON object USING GIN (\
                 to_tsvector('english', COALESCE(heading, '') || ' ' || COALESCE(body, '')))",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let _ = db
            .execute_unprepared("DROP INDEX IF EXISTS idx_object_fts")
            .await?;
        Ok(())
    }
}
