use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter, Set,
};
use serde::Deserialize;
use uuid::Uuid;

use entity::comment;

use crate::PaginatedResponse;
use crate::error::CoreError;

#[derive(Debug, Deserialize)]
pub struct CreateCommentInput {
    #[serde(default)]
    pub object_id: Uuid,
    pub body: String,
    pub author_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCommentInput {
    pub body: Option<String>,
    pub resolved: Option<bool>,
}

pub struct CommentService;

impl CommentService {
    pub async fn create(
        db: &impl ConnectionTrait,
        input: CreateCommentInput,
    ) -> Result<comment::Model, CoreError> {
        let now = chrono::Utc::now().fixed_offset();
        let id = Uuid::now_v7();

        let model = comment::ActiveModel {
            id: Set(id),
            object_id: Set(input.object_id),
            author_id: Set(input.author_id),
            body: Set(input.body),
            resolved: Set(false),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let result = model.insert(db).await?;
        Ok(result)
    }

    pub async fn update(
        db: &impl ConnectionTrait,
        id: Uuid,
        input: UpdateCommentInput,
    ) -> Result<comment::Model, CoreError> {
        let existing = comment::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("comment {id} not found")))?;

        let mut active: comment::ActiveModel = existing.into();
        if let Some(body) = input.body {
            active.body = Set(body);
        }
        if let Some(resolved) = input.resolved {
            active.resolved = Set(resolved);
        }
        active.updated_at = Set(chrono::Utc::now().fixed_offset());

        let result = active.update(db).await?;
        Ok(result)
    }

    pub async fn delete(db: &impl ConnectionTrait, id: Uuid) -> Result<(), CoreError> {
        let result = comment::Entity::delete_by_id(id).exec(db).await?;
        if result.rows_affected == 0 {
            return Err(CoreError::NotFound(format!("comment {id} not found")));
        }
        Ok(())
    }

    pub async fn get(db: &impl ConnectionTrait, id: Uuid) -> Result<comment::Model, CoreError> {
        comment::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("comment {id} not found")))
    }

    pub async fn list(
        db: &impl ConnectionTrait,
        object_id: Uuid,
        offset: u64,
        limit: u64,
    ) -> Result<PaginatedResponse<comment::Model>, CoreError> {
        let paginator = comment::Entity::find()
            .filter(comment::Column::ObjectId.eq(object_id))
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
