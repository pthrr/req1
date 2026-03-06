use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager
            .get_connection()
            .execute_unprepared("ALTER TABLE module ADD COLUMN publish_template TEXT")
            .await?;

        // Migrate existing hacked templates from description field
        let _ = manager
            .get_connection()
            .execute_unprepared(
                "UPDATE module SET publish_template = SUBSTRING(description FROM 13), description = NULL WHERE description LIKE '__template__%'",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager
            .get_connection()
            .execute_unprepared("ALTER TABLE module DROP COLUMN IF EXISTS publish_template")
            .await?;

        Ok(())
    }
}
