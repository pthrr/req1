use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{DbBackend, Statement};

#[derive(DeriveMigrationName)]
pub struct Migration;

// Uses the seed module UUID from m20260221_000002_seed_data
const SEED_MODULE_ID: &str = "00000000-0000-0000-0000-000000000100";

const SCRIPT_ID_LINK_COUNT: &str = "00000000-0000-0000-0000-100000000001";
const SCRIPT_ID_CLASSIFICATION: &str = "00000000-0000-0000-0000-100000000002";
const SCRIPT_ID_HAS_BODY: &str = "00000000-0000-0000-0000-100000000003";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Only seed if the module exists (seed data was applied)
        let result = db
            .query_one(Statement::from_string(
                DbBackend::Postgres,
                format!(
                    "SELECT EXISTS(SELECT 1 FROM module WHERE id = '{SEED_MODULE_ID}') AS has_module"
                ),
            ))
            .await?;
        let has_module: bool = result
            .and_then(|row| row.try_get("", "has_module").ok())
            .unwrap_or(false);
        if !has_module {
            return Ok(());
        }

        let _ = db
            .execute_unprepared(&format!(
                "INSERT INTO script (id, module_id, name, script_type, hook_point, source_code, enabled)
                 VALUES ('{SCRIPT_ID_LINK_COUNT}', '{SEED_MODULE_ID}', 'Link Count', 'layout', NULL, 'return tostring(#req1.links(obj.id))', true)
                 ON CONFLICT (id) DO NOTHING"
            ))
            .await?;

        let _ = db
            .execute_unprepared(&format!(
                "INSERT INTO script (id, module_id, name, script_type, hook_point, source_code, enabled)
                 VALUES ('{SCRIPT_ID_CLASSIFICATION}', '{SEED_MODULE_ID}', 'Classification Badge', 'layout', NULL, 'return obj.classification and string.upper(string.sub(obj.classification, 1, 1)) or \"?\"', true)
                 ON CONFLICT (id) DO NOTHING"
            ))
            .await?;

        let _ = db
            .execute_unprepared(&format!(
                "INSERT INTO script (id, module_id, name, script_type, hook_point, source_code, enabled)
                 VALUES ('{SCRIPT_ID_HAS_BODY}', '{SEED_MODULE_ID}', 'Has Body', 'layout', NULL, 'return obj.body and \"Yes\" or \"No\"', true)
                 ON CONFLICT (id) DO NOTHING"
            ))
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let _ = db
            .execute_unprepared(&format!(
                "DELETE FROM script WHERE id IN ('{SCRIPT_ID_LINK_COUNT}', '{SCRIPT_ID_CLASSIFICATION}', '{SCRIPT_ID_HAS_BODY}')"
            ))
            .await?;

        Ok(())
    }
}
