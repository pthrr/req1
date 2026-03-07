use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, utoipa::ToSchema)]
#[schema(as = Object)]
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
    #[schema(value_type = Option<Object>)]
    pub attributes: Option<serde_json::Value>,
    pub current_version: i32,
    pub classification: String,
    pub content_fingerprint: String,
    pub reviewed_fingerprint: Option<String>,
    #[schema(value_type = Option<String>)]
    pub reviewed_at: Option<DateTimeWithTimeZone>,
    pub reviewed_by: Option<Uuid>,
    #[sea_orm(column_name = "references_", column_type = "JsonBinary")]
    #[schema(value_type = Object)]
    pub references_: serde_json::Value,
    pub object_type_id: Option<Uuid>,
    pub lifecycle_state: Option<String>,
    pub lifecycle_model_id: Option<Uuid>,
    pub source_object_id: Option<Uuid>,
    pub source_module_id: Option<Uuid>,
    pub is_placeholder: bool,
    pub docx_source_id: Option<String>,
    #[schema(value_type = Option<String>)]
    pub deleted_at: Option<DateTimeWithTimeZone>,
    #[schema(value_type = String)]
    pub created_at: DateTimeWithTimeZone,
    #[schema(value_type = String)]
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
