use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use uuid::Uuid;

use crate::{error::AppError, state::AppState};
use entity::app_user;
use req1_core::{
    PaginatedResponse,
    service::app_user::{
        AppUserService, CreateAppUserInput, ListAppUsersFilter, UpdateAppUserInput,
    },
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/users", get(list_users).post(create_user))
        .route(
            "/users/{id}",
            get(get_user).patch(update_user).delete(delete_user),
        )
}

#[utoipa::path(get, path = "/api/v1/users", tag = "Users",
    security(("bearer_auth" = [])),
    params(ListAppUsersFilter),
    responses((status = 200, body = PaginatedResponse<app_user::Model>))
)]
pub(crate) async fn list_users(
    State(state): State<AppState>,
    Query(filter): Query<ListAppUsersFilter>,
) -> Result<Json<PaginatedResponse<app_user::Model>>, AppError> {
    let result = AppUserService::list(&state.db, filter).await?;
    Ok(Json(result))
}

#[utoipa::path(post, path = "/api/v1/users", tag = "Users",
    security(("bearer_auth" = [])),
    request_body = CreateAppUserInput,
    responses((status = 201, body = app_user::Model))
)]
pub(crate) async fn create_user(
    State(state): State<AppState>,
    Json(body): Json<CreateAppUserInput>,
) -> Result<(axum::http::StatusCode, Json<app_user::Model>), AppError> {
    let result = AppUserService::create(&state.db, body).await?;
    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

#[utoipa::path(get, path = "/api/v1/users/{id}", tag = "Users",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "User ID")),
    responses((status = 200, body = app_user::Model), (status = 404, description = "Not found"))
)]
pub(crate) async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<app_user::Model>, AppError> {
    let result = AppUserService::get(&state.db, id).await?;
    Ok(Json(result))
}

#[utoipa::path(patch, path = "/api/v1/users/{id}", tag = "Users",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "User ID")),
    request_body = UpdateAppUserInput,
    responses((status = 200, body = app_user::Model), (status = 404, description = "Not found"))
)]
pub(crate) async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateAppUserInput>,
) -> Result<Json<app_user::Model>, AppError> {
    let result = AppUserService::update(&state.db, id, body).await?;
    Ok(Json(result))
}

#[utoipa::path(delete, path = "/api/v1/users/{id}", tag = "Users",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "User ID")),
    responses((status = 204, description = "Deleted"), (status = 404, description = "Not found"))
)]
pub(crate) async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    AppUserService::delete(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
