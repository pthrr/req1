use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use uuid::Uuid;

use crate::{error::AppError, state::AppState};
use entity::project;
use req1_core::{
    PaginatedResponse, Pagination,
    service::project::{CreateProjectInput, ProjectService, UpdateProjectInput},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/workspaces/{workspace_id}/projects",
            get(list_projects).post(create_project),
        )
        .route(
            "/workspaces/{workspace_id}/projects/{id}",
            get(get_project)
                .patch(update_project)
                .delete(delete_project),
        )
}

async fn list_projects(
    State(state): State<AppState>,
    Path(workspace_id): Path<Uuid>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<PaginatedResponse<project::Model>>, AppError> {
    let result =
        ProjectService::list(&state.db, workspace_id, pagination.offset, pagination.limit).await?;
    Ok(Json(result))
}

async fn create_project(
    State(state): State<AppState>,
    Path(workspace_id): Path<Uuid>,
    Json(body): Json<CreateProjectInput>,
) -> Result<(axum::http::StatusCode, Json<project::Model>), AppError> {
    let input = CreateProjectInput {
        workspace_id,
        ..body
    };
    let result = ProjectService::create(&state.db, input).await?;
    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

async fn get_project(
    State(state): State<AppState>,
    Path((_workspace_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Json<project::Model>, AppError> {
    let result = ProjectService::get(&state.db, id).await?;
    Ok(Json(result))
}

async fn update_project(
    State(state): State<AppState>,
    Path((_workspace_id, id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateProjectInput>,
) -> Result<Json<project::Model>, AppError> {
    let result = ProjectService::update(&state.db, id, body).await?;
    Ok(Json(result))
}

async fn delete_project(
    State(state): State<AppState>,
    Path((_workspace_id, id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, AppError> {
    ProjectService::delete(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
