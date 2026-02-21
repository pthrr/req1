use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};
use serde::Deserialize;
use uuid::Uuid;

use crate::{error::AppError, state::AppState};
use entity::attribute_definition;
use req1_core::{PaginatedResponse, Pagination};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/modules/{module_id}/attribute-definitions",
            get(list_attribute_definitions).post(create_attribute_definition),
        )
        .route(
            "/modules/{module_id}/attribute-definitions/{id}",
            get(get_attribute_definition)
                .patch(update_attribute_definition)
                .delete(delete_attribute_definition),
        )
}

#[derive(Debug, Deserialize)]
struct CreateAttributeDefinitionRequest {
    name: String,
    data_type: String,
    default_value: Option<String>,
    enum_values: Option<serde_json::Value>,
    multi_select: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct UpdateAttributeDefinitionRequest {
    name: Option<String>,
    data_type: Option<String>,
    default_value: Option<String>,
    enum_values: Option<serde_json::Value>,
    multi_select: Option<bool>,
}

async fn list_attribute_definitions(
    State(state): State<AppState>,
    Path(module_id): Path<Uuid>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<PaginatedResponse<attribute_definition::Model>>, AppError> {
    let paginator = attribute_definition::Entity::find()
        .filter(
            attribute_definition::Column::ModuleId
                .eq(module_id)
                .or(attribute_definition::Column::ModuleId.is_null()),
        )
        .paginate(&state.db, pagination.limit);
    let total = paginator.num_items().await?;
    let page = pagination.offset / pagination.limit;
    let items = paginator.fetch_page(page).await?;

    Ok(Json(PaginatedResponse {
        items,
        total,
        offset: pagination.offset,
        limit: pagination.limit,
    }))
}

async fn create_attribute_definition(
    State(state): State<AppState>,
    Path(module_id): Path<Uuid>,
    Json(body): Json<CreateAttributeDefinitionRequest>,
) -> Result<(axum::http::StatusCode, Json<attribute_definition::Model>), AppError> {
    let now = chrono::Utc::now().fixed_offset();
    let id = Uuid::now_v7();

    let valid_types = [
        "string",
        "integer",
        "float",
        "date",
        "bool",
        "enum",
        "rich_text",
        "user_ref",
    ];
    if !valid_types.contains(&body.data_type.as_str()) {
        return Err(AppError::BadRequest(format!(
            "invalid data_type '{}', must be one of: {}",
            body.data_type,
            valid_types.join(", ")
        )));
    }

    req1_core::validation::validate_attr_constraints(
        &body.data_type,
        &body.default_value,
        &body.enum_values,
    )?;

    let model = attribute_definition::ActiveModel {
        id: Set(id),
        module_id: Set(Some(module_id)),
        name: Set(body.name),
        data_type: Set(body.data_type),
        default_value: Set(body.default_value),
        enum_values: Set(
            if body
                .enum_values
                .as_ref()
                .is_some_and(serde_json::Value::is_array)
            {
                body.enum_values
            } else {
                None
            },
        ),
        multi_select: Set(body.multi_select.unwrap_or(false)),
        created_at: Set(now),
    };

    let result = model.insert(&state.db).await?;
    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

async fn get_attribute_definition(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Json<attribute_definition::Model>, AppError> {
    let def = attribute_definition::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("attribute definition {id} not found")))?;

    Ok(Json(def))
}

async fn update_attribute_definition(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateAttributeDefinitionRequest>,
) -> Result<Json<attribute_definition::Model>, AppError> {
    let existing = attribute_definition::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("attribute definition {id} not found")))?;

    let mut active: attribute_definition::ActiveModel = existing.into();
    if let Some(name) = body.name {
        active.name = Set(name);
    }
    if let Some(data_type) = body.data_type {
        let valid_types = [
            "string",
            "integer",
            "float",
            "date",
            "bool",
            "enum",
            "rich_text",
            "user_ref",
        ];
        if !valid_types.contains(&data_type.as_str()) {
            return Err(AppError::BadRequest(format!(
                "invalid data_type '{data_type}'"
            )));
        }
        req1_core::validation::validate_attr_constraints(
            &data_type,
            &body.default_value,
            &body.enum_values,
        )?;
        active.data_type = Set(data_type);
    }
    if let Some(default_value) = body.default_value {
        active.default_value = Set(Some(default_value));
    }
    if let Some(enum_values) = body.enum_values.filter(serde_json::Value::is_array) {
        active.enum_values = Set(Some(enum_values));
    }
    if let Some(multi_select) = body.multi_select {
        active.multi_select = Set(multi_select);
    }

    let result = active.update(&state.db).await?;
    Ok(Json(result))
}

async fn delete_attribute_definition(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, AppError> {
    let result = attribute_definition::Entity::delete_by_id(id)
        .exec(&state.db)
        .await?;
    if result.rows_affected == 0 {
        return Err(AppError::NotFound(format!(
            "attribute definition {id} not found"
        )));
    }
    Ok(axum::http::StatusCode::NO_CONTENT)
}
