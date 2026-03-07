use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, Order, PaginatorTrait,
    QueryFilter, QueryOrder, Set,
};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use entity::diagram;

use crate::PaginatedResponse;
use crate::error::CoreError;

const VALID_DIAGRAM_TYPES: &[&str] = &["use_case", "sequence", "class", "flowchart", "state", "er"];

const fn default_limit() -> u64 {
    50
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateDiagramInput {
    #[serde(default)]
    pub module_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub diagram_type: Option<String>,
    pub source_code: Option<String>,
    pub linked_object_ids: Option<Vec<Uuid>>,
    pub created_by: Option<Uuid>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateDiagramInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub diagram_type: Option<String>,
    pub source_code: Option<String>,
    pub linked_object_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListDiagramsFilter {
    #[serde(default)]
    pub offset: u64,
    #[serde(default = "default_limit")]
    pub limit: u64,
}

pub struct DiagramService;

impl DiagramService {
    pub async fn create(
        db: &impl ConnectionTrait,
        input: CreateDiagramInput,
    ) -> Result<diagram::Model, CoreError> {
        let dtype = input.diagram_type.as_deref().unwrap_or("use_case");
        if !VALID_DIAGRAM_TYPES.contains(&dtype) {
            return Err(CoreError::bad_request(format!(
                "invalid diagram_type '{dtype}', must be one of: {VALID_DIAGRAM_TYPES:?}"
            )));
        }

        let now = chrono::Utc::now().fixed_offset();
        let id = Uuid::now_v7();

        let linked_ids = serde_json::to_value(input.linked_object_ids.unwrap_or_default())
            .unwrap_or(serde_json::json!([]));

        let model = diagram::ActiveModel {
            id: Set(id),
            module_id: Set(input.module_id),
            name: Set(input.name),
            description: Set(input.description),
            diagram_type: Set(dtype.to_string()),
            source_code: Set(input.source_code.unwrap_or_default()),
            linked_object_ids: Set(linked_ids),
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
        input: UpdateDiagramInput,
    ) -> Result<diagram::Model, CoreError> {
        let existing = diagram::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::not_found(format!("diagram {id} not found")))?;

        if let Some(ref dtype) = input.diagram_type
            && !VALID_DIAGRAM_TYPES.contains(&dtype.as_str())
        {
            return Err(CoreError::bad_request(format!(
                "invalid diagram_type '{dtype}', must be one of: {VALID_DIAGRAM_TYPES:?}"
            )));
        }

        let mut active: diagram::ActiveModel = existing.into();
        if let Some(name) = input.name {
            active.name = Set(name);
        }
        if let Some(description) = input.description {
            active.description = Set(Some(description));
        }
        if let Some(diagram_type) = input.diagram_type {
            active.diagram_type = Set(diagram_type);
        }
        if let Some(source_code) = input.source_code {
            active.source_code = Set(source_code);
        }
        if let Some(linked_ids) = input.linked_object_ids {
            active.linked_object_ids =
                Set(serde_json::to_value(linked_ids).unwrap_or(serde_json::json!([])));
        }
        active.updated_at = Set(chrono::Utc::now().fixed_offset());

        let result = active.update(db).await?;
        Ok(result)
    }

    pub async fn delete(db: &impl ConnectionTrait, id: Uuid) -> Result<(), CoreError> {
        let result = diagram::Entity::delete_by_id(id).exec(db).await?;
        if result.rows_affected == 0 {
            return Err(CoreError::not_found(format!("diagram {id} not found")));
        }
        Ok(())
    }

    pub async fn get(db: &impl ConnectionTrait, id: Uuid) -> Result<diagram::Model, CoreError> {
        diagram::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::not_found(format!("diagram {id} not found")))
    }

    pub async fn list(
        db: &impl ConnectionTrait,
        module_id: Uuid,
        filter: ListDiagramsFilter,
    ) -> Result<PaginatedResponse<diagram::Model>, CoreError> {
        let select = diagram::Entity::find()
            .filter(diagram::Column::ModuleId.eq(module_id))
            .order_by(diagram::Column::CreatedAt, Order::Desc);

        let paginator = select.paginate(db, filter.limit);
        let total = paginator.num_items().await?;
        let page = filter.offset.checked_div(filter.limit).unwrap_or(0);
        let items = paginator.fetch_page(page).await?;

        Ok(PaginatedResponse {
            items,
            total,
            offset: filter.offset,
            limit: filter.limit,
        })
    }
}
