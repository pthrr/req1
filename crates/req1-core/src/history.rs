use sea_orm::{ActiveModelTrait, ConnectionTrait, NotSet, Set};
use uuid::Uuid;

use entity::object_history;

use crate::error::CoreError;

pub struct HistoryEntry {
    pub object_id: Uuid,
    pub module_id: Uuid,
    pub version: i32,
    pub attribute_values: Option<serde_json::Value>,
    pub heading: Option<String>,
    pub body: Option<String>,
    pub change_type: String,
}

pub async fn insert_history(
    db: &impl ConnectionTrait,
    entry: HistoryEntry,
) -> Result<(), CoreError> {
    let record = object_history::ActiveModel {
        id: NotSet,
        object_id: Set(entry.object_id),
        module_id: Set(entry.module_id),
        version: Set(entry.version),
        attribute_values: Set(entry.attribute_values),
        heading: Set(entry.heading),
        body: Set(entry.body),
        changed_by: Set(None),
        changed_at: Set(chrono::Utc::now().fixed_offset()),
        change_type: Set(entry.change_type),
    };
    let _ = record.insert(db).await?;
    Ok(())
}
