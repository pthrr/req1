use sea_orm::{ActiveModelTrait, ConnectionTrait, EntityTrait, Set};
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

use entity::change_proposal;

use crate::crud_service;
use crate::error::CoreError;

const VALID_STATUSES: &[&str] = &[
    "draft",
    "submitted",
    "in_review",
    "approved",
    "rejected",
    "applied",
];

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateChangeProposalInput {
    #[serde(default)]
    pub module_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub author_id: Option<Uuid>,
    #[schema(value_type = Option<Object>)]
    pub diff_data: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateChangeProposalInput {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    #[schema(value_type = Option<Object>)]
    pub diff_data: Option<serde_json::Value>,
}

pub struct ChangeProposalService;

impl ChangeProposalService {
    pub async fn create(
        db: &impl ConnectionTrait,
        input: CreateChangeProposalInput,
    ) -> Result<change_proposal::Model, CoreError> {
        let now = chrono::Utc::now().fixed_offset();
        let id = Uuid::now_v7();

        let model = change_proposal::ActiveModel {
            id: Set(id),
            module_id: Set(input.module_id),
            title: Set(input.title),
            description: Set(input.description),
            status: Set("draft".to_owned()),
            author_id: Set(input.author_id),
            diff_data: Set(input.diff_data.unwrap_or(serde_json::json!({}))),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let result = model.insert(db).await?;
        Ok(result)
    }

    pub async fn update(
        db: &impl ConnectionTrait,
        id: Uuid,
        input: UpdateChangeProposalInput,
    ) -> Result<change_proposal::Model, CoreError> {
        let existing = change_proposal::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::not_found(format!("change_proposal {id} not found")))?;

        let mut active: change_proposal::ActiveModel = existing.into();
        if let Some(title) = input.title {
            active.title = Set(title);
        }
        if let Some(description) = input.description {
            active.description = Set(Some(description));
        }
        if let Some(ref status) = input.status {
            if !VALID_STATUSES.contains(&status.as_str()) {
                return Err(CoreError::bad_request(format!(
                    "invalid status '{status}', must be one of: {VALID_STATUSES:?}"
                )));
            }
            active.status = Set(status.clone());
        }
        if let Some(diff_data) = input.diff_data {
            active.diff_data = Set(diff_data);
        }
        active.updated_at = Set(chrono::Utc::now().fixed_offset());

        let result = active.update(db).await?;
        Ok(result)
    }
}

crud_service!(
    ChangeProposalService,
    change_proposal::Entity,
    "change_proposal",
    parent: change_proposal::Column::ModuleId
);
