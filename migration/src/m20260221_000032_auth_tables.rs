use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let _ = db
            .execute_unprepared("ALTER TABLE app_user ADD COLUMN password_hash VARCHAR")
            .await?;

        let _ = db
            .execute_unprepared(
                "CREATE TABLE workspace_member (
                    id UUID PRIMARY KEY,
                    user_id UUID NOT NULL REFERENCES app_user(id) ON DELETE CASCADE,
                    workspace_id UUID NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
                    role VARCHAR NOT NULL DEFAULT 'viewer',
                    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                    UNIQUE(user_id, workspace_id)
                )",
            )
            .await?;

        let _ = db
            .execute_unprepared(
                "CREATE INDEX idx_workspace_member_user ON workspace_member(user_id)",
            )
            .await?;

        let _ = db
            .execute_unprepared(
                "CREATE INDEX idx_workspace_member_workspace ON workspace_member(workspace_id)",
            )
            .await?;

        let _ = db
            .execute_unprepared(
                "CREATE TABLE module_permission (
                    id UUID PRIMARY KEY,
                    user_id UUID NOT NULL REFERENCES app_user(id) ON DELETE CASCADE,
                    module_id UUID NOT NULL REFERENCES module(id) ON DELETE CASCADE,
                    permission VARCHAR NOT NULL DEFAULT 'read',
                    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                    UNIQUE(user_id, module_id)
                )",
            )
            .await?;

        let _ = db
            .execute_unprepared(
                "CREATE INDEX idx_module_permission_user ON module_permission(user_id)",
            )
            .await?;

        let _ = db
            .execute_unprepared(
                "CREATE INDEX idx_module_permission_module ON module_permission(module_id)",
            )
            .await?;

        let _ = db
            .execute_unprepared(
                "CREATE TABLE audit_log (
                    id BIGSERIAL PRIMARY KEY,
                    user_id UUID REFERENCES app_user(id) ON DELETE SET NULL,
                    action VARCHAR NOT NULL,
                    entity_type VARCHAR NOT NULL,
                    entity_id UUID,
                    details JSONB,
                    ip_address VARCHAR,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
                )",
            )
            .await?;

        let _ = db
            .execute_unprepared("CREATE INDEX idx_audit_log_user ON audit_log(user_id)")
            .await?;

        let _ = db
            .execute_unprepared(
                "CREATE INDEX idx_audit_log_entity ON audit_log(entity_type, entity_id)",
            )
            .await?;

        let _ = db
            .execute_unprepared("CREATE INDEX idx_audit_log_created ON audit_log(created_at)")
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let _ = db.execute_unprepared("DROP TABLE IF EXISTS audit_log").await?;
        let _ = db
            .execute_unprepared("DROP TABLE IF EXISTS module_permission")
            .await?;
        let _ = db
            .execute_unprepared("DROP TABLE IF EXISTS workspace_member")
            .await?;
        let _ = db
            .execute_unprepared("ALTER TABLE app_user DROP COLUMN IF EXISTS password_hash")
            .await?;

        Ok(())
    }
}
