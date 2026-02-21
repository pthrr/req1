use sea_orm::{ActiveModelTrait, ConnectionTrait, EntityTrait, PaginatorTrait, Set};
use serde::Deserialize;
use uuid::Uuid;

use entity::workspace;

use crate::PaginatedResponse;
use crate::error::CoreError;

#[derive(Debug, Deserialize)]
pub struct CreateWorkspaceInput {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkspaceInput {
    pub name: Option<String>,
    pub description: Option<String>,
}

pub struct WorkspaceService;

impl WorkspaceService {
    pub async fn create(
        db: &impl ConnectionTrait,
        input: CreateWorkspaceInput,
    ) -> Result<workspace::Model, CoreError> {
        let now = chrono::Utc::now().fixed_offset();
        let id = Uuid::now_v7();

        let model = workspace::ActiveModel {
            id: Set(id),
            name: Set(input.name),
            description: Set(input.description),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let result = model.insert(db).await?;
        Ok(result)
    }

    pub async fn update(
        db: &impl ConnectionTrait,
        id: Uuid,
        input: UpdateWorkspaceInput,
    ) -> Result<workspace::Model, CoreError> {
        let existing = workspace::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("workspace {id} not found")))?;

        let mut active: workspace::ActiveModel = existing.into();
        if let Some(name) = input.name {
            active.name = Set(name);
        }
        if let Some(description) = input.description {
            active.description = Set(Some(description));
        }
        active.updated_at = Set(chrono::Utc::now().fixed_offset());

        let result = active.update(db).await?;
        Ok(result)
    }

    pub async fn delete(db: &impl ConnectionTrait, id: Uuid) -> Result<(), CoreError> {
        let result = workspace::Entity::delete_by_id(id).exec(db).await?;
        if result.rows_affected == 0 {
            return Err(CoreError::NotFound(format!("workspace {id} not found")));
        }
        Ok(())
    }

    pub async fn get(db: &impl ConnectionTrait, id: Uuid) -> Result<workspace::Model, CoreError> {
        workspace::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("workspace {id} not found")))
    }

    pub async fn list(
        db: &impl ConnectionTrait,
        offset: u64,
        limit: u64,
    ) -> Result<PaginatedResponse<workspace::Model>, CoreError> {
        let paginator = workspace::Entity::find().paginate(db, limit);
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
