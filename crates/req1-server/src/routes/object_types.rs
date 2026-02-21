use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use uuid::Uuid;

use crate::{error::AppError, state::AppState};
use entity::object_type;
use req1_core::{
    PaginatedResponse,
    service::object_type::{CreateObjectTypeInput, ObjectTypeService, UpdateObjectTypeInput},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/object-types",
            get(list_object_types).post(create_object_type),
        )
        .route(
            "/object-types/{id}",
            get(get_object_type)
                .patch(update_object_type)
                .delete(delete_object_type),
        )
}

#[derive(Debug, serde::Deserialize)]
struct ListObjectTypesQuery {
    module_id: Uuid,
    #[serde(default)]
    offset: u64,
    #[serde(default = "default_limit")]
    limit: u64,
}

const fn default_limit() -> u64 {
    50
}

async fn list_object_types(
    State(state): State<AppState>,
    Query(query): Query<ListObjectTypesQuery>,
) -> Result<Json<PaginatedResponse<object_type::Model>>, AppError> {
    let result =
        ObjectTypeService::list(&state.db, query.module_id, query.offset, query.limit).await?;
    Ok(Json(result))
}

async fn create_object_type(
    State(state): State<AppState>,
    Json(body): Json<CreateObjectTypeInput>,
) -> Result<(axum::http::StatusCode, Json<object_type::Model>), AppError> {
    let result = ObjectTypeService::create(&state.db, body).await?;
    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

async fn get_object_type(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<object_type::Model>, AppError> {
    let result = ObjectTypeService::get(&state.db, id).await?;
    Ok(Json(result))
}

async fn update_object_type(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateObjectTypeInput>,
) -> Result<Json<object_type::Model>, AppError> {
    let result = ObjectTypeService::update(&state.db, id, body).await?;
    Ok(Json(result))
}

async fn delete_object_type(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    ObjectTypeService::delete(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
