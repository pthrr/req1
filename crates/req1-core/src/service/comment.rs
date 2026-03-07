use sea_orm::{ActiveModelTrait, ConnectionTrait, EntityTrait, Set};
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

use entity::comment;

use crate::crud_service;
use crate::error::CoreError;
use crate::service::mention::MentionService;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateCommentInput {
    #[serde(default)]
    pub object_id: Uuid,
    pub body: String,
    pub author_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, ToSchema)]
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

        let mentioned_ids = MentionService::parse_mentions(db, &input.body).await?;
        let mentioned_json = serde_json::json!(mentioned_ids);

        let body_text = input.body.clone();
        let author = input.author_id;
        let model = comment::ActiveModel {
            id: Set(id),
            object_id: Set(input.object_id),
            author_id: Set(author),
            body: Set(input.body),
            mentioned_user_ids: Set(mentioned_json),
            resolved: Set(false),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let result = model.insert(db).await?;

        if !mentioned_ids.is_empty() {
            MentionService::notify_mentioned(db, &mentioned_ids, author, "comment", id, &body_text)
                .await?;
        }

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
            .ok_or_else(|| CoreError::not_found(format!("comment {id} not found")))?;

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
}

crud_service!(CommentService, comment::Entity, "comment", parent: comment::Column::ObjectId);
