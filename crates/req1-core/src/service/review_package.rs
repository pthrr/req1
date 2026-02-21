use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter, Set,
};
use serde::Deserialize;
use uuid::Uuid;

use entity::review_package;

use crate::PaginatedResponse;
use crate::error::CoreError;

const VALID_STATUSES: &[&str] = &[
    "draft",
    "open",
    "in_review",
    "approved",
    "rejected",
    "closed",
];

#[derive(Debug, Deserialize)]
pub struct CreateReviewPackageInput {
    #[serde(default)]
    pub module_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_by: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateReviewPackageInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
}

pub struct ReviewPackageService;

impl ReviewPackageService {
    pub async fn create(
        db: &impl ConnectionTrait,
        input: CreateReviewPackageInput,
    ) -> Result<review_package::Model, CoreError> {
        let now = chrono::Utc::now().fixed_offset();
        let id = Uuid::now_v7();

        let model = review_package::ActiveModel {
            id: Set(id),
            module_id: Set(input.module_id),
            name: Set(input.name),
            description: Set(input.description),
            status: Set("draft".to_owned()),
            created_by: Set(input.created_by),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let result = model.insert(db).await?;
        Ok(result)
    }

    pub async fn update(
        db: &impl ConnectionTrait,
        id: Uuid,
        input: UpdateReviewPackageInput,
    ) -> Result<review_package::Model, CoreError> {
        let existing = review_package::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("review_package {id} not found")))?;

        let mut active: review_package::ActiveModel = existing.into();
        if let Some(name) = input.name {
            active.name = Set(name);
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
        active.updated_at = Set(chrono::Utc::now().fixed_offset());

        let result = active.update(db).await?;
        Ok(result)
    }

    pub async fn delete(db: &impl ConnectionTrait, id: Uuid) -> Result<(), CoreError> {
        let result = review_package::Entity::delete_by_id(id).exec(db).await?;
        if result.rows_affected == 0 {
            return Err(CoreError::NotFound(format!(
                "review_package {id} not found"
            )));
        }
        Ok(())
    }

    pub async fn get(
        db: &impl ConnectionTrait,
        id: Uuid,
    ) -> Result<review_package::Model, CoreError> {
        review_package::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("review_package {id} not found")))
    }

    pub async fn list(
        db: &impl ConnectionTrait,
        module_id: Uuid,
        offset: u64,
        limit: u64,
    ) -> Result<PaginatedResponse<review_package::Model>, CoreError> {
        let paginator = review_package::Entity::find()
            .filter(review_package::Column::ModuleId.eq(module_id))
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
