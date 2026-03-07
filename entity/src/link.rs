use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, utoipa::ToSchema)]
#[schema(as = Link)]
#[sea_orm(table_name = "link")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub source_object_id: Uuid,
    pub target_object_id: Uuid,
    pub link_type_id: Uuid,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    #[schema(value_type = Option<Object>)]
    pub attributes: Option<serde_json::Value>,
    pub suspect: bool,
    pub source_fingerprint: String,
    pub target_fingerprint: String,
    #[schema(value_type = String)]
    pub created_at: DateTimeWithTimeZone,
    #[schema(value_type = String)]
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
