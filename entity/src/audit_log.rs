use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "audit_log")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Option<Uuid>,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub details: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
