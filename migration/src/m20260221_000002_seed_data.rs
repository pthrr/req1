use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{DbBackend, Statement};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Only seed into an empty database
        let result = db
            .query_one(Statement::from_string(
                DbBackend::Postgres,
                "SELECT EXISTS(SELECT 1 FROM workspace) AS has_data",
            ))
            .await?;
        let has_data: bool = result
            .and_then(|row| row.try_get("", "has_data").ok())
            .unwrap_or(false);
        if has_data {
            return Ok(());
        }

        // Workspace
        let _ = db
            .execute_unprepared(
                "INSERT INTO workspace (id, name, description)
                 VALUES ('00000000-0000-0000-0000-000000000001', 'Default Workspace', 'Seeded workspace for first-run experience')",
            )
            .await?;

        // Project
        let _ = db
            .execute_unprepared(
                "INSERT INTO project (id, workspace_id, name, description)
                 VALUES ('00000000-0000-0000-0000-000000000001', '00000000-0000-0000-0000-000000000001', 'Demo Project', 'Seeded project for first-run experience')",
            )
            .await?;

        // Link types
        let _ = db
            .execute_unprepared(
                "INSERT INTO link_type (id, name, description)
                 VALUES ('00000000-0000-0000-0000-000000000010', 'satisfies', 'Source satisfies target requirement')",
            )
            .await?;

        let _ = db
            .execute_unprepared(
                "INSERT INTO link_type (id, name, description)
                 VALUES ('00000000-0000-0000-0000-000000000011', 'derives-from', 'Source is derived from target')",
            )
            .await?;

        // Module
        let _ = db
            .execute_unprepared(
                "INSERT INTO module (id, project_id, name, description)
                 VALUES ('00000000-0000-0000-0000-000000000100', '00000000-0000-0000-0000-000000000001', 'System Requirements', 'Top-level system requirements module')",
            )
            .await?;

        // Objects
        let _ = db
            .execute_unprepared(
                "INSERT INTO object (id, module_id, position, heading, body, current_version)
                 VALUES ('00000000-0000-0000-0000-000000001001', '00000000-0000-0000-0000-000000000100', 0, 'System Overview', 'The system shall provide requirements management capabilities.', 1)",
            )
            .await?;

        let _ = db
            .execute_unprepared(
                "INSERT INTO object (id, module_id, position, heading, body, current_version)
                 VALUES ('00000000-0000-0000-0000-000000001002', '00000000-0000-0000-0000-000000000100', 1, 'Performance Requirements', 'The system shall respond to user actions within 200ms.', 1)",
            )
            .await?;

        let _ = db
            .execute_unprepared(
                "INSERT INTO object (id, module_id, position, heading, body, current_version)
                 VALUES ('00000000-0000-0000-0000-000000001003', '00000000-0000-0000-0000-000000000100', 2, 'Security Requirements', 'The system shall enforce role-based access control.', 1)",
            )
            .await?;

        // History records for initial creation
        let _ = db
            .execute_unprepared(
                "INSERT INTO object_history (object_id, module_id, version, heading, body, change_type)
                 VALUES ('00000000-0000-0000-0000-000000001001', '00000000-0000-0000-0000-000000000100', 1, 'System Overview', 'The system shall provide requirements management capabilities.', 'create')",
            )
            .await?;

        let _ = db
            .execute_unprepared(
                "INSERT INTO object_history (object_id, module_id, version, heading, body, change_type)
                 VALUES ('00000000-0000-0000-0000-000000001002', '00000000-0000-0000-0000-000000000100', 1, 'Performance Requirements', 'The system shall respond to user actions within 200ms.', 'create')",
            )
            .await?;

        let _ = db
            .execute_unprepared(
                "INSERT INTO object_history (object_id, module_id, version, heading, body, change_type)
                 VALUES ('00000000-0000-0000-0000-000000001003', '00000000-0000-0000-0000-000000000100', 1, 'Security Requirements', 'The system shall enforce role-based access control.', 'create')",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Delete in reverse dependency order
        let _ = db
            .execute_unprepared(
                "DELETE FROM object_history WHERE object_id IN (
                    '00000000-0000-0000-0000-000000001001',
                    '00000000-0000-0000-0000-000000001002',
                    '00000000-0000-0000-0000-000000001003'
                 )",
            )
            .await?;

        let _ = db
            .execute_unprepared(
                "DELETE FROM object WHERE id IN (
                    '00000000-0000-0000-0000-000000001001',
                    '00000000-0000-0000-0000-000000001002',
                    '00000000-0000-0000-0000-000000001003'
                 )",
            )
            .await?;

        let _ = db
            .execute_unprepared(
                "DELETE FROM module WHERE id = '00000000-0000-0000-0000-000000000100'",
            )
            .await?;

        let _ = db
            .execute_unprepared(
                "DELETE FROM link_type WHERE id IN (
                    '00000000-0000-0000-0000-000000000010',
                    '00000000-0000-0000-0000-000000000011'
                 )",
            )
            .await?;

        let _ = db
            .execute_unprepared(
                "DELETE FROM project WHERE id = '00000000-0000-0000-0000-000000000001'",
            )
            .await?;

        let _ = db
            .execute_unprepared(
                "DELETE FROM workspace WHERE id = '00000000-0000-0000-0000-000000000001'",
            )
            .await?;

        Ok(())
    }
}
