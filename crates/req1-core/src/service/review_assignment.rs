use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter, Set,
};
use serde::Deserialize;
use uuid::Uuid;

use entity::review_assignment;

use crate::PaginatedResponse;
use crate::error::CoreError;

const VALID_STATUSES: &[&str] = &["pending", "approved", "rejected", "abstained"];

#[derive(Debug, Deserialize)]
pub struct CreateReviewAssignmentInput {
    #[serde(default)]
    pub package_id: Uuid,
    pub reviewer_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateReviewAssignmentInput {
    pub status: Option<String>,
    pub comment: Option<String>,
}

pub struct ReviewAssignmentService;

impl ReviewAssignmentService {
    pub async fn create(
        db: &impl ConnectionTrait,
        input: CreateReviewAssignmentInput,
    ) -> Result<review_assignment::Model, CoreError> {
        let now = chrono::Utc::now().fixed_offset();
        let id = Uuid::now_v7();

        let model = review_assignment::ActiveModel {
            id: Set(id),
            package_id: Set(input.package_id),
            reviewer_id: Set(input.reviewer_id),
            status: Set("pending".to_owned()),
            comment: Set(None),
            signed_at: Set(None),
            created_at: Set(now),
        };

        let result = model.insert(db).await?;
        Ok(result)
    }

    pub async fn update(
        db: &impl ConnectionTrait,
        id: Uuid,
        input: UpdateReviewAssignmentInput,
    ) -> Result<review_assignment::Model, CoreError> {
        let existing = review_assignment::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("review_assignment {id} not found")))?;

        let mut active: review_assignment::ActiveModel = existing.into();
        if let Some(ref status) = input.status {
            if !VALID_STATUSES.contains(&status.as_str()) {
                return Err(CoreError::BadRequest(format!(
                    "invalid status '{status}', must be one of: {VALID_STATUSES:?}"
                )));
            }
            active.status = Set(status.clone());
            if matches!(status.as_str(), "approved" | "rejected" | "abstained") {
                active.signed_at = Set(Some(chrono::Utc::now().fixed_offset()));
            }
        }
        if let Some(comment) = input.comment {
            active.comment = Set(Some(comment));
        }

        let result = active.update(db).await?;
        Ok(result)
    }

    pub async fn delete(db: &impl ConnectionTrait, id: Uuid) -> Result<(), CoreError> {
        let result = review_assignment::Entity::delete_by_id(id).exec(db).await?;
        if result.rows_affected == 0 {
            return Err(CoreError::NotFound(format!(
                "review_assignment {id} not found"
            )));
        }
        Ok(())
    }

    pub async fn get(
        db: &impl ConnectionTrait,
        id: Uuid,
    ) -> Result<review_assignment::Model, CoreError> {
        review_assignment::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("review_assignment {id} not found")))
    }

    pub async fn list(
        db: &impl ConnectionTrait,
        package_id: Uuid,
        offset: u64,
        limit: u64,
    ) -> Result<PaginatedResponse<review_assignment::Model>, CoreError> {
        let paginator = review_assignment::Entity::find()
            .filter(review_assignment::Column::PackageId.eq(package_id))
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
