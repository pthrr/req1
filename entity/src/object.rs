use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "object")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub module_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub position: i32,
    pub level: String,
    pub heading: Option<String>,
    pub body: Option<String>,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub attributes: Option<serde_json::Value>,
    pub current_version: i32,
    pub classification: String,
    pub content_fingerprint: String,
    pub reviewed_fingerprint: Option<String>,
    pub reviewed_at: Option<DateTimeWithTimeZone>,
    pub reviewed_by: Option<Uuid>,
    #[sea_orm(column_name = "references_", column_type = "JsonBinary")]
    pub references_: serde_json::Value,
    pub object_type_id: Option<Uuid>,
    pub deleted_at: Option<DateTimeWithTimeZone>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
