use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE module
                    ADD COLUMN prefix VARCHAR NOT NULL DEFAULT '',
                    ADD COLUMN separator VARCHAR NOT NULL DEFAULT '-',
                    ADD COLUMN digits INT NOT NULL DEFAULT 3,
                    ADD COLUMN required_attributes JSONB NOT NULL DEFAULT '[]'::jsonb,
                    ADD COLUMN default_classification VARCHAR NOT NULL DEFAULT 'normative'",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE module
                    DROP COLUMN IF EXISTS prefix,
                    DROP COLUMN IF EXISTS separator,
                    DROP COLUMN IF EXISTS digits,
                    DROP COLUMN IF EXISTS required_attributes,
                    DROP COLUMN IF EXISTS default_classification",
            )
            .await?;

        Ok(())
    }
}
