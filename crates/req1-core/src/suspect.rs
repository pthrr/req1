use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};
use uuid::Uuid;

use crate::error::CoreError;

/// Flag links as suspect when the object's content fingerprint has changed
/// since the link was created/last resolved.
pub async fn flag_suspect_links(
    db: &impl ConnectionTrait,
    object_id: Uuid,
    new_fingerprint: &str,
) -> Result<(), CoreError> {
    let sql = r"
UPDATE link SET suspect = true, updated_at = NOW()
WHERE suspect = false
  AND (
    (source_object_id = $1 AND source_fingerprint <> $2)
    OR (target_object_id = $1 AND target_fingerprint <> $2)
  )
";

    let _ = db
        .execute(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            sql,
            [object_id.into(), new_fingerprint.into()],
        ))
        .await?;

    Ok(())
}
