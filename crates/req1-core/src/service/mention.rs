use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

use entity::{app_user, notification};

use crate::error::CoreError;

pub struct MentionService;

impl MentionService {
    /// Parse `@DisplayName` mentions in the body text, returning matching user IDs.
    pub async fn parse_mentions(
        db: &impl ConnectionTrait,
        body: &str,
    ) -> Result<Vec<Uuid>, CoreError> {
        let users = app_user::Entity::find()
            .filter(app_user::Column::Active.eq(true))
            .all(db)
            .await?;

        let lower_body = body.to_lowercase();
        let mut mentioned = Vec::new();
        for user in &users {
            let pattern = format!("@{}", user.display_name.to_lowercase());
            if lower_body.contains(&pattern) {
                mentioned.push(user.id);
            }
        }
        mentioned.dedup();
        Ok(mentioned)
    }

    /// Create a notification for each mentioned user (skip self-mentions).
    pub async fn notify_mentioned(
        db: &impl ConnectionTrait,
        mentioned_ids: &[Uuid],
        author_id: Option<Uuid>,
        entity_type: &str,
        entity_id: Uuid,
        body_preview: &str,
    ) -> Result<(), CoreError> {
        let preview = if body_preview.len() > 100 {
            format!("{}...", &body_preview[..100])
        } else {
            body_preview.to_owned()
        };

        let now = chrono::Utc::now().fixed_offset();
        for user_id in mentioned_ids {
            // Skip self-mentions
            if author_id == Some(*user_id) {
                continue;
            }

            let model = notification::ActiveModel {
                id: Set(Uuid::now_v7()),
                user_id: Set(*user_id),
                notification_type: Set("mention".to_owned()),
                title: Set(format!("You were mentioned in a {entity_type}")),
                body: Set(preview.clone()),
                entity_type: Set(entity_type.to_owned()),
                entity_id: Set(Some(entity_id)),
                read: Set(false),
                created_at: Set(now),
            };

            let _ = model.insert(db).await?;
        }

        Ok(())
    }
}
