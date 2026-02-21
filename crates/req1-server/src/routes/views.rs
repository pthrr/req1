use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use uuid::Uuid;

use crate::{error::AppError, state::AppState};
use entity::view;
use req1_core::{
    PaginatedResponse, Pagination,
    service::view::{CreateViewInput, UpdateViewInput, ViewService},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/modules/{module_id}/views",
            get(list_views).post(create_view),
        )
        .route(
            "/modules/{module_id}/views/{id}",
            get(get_view).patch(update_view).delete(delete_view),
        )
}

async fn list_views(
    State(state): State<AppState>,
    Path(module_id): Path<Uuid>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<PaginatedResponse<view::Model>>, AppError> {
    let result =
        ViewService::list(&state.db, module_id, pagination.offset, pagination.limit).await?;
    Ok(Json(result))
}

async fn create_view(
    State(state): State<AppState>,
    Path(module_id): Path<Uuid>,
    Json(body): Json<CreateViewInput>,
) -> Result<(axum::http::StatusCode, Json<view::Model>), AppError> {
    let input = CreateViewInput { module_id, ..body };
    let result = ViewService::create(&state.db, input).await?;
    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

async fn get_view(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Json<view::Model>, AppError> {
    let result = ViewService::get(&state.db, id).await?;
    Ok(Json(result))
}

async fn update_view(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateViewInput>,
) -> Result<Json<view::Model>, AppError> {
    let result = ViewService::update(&state.db, id, body).await?;
    Ok(Json(result))
}

async fn delete_view(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, AppError> {
    ViewService::delete(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
