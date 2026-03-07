use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use entity::lifecycle_model;

use crate::error::CoreError;

#[derive(Debug, Deserialize, Serialize, Clone, ToSchema)]
pub struct LifecycleState {
    pub name: String,
    pub color: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, ToSchema)]
pub struct LifecycleTransition {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateLifecycleModelInput {
    pub name: String,
    pub description: Option<String>,
    pub initial_state: Option<String>,
    pub states: Option<Vec<LifecycleState>>,
    pub transitions: Option<Vec<LifecycleTransition>>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateLifecycleModelInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub initial_state: Option<String>,
    pub states: Option<Vec<LifecycleState>>,
    pub transitions: Option<Vec<LifecycleTransition>>,
}

pub struct LifecycleService;

impl LifecycleService {
    pub async fn create(
        db: &impl ConnectionTrait,
        module_id: Uuid,
        input: CreateLifecycleModelInput,
    ) -> Result<lifecycle_model::Model, CoreError> {
        let now = chrono::Utc::now().fixed_offset();
        let id = Uuid::now_v7();

        let states_json = serde_json::to_value(input.states.unwrap_or_default())
            .map_err(|e| CoreError::internal(format!("json error: {e}")))?;
        let transitions_json = serde_json::to_value(input.transitions.unwrap_or_default())
            .map_err(|e| CoreError::internal(format!("json error: {e}")))?;

        let model = lifecycle_model::ActiveModel {
            id: Set(id),
            module_id: Set(module_id),
            name: Set(input.name),
            description: Set(input.description),
            initial_state: Set(input.initial_state.unwrap_or_else(|| "new".to_owned())),
            states: Set(states_json),
            transitions: Set(transitions_json),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let result = model.insert(db).await?;
        Ok(result)
    }

    pub async fn list(
        db: &impl ConnectionTrait,
        module_id: Uuid,
    ) -> Result<Vec<lifecycle_model::Model>, CoreError> {
        let items = lifecycle_model::Entity::find()
            .filter(lifecycle_model::Column::ModuleId.eq(module_id))
            .all(db)
            .await?;
        Ok(items)
    }

    pub async fn get(
        db: &impl ConnectionTrait,
        id: Uuid,
    ) -> Result<lifecycle_model::Model, CoreError> {
        lifecycle_model::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::not_found(format!("lifecycle model {id} not found")))
    }

    pub async fn update(
        db: &impl ConnectionTrait,
        id: Uuid,
        input: UpdateLifecycleModelInput,
    ) -> Result<lifecycle_model::Model, CoreError> {
        let existing = lifecycle_model::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::not_found(format!("lifecycle model {id} not found")))?;

        let mut active: lifecycle_model::ActiveModel = existing.into();

        if let Some(name) = input.name {
            active.name = Set(name);
        }
        if let Some(description) = input.description {
            active.description = Set(Some(description));
        }
        if let Some(initial_state) = input.initial_state {
            active.initial_state = Set(initial_state);
        }
        if let Some(states) = input.states {
            let states_json = serde_json::to_value(states)
                .map_err(|e| CoreError::internal(format!("json error: {e}")))?;
            active.states = Set(states_json);
        }
        if let Some(transitions) = input.transitions {
            let transitions_json = serde_json::to_value(transitions)
                .map_err(|e| CoreError::internal(format!("json error: {e}")))?;
            active.transitions = Set(transitions_json);
        }

        active.updated_at = Set(chrono::Utc::now().fixed_offset());

        let result = active.update(db).await?;
        Ok(result)
    }

    pub async fn delete(db: &impl ConnectionTrait, id: Uuid) -> Result<(), CoreError> {
        let result = lifecycle_model::Entity::delete_by_id(id).exec(db).await?;
        if result.rows_affected == 0 {
            return Err(CoreError::not_found(format!(
                "lifecycle model {id} not found"
            )));
        }
        Ok(())
    }

    /// Validate that a state transition is allowed by the lifecycle model.
    pub async fn validate_transition(
        db: &impl ConnectionTrait,
        lifecycle_model_id: Uuid,
        from_state: &str,
        to_state: &str,
    ) -> Result<(), CoreError> {
        let model = Self::get(db, lifecycle_model_id).await?;

        let transitions: Vec<LifecycleTransition> =
            serde_json::from_value(model.transitions.clone()).unwrap_or_default();

        let allowed = transitions
            .iter()
            .any(|t| t.from == from_state && t.to == to_state);

        if !allowed {
            return Err(CoreError::bad_request(format!(
                "lifecycle transition from '{from_state}' to '{to_state}' is not allowed"
            )));
        }

        Ok(())
    }
}
