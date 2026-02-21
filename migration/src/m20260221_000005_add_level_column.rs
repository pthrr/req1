use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Object::Table)
                    .add_column(
                        ColumnDef::new(Object::Level)
                            .string()
                            .not_null()
                            .default("0"),
                    )
                    .to_owned(),
            )
            .await?;

        let db = manager.get_connection();
        let _ = db
            .execute_unprepared(
                r"
WITH RECURSIVE tree(id, parent_id, module_id, position, level, rn) AS (
    SELECT id, parent_id, module_id, position,
           CAST(ROW_NUMBER() OVER (PARTITION BY module_id ORDER BY position) AS TEXT),
           ROW_NUMBER() OVER (PARTITION BY module_id ORDER BY position)
    FROM object WHERE parent_id IS NULL
    UNION ALL
    SELECT o.id, o.parent_id, o.module_id, o.position,
           t.level || '.' || CAST(ROW_NUMBER() OVER (PARTITION BY o.parent_id ORDER BY o.position) AS TEXT),
           ROW_NUMBER() OVER (PARTITION BY o.parent_id ORDER BY o.position)
    FROM object o JOIN tree t ON o.parent_id = t.id
)
UPDATE object SET level = tree.level FROM tree WHERE object.id = tree.id
",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Object::Table)
                    .drop_column(Object::Level)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Object {
    Table,
    Level,
}
