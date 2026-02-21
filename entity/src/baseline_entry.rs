use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "baseline_entry")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub baseline_id: Uuid,
    #[sea_orm(primary_key, auto_increment = false)]
    pub object_id: Uuid,
    pub version: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
