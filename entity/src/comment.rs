use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize, utoipa::ToSchema,
)]
#[schema(as = Comment)]
#[sea_orm(table_name = "comment")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub object_id: Uuid,
    pub author_id: Option<Uuid>,
    pub body: String,
    #[sea_orm(column_type = "JsonBinary")]
    #[schema(value_type = Vec<String>)]
    pub mentioned_user_ids: serde_json::Value,
    pub resolved: bool,
    #[schema(value_type = String)]
    pub created_at: DateTimeWithTimeZone,
    #[schema(value_type = String)]
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
