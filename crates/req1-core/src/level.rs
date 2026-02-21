use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};
use uuid::Uuid;

use crate::error::CoreError;

pub async fn recompute_module_levels(
    db: &impl ConnectionTrait,
    module_id: Uuid,
) -> Result<(), CoreError> {
    let sql = r"
WITH RECURSIVE tree(id, parent_id, module_id, position, level, rn) AS (
    SELECT id, parent_id, module_id, position,
           CAST(ROW_NUMBER() OVER (PARTITION BY module_id ORDER BY position) AS TEXT),
           ROW_NUMBER() OVER (PARTITION BY module_id ORDER BY position)
    FROM object WHERE parent_id IS NULL AND module_id = $1
    UNION ALL
    SELECT o.id, o.parent_id, o.module_id, o.position,
           t.level || '.' || CAST(ROW_NUMBER() OVER (PARTITION BY o.parent_id ORDER BY o.position) AS TEXT),
           ROW_NUMBER() OVER (PARTITION BY o.parent_id ORDER BY o.position)
    FROM object o JOIN tree t ON o.parent_id = t.id
    WHERE o.module_id = $1
)
UPDATE object SET level = tree.level FROM tree WHERE object.id = tree.id
";

    let _ = db
        .execute(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            sql,
            [module_id.into()],
        ))
        .await?;

    Ok(())
}
