use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE attribute_definition ADD COLUMN depends_on UUID REFERENCES attribute_definition(id) ON DELETE SET NULL",
            )
            .await?;

        let _ = manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE attribute_definition ADD COLUMN dependency_mapping JSONB",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE attribute_definition DROP COLUMN IF EXISTS dependency_mapping",
            )
            .await?;

        let _ = manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE attribute_definition DROP COLUMN IF EXISTS depends_on",
            )
            .await?;

        Ok(())
    }
}
