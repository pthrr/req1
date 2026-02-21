use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter, Set,
};
use serde::Deserialize;
use uuid::Uuid;

use entity::object_type;

use crate::PaginatedResponse;
use crate::error::CoreError;

#[derive(Debug, Deserialize)]
pub struct CreateObjectTypeInput {
    pub module_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub default_classification: Option<String>,
    pub required_attributes: Option<serde_json::Value>,
    pub attribute_schema: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateObjectTypeInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub default_classification: Option<String>,
    pub required_attributes: Option<serde_json::Value>,
    pub attribute_schema: Option<serde_json::Value>,
}

pub struct ObjectTypeService;

impl ObjectTypeService {
    pub async fn create(
        db: &impl ConnectionTrait,
        input: CreateObjectTypeInput,
    ) -> Result<object_type::Model, CoreError> {
        let now = chrono::Utc::now().fixed_offset();
        let id = Uuid::now_v7();

        let model = object_type::ActiveModel {
            id: Set(id),
            module_id: Set(input.module_id),
            name: Set(input.name),
            description: Set(input.description),
            default_classification: Set(input
                .default_classification
                .unwrap_or_else(|| "normative".to_owned())),
            required_attributes: Set(input.required_attributes.unwrap_or(serde_json::json!([]))),
            attribute_schema: Set(input.attribute_schema.unwrap_or(serde_json::json!({}))),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let result = model.insert(db).await?;
        Ok(result)
    }

    pub async fn update(
        db: &impl ConnectionTrait,
        id: Uuid,
        input: UpdateObjectTypeInput,
    ) -> Result<object_type::Model, CoreError> {
        let existing = object_type::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("object_type {id} not found")))?;

        let mut active: object_type::ActiveModel = existing.into();
        if let Some(name) = input.name {
            active.name = Set(name);
        }
        if let Some(description) = input.description {
            active.description = Set(Some(description));
        }
        if let Some(default_classification) = input.default_classification {
            active.default_classification = Set(default_classification);
        }
        if let Some(required_attributes) = input.required_attributes {
            active.required_attributes = Set(required_attributes);
        }
        if let Some(attribute_schema) = input.attribute_schema {
            active.attribute_schema = Set(attribute_schema);
        }
        active.updated_at = Set(chrono::Utc::now().fixed_offset());

        let result = active.update(db).await?;
        Ok(result)
    }

    pub async fn delete(db: &impl ConnectionTrait, id: Uuid) -> Result<(), CoreError> {
        let result = object_type::Entity::delete_by_id(id).exec(db).await?;
        if result.rows_affected == 0 {
            return Err(CoreError::NotFound(format!("object_type {id} not found")));
        }
        Ok(())
    }

    pub async fn get(db: &impl ConnectionTrait, id: Uuid) -> Result<object_type::Model, CoreError> {
        object_type::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("object_type {id} not found")))
    }

    pub async fn list(
        db: &impl ConnectionTrait,
        module_id: Uuid,
        offset: u64,
        limit: u64,
    ) -> Result<PaginatedResponse<object_type::Model>, CoreError> {
        let paginator = object_type::Entity::find()
            .filter(object_type::Column::ModuleId.eq(module_id))
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
