use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter, Set,
};
use serde::Deserialize;
use uuid::Uuid;

use entity::change_proposal;

use crate::PaginatedResponse;
use crate::error::CoreError;

const VALID_STATUSES: &[&str] = &[
    "draft",
    "submitted",
    "in_review",
    "approved",
    "rejected",
    "applied",
];

#[derive(Debug, Deserialize)]
pub struct CreateChangeProposalInput {
    #[serde(default)]
    pub module_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub author_id: Option<Uuid>,
    pub diff_data: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateChangeProposalInput {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
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
            .ok_or_else(|| CoreError::NotFound(format!("change_proposal {id} not found")))?;

        let mut active: change_proposal::ActiveModel = existing.into();
        if let Some(title) = input.title {
            active.title = Set(title);
        }
        if let Some(description) = input.description {
            active.description = Set(Some(description));
        }
        if let Some(ref status) = input.status {
            if !VALID_STATUSES.contains(&status.as_str()) {
                return Err(CoreError::BadRequest(format!(
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

    pub async fn delete(db: &impl ConnectionTrait, id: Uuid) -> Result<(), CoreError> {
        let result = change_proposal::Entity::delete_by_id(id).exec(db).await?;
        if result.rows_affected == 0 {
            return Err(CoreError::NotFound(format!(
                "change_proposal {id} not found"
            )));
        }
        Ok(())
    }

    pub async fn get(
        db: &impl ConnectionTrait,
        id: Uuid,
    ) -> Result<change_proposal::Model, CoreError> {
        change_proposal::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("change_proposal {id} not found")))
    }

    pub async fn list(
        db: &impl ConnectionTrait,
        module_id: Uuid,
        offset: u64,
        limit: u64,
    ) -> Result<PaginatedResponse<change_proposal::Model>, CoreError> {
        let paginator = change_proposal::Entity::find()
            .filter(change_proposal::Column::ModuleId.eq(module_id))
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
