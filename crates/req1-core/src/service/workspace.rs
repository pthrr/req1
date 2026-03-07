use sea_orm::{ActiveModelTrait, ConnectionTrait, EntityTrait, Set};
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

use entity::workspace;

use crate::crud_service;
use crate::error::CoreError;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateWorkspaceInput {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
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
            .ok_or_else(|| CoreError::not_found(format!("workspace {id} not found")))?;

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
}

crud_service!(WorkspaceService, workspace::Entity, "workspace");
