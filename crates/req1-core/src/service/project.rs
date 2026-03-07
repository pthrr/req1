use sea_orm::{ActiveModelTrait, ConnectionTrait, EntityTrait, Set};
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

use entity::project;

use crate::crud_service;
use crate::error::CoreError;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateProjectInput {
    #[serde(default)]
    pub workspace_id: Uuid,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
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
            .ok_or_else(|| CoreError::not_found(format!("project {id} not found")))?;

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
}

crud_service!(ProjectService, project::Entity, "project", parent: project::Column::WorkspaceId);
