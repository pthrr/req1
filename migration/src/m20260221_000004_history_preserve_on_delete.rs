use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop the CASCADE FK on object_history.object_id so history
        // survives object deletion (audit trail must be permanent).
        let db = manager.get_connection();
        let _ = db
            .execute_unprepared(
                "ALTER TABLE object_history DROP CONSTRAINT IF EXISTS object_history_object_id_fkey",
            )
            .await?;
        // Also try the SeaORM-generated name pattern (hyphens require quoting)
        let _ = db
            .execute_unprepared(
                r#"ALTER TABLE object_history DROP CONSTRAINT IF EXISTS "fk-object_history-object_id-object-id""#,
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let _ = db
            .execute_unprepared(
                "ALTER TABLE object_history ADD CONSTRAINT object_history_object_id_fkey \
                 FOREIGN KEY (object_id) REFERENCES object(id) ON DELETE CASCADE",
            )
            .await?;
        Ok(())
    }
}
