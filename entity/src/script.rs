use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize, utoipa::ToSchema,
)]
#[schema(as = Script)]
#[sea_orm(table_name = "script")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub module_id: Uuid,
    pub name: String,
    /// `"trigger"`, `"layout"`, or `"action"`
    pub script_type: String,
    /// Required for triggers: `"pre_save"`, `"post_save"`, `"pre_delete"`, `"post_delete"`
    pub hook_point: Option<String>,
    #[sea_orm(column_type = "Text")]
    pub source_code: String,
    pub enabled: bool,
    pub priority: i32,
    pub cron_expression: Option<String>,
    #[schema(value_type = Option<String>)]
    pub last_run_at: Option<DateTimeWithTimeZone>,
    #[schema(value_type = Option<String>)]
    pub next_run_at: Option<DateTimeWithTimeZone>,
    #[schema(value_type = String)]
    pub created_at: DateTimeWithTimeZone,
    #[schema(value_type = String)]
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
