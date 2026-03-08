use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{DbBackend, Statement};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Only seed into an empty user table
        let result = db
            .query_one(Statement::from_string(
                DbBackend::Postgres,
                "SELECT EXISTS(SELECT 1 FROM app_user) AS has_users",
            ))
            .await?;
        let has_users: bool = result
            .and_then(|row| row.try_get("", "has_users").ok())
            .unwrap_or(false);
        if has_users {
            return Ok(());
        }

        // bcrypt hash of "admin" with cost 12
        let _ = db
            .execute_unprepared(
                "INSERT INTO app_user (id, email, display_name, role, password_hash, active)
                 VALUES (
                     '00000000-0000-0000-0000-000000000002',
                     'admin@localhost',
                     'Admin',
                     'admin',
                     '$2b$12$p0soPXdAWX9bebpYaMUshu6h.3wbyDa4MdwmOwtGGYmIVt2iLFMY.',
                     true
                 )",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let _ = db
            .execute_unprepared(
                "DELETE FROM app_user WHERE id = '00000000-0000-0000-0000-000000000002'",
            )
            .await?;

        Ok(())
    }
}
