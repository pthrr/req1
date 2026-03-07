use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize, utoipa::ToSchema,
)]
#[schema(as = TestCase)]
#[sea_orm(table_name = "test_case")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub module_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub preconditions: Option<String>,
    pub expected_result: Option<String>,
    pub test_type: String,
    pub priority: String,
    pub status: String,
    #[sea_orm(column_type = "JsonBinary")]
    #[schema(value_type = Vec<String>)]
    pub requirement_ids: serde_json::Value,
    #[schema(value_type = String)]
    pub created_at: DateTimeWithTimeZone,
    #[schema(value_type = String)]
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
