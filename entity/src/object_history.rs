use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "object_history")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub object_id: Uuid,
    pub module_id: Uuid,
    pub version: i32,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub attribute_values: Option<serde_json::Value>,
    pub heading: Option<String>,
    pub body: Option<String>,
    pub changed_by: Option<Uuid>,
    pub changed_at: DateTimeWithTimeZone,
    pub change_type: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
