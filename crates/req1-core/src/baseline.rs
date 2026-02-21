use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

use entity::{baseline_entry, object};

use crate::error::CoreError;

pub async fn snapshot_baseline(
    db: &impl ConnectionTrait,
    baseline_id: Uuid,
    module_id: Uuid,
) -> Result<Vec<baseline_entry::Model>, CoreError> {
    let objects = object::Entity::find()
        .filter(object::Column::ModuleId.eq(module_id))
        .all(db)
        .await?;

    let mut entries = Vec::with_capacity(objects.len());
    for obj in &objects {
        let entry = baseline_entry::ActiveModel {
            baseline_id: Set(baseline_id),
            object_id: Set(obj.id),
            version: Set(obj.current_version),
        };
        let inserted = entry.insert(db).await?;
        entries.push(inserted);
    }

    Ok(entries)
}
