use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize, utoipa::ToSchema,
)]
#[schema(as = ChangeProposal)]
#[sea_orm(table_name = "change_proposal")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub module_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub author_id: Option<Uuid>,
    #[sea_orm(column_type = "JsonBinary")]
    #[schema(value_type = Object)]
    pub diff_data: serde_json::Value,
    #[schema(value_type = String)]
    pub created_at: DateTimeWithTimeZone,
    #[schema(value_type = String)]
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
