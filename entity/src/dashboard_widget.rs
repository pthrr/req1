use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "dashboard_widget")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub dashboard_id: Uuid,
    pub widget_type: String,
    pub title: String,
    #[sea_orm(column_type = "JsonBinary")]
    pub config: serde_json::Value,
    pub position_x: i32,
    pub position_y: i32,
    pub width: i32,
    pub height: i32,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
