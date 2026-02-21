use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE object ADD COLUMN classification VARCHAR NOT NULL DEFAULT 'normative'",
            )
            .await?;

        let _ = manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE object ADD CONSTRAINT chk_object_classification
                 CHECK (classification IN ('normative', 'informative', 'heading'))",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager
            .get_connection()
            .execute_unprepared("ALTER TABLE object DROP CONSTRAINT chk_object_classification")
            .await?;

        let _ = manager
            .get_connection()
            .execute_unprepared("ALTER TABLE object DROP COLUMN classification")
            .await?;

        Ok(())
    }
}
