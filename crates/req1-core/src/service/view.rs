use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter, Set,
};
use serde::Deserialize;
use uuid::Uuid;

use entity::view;

use crate::PaginatedResponse;
use crate::error::CoreError;

#[derive(Debug, Deserialize)]
pub struct CreateViewInput {
    #[serde(default)]
    pub module_id: Uuid,
    pub name: String,
    pub column_config: Option<serde_json::Value>,
    pub filter_config: Option<serde_json::Value>,
    pub sort_config: Option<serde_json::Value>,
    pub is_default: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateViewInput {
    pub name: Option<String>,
    pub column_config: Option<serde_json::Value>,
    pub filter_config: Option<serde_json::Value>,
    pub sort_config: Option<serde_json::Value>,
    pub is_default: Option<bool>,
}

pub struct ViewService;

impl ViewService {
    pub async fn create(
        db: &impl ConnectionTrait,
        input: CreateViewInput,
    ) -> Result<view::Model, CoreError> {
        let now = chrono::Utc::now().fixed_offset();
        let id = Uuid::now_v7();

        let model = view::ActiveModel {
            id: Set(id),
            module_id: Set(input.module_id),
            name: Set(input.name),
            column_config: Set(input.column_config.unwrap_or(serde_json::json!([]))),
            filter_config: Set(input.filter_config.unwrap_or(serde_json::json!({}))),
            sort_config: Set(input.sort_config.unwrap_or(serde_json::json!([]))),
            is_default: Set(input.is_default.unwrap_or(false)),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let result = model.insert(db).await?;
        Ok(result)
    }

    pub async fn update(
        db: &impl ConnectionTrait,
        id: Uuid,
        input: UpdateViewInput,
    ) -> Result<view::Model, CoreError> {
        let existing = view::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("view {id} not found")))?;

        let mut active: view::ActiveModel = existing.into();
        if let Some(name) = input.name {
            active.name = Set(name);
        }
        if let Some(column_config) = input.column_config {
            active.column_config = Set(column_config);
        }
        if let Some(filter_config) = input.filter_config {
            active.filter_config = Set(filter_config);
        }
        if let Some(sort_config) = input.sort_config {
            active.sort_config = Set(sort_config);
        }
        if let Some(is_default) = input.is_default {
            active.is_default = Set(is_default);
        }
        active.updated_at = Set(chrono::Utc::now().fixed_offset());

        let result = active.update(db).await?;
        Ok(result)
    }

    pub async fn delete(db: &impl ConnectionTrait, id: Uuid) -> Result<(), CoreError> {
        let result = view::Entity::delete_by_id(id).exec(db).await?;
        if result.rows_affected == 0 {
            return Err(CoreError::NotFound(format!("view {id} not found")));
        }
        Ok(())
    }

    pub async fn get(db: &impl ConnectionTrait, id: Uuid) -> Result<view::Model, CoreError> {
        view::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("view {id} not found")))
    }

    pub async fn list(
        db: &impl ConnectionTrait,
        module_id: Uuid,
        offset: u64,
        limit: u64,
    ) -> Result<PaginatedResponse<view::Model>, CoreError> {
        let paginator = view::Entity::find()
            .filter(view::Column::ModuleId.eq(module_id))
            .paginate(db, limit);
        let total = paginator.num_items().await?;
        let page = offset / limit;
        let items = paginator.fetch_page(page).await?;

        Ok(PaginatedResponse {
            items,
            total,
            offset,
            limit,
        })
    }
}
