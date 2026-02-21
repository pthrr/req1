use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter, Set,
};
use serde::Deserialize;
use uuid::Uuid;

use entity::project;

use crate::PaginatedResponse;
use crate::error::CoreError;

#[derive(Debug, Deserialize)]
pub struct CreateProjectInput {
    #[serde(default)]
    pub workspace_id: Uuid,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProjectInput {
    pub name: Option<String>,
    pub description: Option<String>,
}

pub struct ProjectService;

impl ProjectService {
    pub async fn create(
        db: &impl ConnectionTrait,
        input: CreateProjectInput,
    ) -> Result<project::Model, CoreError> {
        let now = chrono::Utc::now().fixed_offset();
        let id = Uuid::now_v7();

        let model = project::ActiveModel {
            id: Set(id),
            workspace_id: Set(input.workspace_id),
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
        input: UpdateProjectInput,
    ) -> Result<project::Model, CoreError> {
        let existing = project::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("project {id} not found")))?;

        let mut active: project::ActiveModel = existing.into();
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
        let result = project::Entity::delete_by_id(id).exec(db).await?;
        if result.rows_affected == 0 {
            return Err(CoreError::NotFound(format!("project {id} not found")));
        }
        Ok(())
    }

    pub async fn get(db: &impl ConnectionTrait, id: Uuid) -> Result<project::Model, CoreError> {
        project::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("project {id} not found")))
    }

    pub async fn list(
        db: &impl ConnectionTrait,
        workspace_id: Uuid,
        offset: u64,
        limit: u64,
    ) -> Result<PaginatedResponse<project::Model>, CoreError> {
        let paginator = project::Entity::find()
            .filter(project::Column::WorkspaceId.eq(workspace_id))
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
