use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "attribute_definition")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub module_id: Option<Uuid>,
    pub name: String,
    pub data_type: String,
    pub default_value: Option<String>,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub enum_values: Option<serde_json::Value>,
    pub multi_select: bool,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
