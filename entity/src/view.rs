use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "view")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub module_id: Uuid,
    pub name: String,
    pub is_default: bool,
    #[sea_orm(column_type = "JsonBinary")]
    pub column_config: serde_json::Value,
    #[sea_orm(column_type = "JsonBinary")]
    pub filter_config: serde_json::Value,
    #[sea_orm(column_type = "JsonBinary")]
    pub sort_config: serde_json::Value,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
