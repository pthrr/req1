#![allow(unused_qualifications)]

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use sea_orm::TransactionTrait;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{error::AppError, state::AppState};
use req1_core::{
    PaginatedResponse, Pagination,
    service::baseline::{
        BaselineDiff, BaselineService, BaselineWithEntries, CreateBaselineInput, DiffBaselineInput,
    },
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/modules/{module_id}/baselines",
            get(list_baselines).post(create_baseline),
        )
        .route(
            "/modules/{module_id}/baselines/{id}",
            get(get_baseline).delete(delete_baseline),
        )
        .route("/modules/{module_id}/baseline-diff", get(diff_baselines))
        .route("/baseline-diff", get(diff_baselines_global))
        .route("/baselines", get(list_all_baselines))
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct CreateBaselineRequest {
    name: String,
    description: Option<String>,
    baseline_set_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub(crate) struct DiffQuery {
    a: Uuid,
    b: Uuid,
}

#[utoipa::path(get, path = "/api/v1/modules/{module_id}/baselines", tag = "Baselines",
    security(("bearer_auth" = [])),
    params(
        ("module_id" = Uuid, Path, description = "Module ID"),
        Pagination,
    ),
    responses((status = 200, body = PaginatedResponse<entity::baseline::Model>))
)]
pub(crate) async fn list_baselines(
    State(state): State<AppState>,
    Path(module_id): Path<Uuid>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<PaginatedResponse<entity::baseline::Model>>, AppError> {
    let result =
        BaselineService::list(&state.db, module_id, pagination.offset, pagination.limit).await?;
    Ok(Json(result))
}

#[utoipa::path(post, path = "/api/v1/modules/{module_id}/baselines", tag = "Baselines",
    security(("bearer_auth" = [])),
    params(("module_id" = Uuid, Path, description = "Module ID")),
    request_body = CreateBaselineRequest,
    responses((status = 201, body = BaselineWithEntries))
)]
pub(crate) async fn create_baseline(
    State(state): State<AppState>,
    Path(module_id): Path<Uuid>,
    Json(body): Json<CreateBaselineRequest>,
) -> Result<(axum::http::StatusCode, Json<BaselineWithEntries>), AppError> {
    let txn = state.db.begin().await?;
    let result = BaselineService::create(
        &txn,
        CreateBaselineInput {
            module_id,
            name: body.name,
            description: body.description,
            baseline_set_id: body.baseline_set_id,
        },
    )
    .await?;
    txn.commit().await?;

    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

#[utoipa::path(get, path = "/api/v1/modules/{module_id}/baselines/{id}", tag = "Baselines",
    security(("bearer_auth" = [])),
    params(
        ("module_id" = Uuid, Path, description = "Module ID"),
        ("id" = Uuid, Path, description = "Baseline ID"),
    ),
    responses((status = 200, body = BaselineWithEntries), (status = 404, description = "Not found"))
)]
pub(crate) async fn get_baseline(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Json<BaselineWithEntries>, AppError> {
    let result = BaselineService::get(&state.db, id).await?;
    Ok(Json(result))
}

#[utoipa::path(delete, path = "/api/v1/modules/{module_id}/baselines/{id}", tag = "Baselines",
    security(("bearer_auth" = [])),
    params(
        ("module_id" = Uuid, Path, description = "Module ID"),
        ("id" = Uuid, Path, description = "Baseline ID"),
    ),
    responses((status = 204, description = "Deleted"), (status = 404, description = "Not found"))
)]
pub(crate) async fn delete_baseline(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, AppError> {
    BaselineService::delete(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[utoipa::path(get, path = "/api/v1/modules/{module_id}/baseline-diff", tag = "Baselines",
    security(("bearer_auth" = [])),
    params(
        ("module_id" = Uuid, Path, description = "Module ID"),
        DiffQuery,
    ),
    responses((status = 200, body = BaselineDiff))
)]
pub(crate) async fn diff_baselines(
    State(state): State<AppState>,
    Path(_module_id): Path<Uuid>,
    Query(query): Query<DiffQuery>,
) -> Result<Json<BaselineDiff>, AppError> {
    let result = BaselineService::diff(
        &state.db,
        DiffBaselineInput {
            a: query.a,
            b: query.b,
        },
    )
    .await?;
    Ok(Json(result))
}

#[utoipa::path(get, path = "/api/v1/baseline-diff", tag = "Baselines",
    security(("bearer_auth" = [])),
    params(DiffQuery),
    responses((status = 200, body = BaselineDiff))
)]
pub(crate) async fn diff_baselines_global(
    State(state): State<AppState>,
    Query(query): Query<DiffQuery>,
) -> Result<Json<BaselineDiff>, AppError> {
    let result = BaselineService::diff(
        &state.db,
        DiffBaselineInput {
            a: query.a,
            b: query.b,
        },
    )
    .await?;
    Ok(Json(result))
}

#[utoipa::path(get, path = "/api/v1/baselines", tag = "Baselines",
    security(("bearer_auth" = [])),
    params(Pagination),
    responses((status = 200, body = PaginatedResponse<entity::baseline::Model>))
)]
pub(crate) async fn list_all_baselines(
    State(state): State<AppState>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<PaginatedResponse<entity::baseline::Model>>, AppError> {
    let result = BaselineService::list_all(&state.db, pagination.offset, pagination.limit).await?;
    Ok(Json(result))
}
