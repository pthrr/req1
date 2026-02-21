use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "link")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub source_object_id: Uuid,
    pub target_object_id: Uuid,
    pub link_type_id: Uuid,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub attributes: Option<serde_json::Value>,
    pub suspect: bool,
    pub source_fingerprint: String,
    pub target_fingerprint: String,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
