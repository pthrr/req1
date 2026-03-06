use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let _ = db
            .execute_unprepared(
                "CREATE TABLE e_signature (
                    id UUID PRIMARY KEY,
                    user_id UUID NOT NULL REFERENCES app_user(id),
                    entity_type VARCHAR NOT NULL,
                    entity_id UUID NOT NULL,
                    meaning VARCHAR NOT NULL,
                    signature_hash VARCHAR NOT NULL,
                    ip_address VARCHAR,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
                )",
            )
            .await?;

        let _ = db
            .execute_unprepared("CREATE INDEX idx_e_signature_user ON e_signature(user_id)")
            .await?;

        let _ = db
            .execute_unprepared(
                "CREATE INDEX idx_e_signature_entity ON e_signature(entity_type, entity_id)",
            )
            .await?;

        let _ = db
            .execute_unprepared(
                "ALTER TABLE module ADD COLUMN signature_config JSONB NOT NULL DEFAULT '{}'::jsonb",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let _ = db
            .execute_unprepared("ALTER TABLE module DROP COLUMN IF EXISTS signature_config")
            .await?;
        let _ = db
            .execute_unprepared("DROP TABLE IF EXISTS e_signature")
            .await?;
        Ok(())
    }
}
