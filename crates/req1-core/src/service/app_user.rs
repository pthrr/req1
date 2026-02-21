use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter, Set,
};
use serde::Deserialize;
use uuid::Uuid;

use entity::app_user;

use crate::PaginatedResponse;
use crate::error::CoreError;

const VALID_ROLES: &[&str] = &["admin", "editor", "reviewer", "viewer"];

#[derive(Debug, Deserialize)]
pub struct CreateAppUserInput {
    pub email: String,
    pub display_name: String,
    pub role: Option<String>,
    pub active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAppUserInput {
    pub display_name: Option<String>,
    pub role: Option<String>,
    pub active: Option<bool>,
}

const fn default_limit() -> u64 {
    50
}

#[derive(Debug, Deserialize)]
pub struct ListAppUsersFilter {
    #[serde(default)]
    pub offset: u64,
    #[serde(default = "default_limit")]
    pub limit: u64,
    pub active: Option<bool>,
}

pub struct AppUserService;

impl AppUserService {
    pub async fn create(
        db: &impl ConnectionTrait,
        input: CreateAppUserInput,
    ) -> Result<app_user::Model, CoreError> {
        let now = chrono::Utc::now().fixed_offset();
        let id = Uuid::now_v7();

        let role = input.role.unwrap_or_else(|| "viewer".to_owned());
        if !VALID_ROLES.contains(&role.as_str()) {
            return Err(CoreError::BadRequest(format!(
                "invalid role '{role}', must be one of: {VALID_ROLES:?}"
            )));
        }

        let model = app_user::ActiveModel {
            id: Set(id),
            email: Set(input.email),
            display_name: Set(input.display_name),
            role: Set(role),
            active: Set(input.active.unwrap_or(true)),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let result = model.insert(db).await?;
        Ok(result)
    }

    pub async fn update(
        db: &impl ConnectionTrait,
        id: Uuid,
        input: UpdateAppUserInput,
    ) -> Result<app_user::Model, CoreError> {
        let existing = app_user::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("app_user {id} not found")))?;

        let mut active: app_user::ActiveModel = existing.into();
        if let Some(display_name) = input.display_name {
            active.display_name = Set(display_name);
        }
        if let Some(ref role) = input.role {
            if !VALID_ROLES.contains(&role.as_str()) {
                return Err(CoreError::BadRequest(format!(
                    "invalid role '{role}', must be one of: {VALID_ROLES:?}"
                )));
            }
            active.role = Set(role.clone());
        }
        if let Some(is_active) = input.active {
            active.active = Set(is_active);
        }
        active.updated_at = Set(chrono::Utc::now().fixed_offset());

        let result = active.update(db).await?;
        Ok(result)
    }

    pub async fn delete(db: &impl ConnectionTrait, id: Uuid) -> Result<(), CoreError> {
        let result = app_user::Entity::delete_by_id(id).exec(db).await?;
        if result.rows_affected == 0 {
            return Err(CoreError::NotFound(format!("app_user {id} not found")));
        }
        Ok(())
    }

    pub async fn get(db: &impl ConnectionTrait, id: Uuid) -> Result<app_user::Model, CoreError> {
        app_user::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("app_user {id} not found")))
    }

    pub async fn list(
        db: &impl ConnectionTrait,
        filter: ListAppUsersFilter,
    ) -> Result<PaginatedResponse<app_user::Model>, CoreError> {
        let mut select = app_user::Entity::find();
        if let Some(is_active) = filter.active {
            select = select.filter(app_user::Column::Active.eq(is_active));
        }
        let paginator = select.paginate(db, filter.limit);
        let total = paginator.num_items().await?;
        let page = filter.offset / filter.limit;
        let items = paginator.fetch_page(page).await?;

        Ok(PaginatedResponse {
            items,
            total,
            offset: filter.offset,
            limit: filter.limit,
        })
    }
}
