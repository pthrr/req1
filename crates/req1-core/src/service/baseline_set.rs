use sea_orm::{ActiveModelTrait, ConnectionTrait, EntityTrait, Set};
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

use entity::baseline_set;

use crate::crud_service;
use crate::error::CoreError;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateBaselineSetInput {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub created_by: Option<Uuid>,
}

#[derive(Debug, Deserialize, ToSchema)]
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
            .ok_or_else(|| CoreError::not_found(format!("baseline_set {id} not found")))?;

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
}

crud_service!(BaselineSetService, baseline_set::Entity, "baseline_set");
