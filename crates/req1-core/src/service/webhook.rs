use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use entity::webhook;

use crate::error::CoreError;

#[derive(Debug, Deserialize)]
pub struct CreateWebhookInput {
    pub module_id: Uuid,
    pub name: String,
    pub url: String,
    pub secret: Option<String>,
    pub events: Option<String>,
    pub active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWebhookInput {
    pub name: Option<String>,
    pub url: Option<String>,
    pub secret: Option<String>,
    pub events: Option<String>,
    pub active: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct WebhookPayload {
    pub event: String,
    pub module_id: String,
    pub object_id: String,
    pub data: serde_json::Value,
}

pub struct WebhookService;

impl WebhookService {
    pub async fn create(
        db: &impl ConnectionTrait,
        input: CreateWebhookInput,
    ) -> Result<webhook::Model, CoreError> {
        let now = chrono::Utc::now().fixed_offset();
        let id = Uuid::now_v7();
        let events = input
            .events
            .unwrap_or_else(|| "object.created,object.updated,object.deleted".to_owned());

        let model = webhook::ActiveModel {
            id: Set(id),
            module_id: Set(input.module_id),
            name: Set(input.name),
            url: Set(input.url),
            secret: Set(input.secret),
            events: Set(events),
            active: Set(input.active.unwrap_or(true)),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let result = model.insert(db).await?;
        Ok(result)
    }

    pub async fn list(
        db: &impl ConnectionTrait,
        module_id: Uuid,
    ) -> Result<Vec<webhook::Model>, CoreError> {
        let items = webhook::Entity::find()
            .filter(webhook::Column::ModuleId.eq(module_id))
            .all(db)
            .await?;
        Ok(items)
    }

    pub async fn get(
        db: &impl ConnectionTrait,
        id: Uuid,
    ) -> Result<webhook::Model, CoreError> {
        webhook::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("webhook {id} not found")))
    }

    pub async fn update(
        db: &impl ConnectionTrait,
        id: Uuid,
        input: UpdateWebhookInput,
    ) -> Result<webhook::Model, CoreError> {
        let existing = Self::get(db, id).await?;
        let mut active: webhook::ActiveModel = existing.into();

        if let Some(name) = input.name {
            active.name = Set(name);
        }
        if let Some(url) = input.url {
            active.url = Set(url);
        }
        if let Some(secret) = input.secret {
            active.secret = Set(Some(secret));
        }
        if let Some(events) = input.events {
            active.events = Set(events);
        }
        if let Some(active_flag) = input.active {
            active.active = Set(active_flag);
        }
        active.updated_at = Set(chrono::Utc::now().fixed_offset());

        let result = active.update(db).await?;
        Ok(result)
    }

    pub async fn delete(db: &impl ConnectionTrait, id: Uuid) -> Result<(), CoreError> {
        let result = webhook::Entity::delete_by_id(id).exec(db).await?;
        if result.rows_affected == 0 {
            return Err(CoreError::NotFound(format!("webhook {id} not found")));
        }
        Ok(())
    }

    /// Fire webhooks for a given module/event. Spawns background tasks — does not block.
    pub async fn fire(
        db: &impl ConnectionTrait,
        module_id: Uuid,
        event: &str,
        object_id: Uuid,
        data: serde_json::Value,
    ) -> Result<(), CoreError> {
        let hooks = webhook::Entity::find()
            .filter(webhook::Column::ModuleId.eq(module_id))
            .filter(webhook::Column::Active.eq(true))
            .all(db)
            .await?;

        let event_owned = event.to_owned();
        for hook in hooks {
            // Check if the webhook is subscribed to this event
            let subscribed = hook
                .events
                .split(',')
                .any(|e| e.trim() == event_owned);
            if !subscribed {
                continue;
            }

            let payload = WebhookPayload {
                event: event_owned.clone(),
                module_id: module_id.to_string(),
                object_id: object_id.to_string(),
                data: data.clone(),
            };

            let url = hook.url.clone();
            let secret = hook.secret.clone();

            drop(tokio::spawn(async move {
                let client = reqwest::Client::new();
                let mut req = client.post(&url).json(&payload);
                if let Some(ref s) = secret {
                    req = req.header("X-Webhook-Secret", s);
                }
                if let Err(e) = req.send().await {
                    tracing::warn!("webhook delivery to {} failed: {e}", url);
                }
            }));
        }

        Ok(())
    }
}
