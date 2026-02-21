use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE review_package (
                    id UUID PRIMARY KEY,
                    module_id UUID NOT NULL REFERENCES module(id) ON DELETE CASCADE,
                    name VARCHAR NOT NULL,
                    description TEXT,
                    status VARCHAR NOT NULL DEFAULT 'draft',
                    created_by UUID,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                    CONSTRAINT chk_review_status CHECK (status IN ('draft', 'in_review', 'approved', 'rejected'))
                )",
            )
            .await?;

        let _ = manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE review_assignment (
                    id UUID PRIMARY KEY,
                    package_id UUID NOT NULL REFERENCES review_package(id) ON DELETE CASCADE,
                    reviewer_id UUID,
                    status VARCHAR NOT NULL DEFAULT 'pending',
                    comment TEXT,
                    signed_at TIMESTAMPTZ,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                    CONSTRAINT chk_assignment_status CHECK (status IN ('pending', 'approved', 'rejected'))
                )",
            )
            .await?;

        let _ = manager
            .get_connection()
            .execute_unprepared(
                "CREATE INDEX idx_review_package_module ON review_package(module_id)",
            )
            .await?;

        let _ = manager
            .get_connection()
            .execute_unprepared(
                "CREATE INDEX idx_review_assignment_package ON review_assignment(package_id)",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS review_assignment")
            .await?;

        let _ = manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS review_package")
            .await?;

        Ok(())
    }
}
