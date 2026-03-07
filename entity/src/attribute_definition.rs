use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, utoipa::ToSchema)]
#[schema(as = AttributeDefinition)]
#[sea_orm(table_name = "attribute_definition")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub module_id: Option<Uuid>,
    pub name: String,
    pub data_type: String,
    pub default_value: Option<String>,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    #[schema(value_type = Option<Vec<String>>)]
    pub enum_values: Option<serde_json::Value>,
    pub multi_select: bool,
    pub depends_on: Option<Uuid>,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    #[schema(value_type = Option<Object>)]
    pub dependency_mapping: Option<serde_json::Value>,
    #[schema(value_type = String)]
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
