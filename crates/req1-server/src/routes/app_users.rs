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

async fn list_users(
    State(state): State<AppState>,
    Query(filter): Query<ListAppUsersFilter>,
) -> Result<Json<PaginatedResponse<app_user::Model>>, AppError> {
    let result = AppUserService::list(&state.db, filter).await?;
    Ok(Json(result))
}

async fn create_user(
    State(state): State<AppState>,
    Json(body): Json<CreateAppUserInput>,
) -> Result<(axum::http::StatusCode, Json<app_user::Model>), AppError> {
    let result = AppUserService::create(&state.db, body).await?;
    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<app_user::Model>, AppError> {
    let result = AppUserService::get(&state.db, id).await?;
    Ok(Json(result))
}

async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateAppUserInput>,
) -> Result<Json<app_user::Model>, AppError> {
    let result = AppUserService::update(&state.db, id, body).await?;
    Ok(Json(result))
}

async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    AppUserService::delete(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
