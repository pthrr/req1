use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use uuid::Uuid;

use crate::{error::AppError, state::AppState};
use entity::workspace;
use req1_core::{
    PaginatedResponse, Pagination,
    service::workspace::{CreateWorkspaceInput, UpdateWorkspaceInput, WorkspaceService},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/workspaces", get(list_workspaces).post(create_workspace))
        .route(
            "/workspaces/{id}",
            get(get_workspace)
                .patch(update_workspace)
                .delete(delete_workspace),
        )
}

#[utoipa::path(get, path = "/api/v1/workspaces", tag = "Workspaces",
    security(("bearer_auth" = [])),
    params(Pagination),
    responses((status = 200, body = PaginatedResponse<workspace::Model>))
)]
pub(crate) async fn list_workspaces(
    State(state): State<AppState>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<PaginatedResponse<workspace::Model>>, AppError> {
    let result = WorkspaceService::list(&state.db, pagination.offset, pagination.limit).await?;
    Ok(Json(result))
}

#[utoipa::path(post, path = "/api/v1/workspaces", tag = "Workspaces",
    security(("bearer_auth" = [])),
    request_body = CreateWorkspaceInput,
    responses((status = 201, body = workspace::Model))
)]
pub(crate) async fn create_workspace(
    State(state): State<AppState>,
    Json(body): Json<CreateWorkspaceInput>,
) -> Result<(axum::http::StatusCode, Json<workspace::Model>), AppError> {
    let result = WorkspaceService::create(&state.db, body).await?;
    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

#[utoipa::path(get, path = "/api/v1/workspaces/{id}", tag = "Workspaces",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Workspace ID")),
    responses((status = 200, body = workspace::Model), (status = 404, description = "Not found"))
)]
pub(crate) async fn get_workspace(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<workspace::Model>, AppError> {
    let result = WorkspaceService::get(&state.db, id).await?;
    Ok(Json(result))
}

#[utoipa::path(patch, path = "/api/v1/workspaces/{id}", tag = "Workspaces",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Workspace ID")),
    request_body = UpdateWorkspaceInput,
    responses((status = 200, body = workspace::Model), (status = 404, description = "Not found"))
)]
pub(crate) async fn update_workspace(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateWorkspaceInput>,
) -> Result<Json<workspace::Model>, AppError> {
    let result = WorkspaceService::update(&state.db, id, body).await?;
    Ok(Json(result))
}

#[utoipa::path(delete, path = "/api/v1/workspaces/{id}", tag = "Workspaces",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Workspace ID")),
    responses((status = 204, description = "Deleted"), (status = 404, description = "Not found"))
)]
pub(crate) async fn delete_workspace(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    WorkspaceService::delete(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
