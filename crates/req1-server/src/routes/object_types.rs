#![allow(unused_qualifications)]

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use utoipa::IntoParams;
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

#[derive(Debug, serde::Deserialize, IntoParams)]
pub(crate) struct ListObjectTypesQuery {
    module_id: Uuid,
    #[serde(default)]
    offset: u64,
    #[serde(default = "default_limit")]
    limit: u64,
}

const fn default_limit() -> u64 {
    50
}

#[utoipa::path(get, path = "/api/v1/object-types", tag = "ObjectTypes",
    security(("bearer_auth" = [])),
    params(ListObjectTypesQuery),
    responses((status = 200, body = PaginatedResponse<object_type::Model>))
)]
pub(crate) async fn list_object_types(
    State(state): State<AppState>,
    Query(query): Query<ListObjectTypesQuery>,
) -> Result<Json<PaginatedResponse<object_type::Model>>, AppError> {
    let result =
        ObjectTypeService::list(&state.db, query.module_id, query.offset, query.limit).await?;
    Ok(Json(result))
}

#[utoipa::path(post, path = "/api/v1/object-types", tag = "ObjectTypes",
    security(("bearer_auth" = [])),
    request_body = CreateObjectTypeInput,
    responses((status = 201, body = object_type::Model))
)]
pub(crate) async fn create_object_type(
    State(state): State<AppState>,
    Json(body): Json<CreateObjectTypeInput>,
) -> Result<(axum::http::StatusCode, Json<object_type::Model>), AppError> {
    let result = ObjectTypeService::create(&state.db, body).await?;
    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

#[utoipa::path(get, path = "/api/v1/object-types/{id}", tag = "ObjectTypes",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Object type ID")),
    responses((status = 200, body = object_type::Model), (status = 404, description = "Not found"))
)]
pub(crate) async fn get_object_type(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<object_type::Model>, AppError> {
    let result = ObjectTypeService::get(&state.db, id).await?;
    Ok(Json(result))
}

#[utoipa::path(patch, path = "/api/v1/object-types/{id}", tag = "ObjectTypes",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Object type ID")),
    request_body = UpdateObjectTypeInput,
    responses((status = 200, body = object_type::Model), (status = 404, description = "Not found"))
)]
pub(crate) async fn update_object_type(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateObjectTypeInput>,
) -> Result<Json<object_type::Model>, AppError> {
    let result = ObjectTypeService::update(&state.db, id, body).await?;
    Ok(Json(result))
}

#[utoipa::path(delete, path = "/api/v1/object-types/{id}", tag = "ObjectTypes",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Object type ID")),
    responses((status = 204, description = "Deleted"), (status = 404, description = "Not found"))
)]
pub(crate) async fn delete_object_type(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    ObjectTypeService::delete(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
