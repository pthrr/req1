use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize, utoipa::ToSchema,
)]
#[schema(as = TestExecution)]
#[sea_orm(table_name = "test_execution")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub test_case_id: Uuid,
    pub status: String,
    pub executor: Option<String>,
    #[schema(value_type = Option<String>)]
    pub executed_at: Option<DateTimeWithTimeZone>,
    pub duration_ms: Option<i64>,
    pub evidence: Option<String>,
    pub environment: Option<String>,
    #[schema(value_type = String)]
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
