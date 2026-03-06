use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "diagram")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub module_id: Uuid,
    pub name: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub description: Option<String>,
    pub diagram_type: String,
    #[sea_orm(column_type = "Text")]
    pub source_code: String,
    #[sea_orm(column_type = "JsonBinary")]
    pub linked_object_ids: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
