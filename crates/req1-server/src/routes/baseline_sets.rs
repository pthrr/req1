use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use uuid::Uuid;

use crate::{error::AppError, state::AppState};
use entity::baseline_set;
use req1_core::{
    PaginatedResponse, Pagination,
    service::baseline_set::{BaselineSetService, CreateBaselineSetInput, UpdateBaselineSetInput},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/baseline-sets",
            get(list_baseline_sets).post(create_baseline_set),
        )
        .route(
            "/baseline-sets/{id}",
            get(get_baseline_set)
                .patch(update_baseline_set)
                .delete(delete_baseline_set),
        )
}

async fn list_baseline_sets(
    State(state): State<AppState>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<PaginatedResponse<baseline_set::Model>>, AppError> {
    let result = BaselineSetService::list(&state.db, pagination.offset, pagination.limit).await?;
    Ok(Json(result))
}

async fn create_baseline_set(
    State(state): State<AppState>,
    Json(body): Json<CreateBaselineSetInput>,
) -> Result<(axum::http::StatusCode, Json<baseline_set::Model>), AppError> {
    let result = BaselineSetService::create(&state.db, body).await?;
    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

async fn get_baseline_set(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<baseline_set::Model>, AppError> {
    let result = BaselineSetService::get(&state.db, id).await?;
    Ok(Json(result))
}

async fn update_baseline_set(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateBaselineSetInput>,
) -> Result<Json<baseline_set::Model>, AppError> {
    let result = BaselineSetService::update(&state.db, id, body).await?;
    Ok(Json(result))
}

async fn delete_baseline_set(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    BaselineSetService::delete(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
