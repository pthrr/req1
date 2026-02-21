use sea_orm::{ActiveModelTrait, ConnectionTrait, EntityTrait, PaginatorTrait, Set};
use serde::Deserialize;
use uuid::Uuid;

use entity::baseline_set;

use crate::PaginatedResponse;
use crate::error::CoreError;

#[derive(Debug, Deserialize)]
pub struct CreateBaselineSetInput {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub created_by: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBaselineSetInput {
    pub name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
}

pub struct BaselineSetService;

impl BaselineSetService {
    pub async fn create(
        db: &impl ConnectionTrait,
        input: CreateBaselineSetInput,
    ) -> Result<baseline_set::Model, CoreError> {
        let now = chrono::Utc::now().fixed_offset();
        let id = Uuid::now_v7();

        let model = baseline_set::ActiveModel {
            id: Set(id),
            name: Set(input.name),
            version: Set(input.version),
            description: Set(input.description),
            created_by: Set(input.created_by),
            created_at: Set(now),
        };

        let result = model.insert(db).await?;
        Ok(result)
    }

    pub async fn update(
        db: &impl ConnectionTrait,
        id: Uuid,
        input: UpdateBaselineSetInput,
    ) -> Result<baseline_set::Model, CoreError> {
        let existing = baseline_set::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("baseline_set {id} not found")))?;

        let mut active: baseline_set::ActiveModel = existing.into();
        if let Some(name) = input.name {
            active.name = Set(name);
        }
        if let Some(version) = input.version {
            active.version = Set(version);
        }
        if let Some(description) = input.description {
            active.description = Set(Some(description));
        }

        let result = active.update(db).await?;
        Ok(result)
    }

    pub async fn delete(db: &impl ConnectionTrait, id: Uuid) -> Result<(), CoreError> {
        let result = baseline_set::Entity::delete_by_id(id).exec(db).await?;
        if result.rows_affected == 0 {
            return Err(CoreError::NotFound(format!("baseline_set {id} not found")));
        }
        Ok(())
    }

    pub async fn get(
        db: &impl ConnectionTrait,
        id: Uuid,
    ) -> Result<baseline_set::Model, CoreError> {
        baseline_set::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("baseline_set {id} not found")))
    }

    pub async fn list(
        db: &impl ConnectionTrait,
        offset: u64,
        limit: u64,
    ) -> Result<PaginatedResponse<baseline_set::Model>, CoreError> {
        let paginator = baseline_set::Entity::find().paginate(db, limit);
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
