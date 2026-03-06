use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "script_execution")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub script_id: Uuid,
    pub status: String,
    pub started_at: DateTimeWithTimeZone,
    pub finished_at: Option<DateTimeWithTimeZone>,
    pub duration_ms: Option<i64>,
    #[sea_orm(column_type = "Text", nullable)]
    pub output: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub error_message: Option<String>,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
