use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Order, Set,
};
use serde::Deserialize;
use uuid::Uuid;

use entity::review_comment;

use crate::PaginatedResponse;
use crate::error::CoreError;
use crate::service::mention::MentionService;

#[derive(Debug, Deserialize)]
pub struct CreateReviewCommentInput {
    #[serde(default)]
    pub package_id: Uuid,
    pub author_id: Option<Uuid>,
    pub body: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateReviewCommentInput {
    pub body: Option<String>,
}

pub struct ReviewCommentService;

impl ReviewCommentService {
    pub async fn create(
        db: &impl ConnectionTrait,
        input: CreateReviewCommentInput,
    ) -> Result<review_comment::Model, CoreError> {
        let now = chrono::Utc::now().fixed_offset();
        let id = Uuid::now_v7();

        let mentioned_ids = MentionService::parse_mentions(db, &input.body).await?;
        let mentioned_json = serde_json::json!(mentioned_ids);

        let body_text = input.body.clone();
        let author = input.author_id;
        let model = review_comment::ActiveModel {
            id: Set(id),
            package_id: Set(input.package_id),
            author_id: Set(author),
            body: Set(input.body),
            mentioned_user_ids: Set(mentioned_json),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let result = model.insert(db).await?;

        if !mentioned_ids.is_empty() {
            MentionService::notify_mentioned(
                db,
                &mentioned_ids,
                author,
                "review_comment",
                id,
                &body_text,
            )
            .await?;
        }

        Ok(result)
    }

    pub async fn update(
        db: &impl ConnectionTrait,
        id: Uuid,
        input: UpdateReviewCommentInput,
    ) -> Result<review_comment::Model, CoreError> {
        let existing = review_comment::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("review_comment {id} not found")))?;

        let mut active: review_comment::ActiveModel = existing.into();
        if let Some(body) = input.body {
            active.body = Set(body);
        }
        active.updated_at = Set(chrono::Utc::now().fixed_offset());

        let result = active.update(db).await?;
        Ok(result)
    }

    pub async fn delete(db: &impl ConnectionTrait, id: Uuid) -> Result<(), CoreError> {
        let result = review_comment::Entity::delete_by_id(id).exec(db).await?;
        if result.rows_affected == 0 {
            return Err(CoreError::NotFound(format!(
                "review_comment {id} not found"
            )));
        }
        Ok(())
    }

    pub async fn get(
        db: &impl ConnectionTrait,
        id: Uuid,
    ) -> Result<review_comment::Model, CoreError> {
        review_comment::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("review_comment {id} not found")))
    }

    pub async fn list(
        db: &impl ConnectionTrait,
        package_id: Uuid,
        offset: u64,
        limit: u64,
    ) -> Result<PaginatedResponse<review_comment::Model>, CoreError> {
        let paginator = review_comment::Entity::find()
            .filter(review_comment::Column::PackageId.eq(package_id))
            .order_by(review_comment::Column::CreatedAt, Order::Asc)
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
