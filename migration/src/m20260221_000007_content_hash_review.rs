use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add content_fingerprint column
        let _ = manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE object ADD COLUMN content_fingerprint VARCHAR NOT NULL DEFAULT ''",
            )
            .await?;

        // Add reviewed_fingerprint column
        let _ = manager
            .get_connection()
            .execute_unprepared("ALTER TABLE object ADD COLUMN reviewed_fingerprint VARCHAR")
            .await?;

        // Backfill fingerprints using SHA-256
        // Use convert_to + bytea concatenation with decode('00','hex') to match Rust fingerprint (which uses b"\0" separator)
        let _ = manager
            .get_connection()
            .execute_unprepared(
                "UPDATE object SET content_fingerprint = encode(sha256(
                    convert_to(COALESCE(heading, ''), 'UTF8') || decode('00', 'hex') || convert_to(COALESCE(body, ''), 'UTF8') || decode('00', 'hex') || convert_to(COALESCE(attributes::text, ''), 'UTF8')
                ), 'hex')",
            )
            .await?;

        // For reviewed objects, set reviewed_fingerprint = content_fingerprint
        let _ = manager
            .get_connection()
            .execute_unprepared(
                "UPDATE object SET reviewed_fingerprint = content_fingerprint WHERE reviewed = true",
            )
            .await?;

        // Drop the old reviewed column
        let _ = manager
            .get_connection()
            .execute_unprepared("ALTER TABLE object DROP COLUMN reviewed")
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE object ADD COLUMN reviewed BOOLEAN NOT NULL DEFAULT false",
            )
            .await?;

        let _ = manager
            .get_connection()
            .execute_unprepared(
                "UPDATE object SET reviewed = (reviewed_fingerprint IS NOT NULL AND reviewed_fingerprint = content_fingerprint)",
            )
            .await?;

        let _ = manager
            .get_connection()
            .execute_unprepared("ALTER TABLE object DROP COLUMN reviewed_fingerprint")
            .await?;

        let _ = manager
            .get_connection()
            .execute_unprepared("ALTER TABLE object DROP COLUMN content_fingerprint")
            .await?;

        Ok(())
    }
}
