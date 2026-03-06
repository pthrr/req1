use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    routing::{get, post},
};
use serde::Serialize;
use uuid::Uuid;

use crate::{error::AppError, state::AppState};
use entity::notification;
use req1_core::auth::AuthUser;
use req1_core::{
    PaginatedResponse,
    service::notification::{ListNotificationsFilter, NotificationService},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/notifications", get(list_notifications))
        .route("/notifications/unread-count", get(unread_count))
        .route("/notifications/{id}/read", post(mark_read))
        .route("/notifications/read-all", post(mark_all_read))
}

async fn list_notifications(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(filter): Query<ListNotificationsFilter>,
) -> Result<Json<PaginatedResponse<notification::Model>>, AppError> {
    let result = NotificationService::list(&state.db, auth_user.id, filter).await?;
    Ok(Json(result))
}

#[derive(Serialize)]
struct UnreadCountResponse {
    count: u64,
}

async fn unread_count(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<UnreadCountResponse>, AppError> {
    let count = NotificationService::unread_count(&state.db, auth_user.id).await?;
    Ok(Json(UnreadCountResponse { count }))
}

async fn mark_read(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<notification::Model>, AppError> {
    let result = NotificationService::mark_read(&state.db, id, auth_user.id).await?;
    Ok(Json(result))
}

#[derive(Serialize)]
struct MarkAllReadResponse {
    updated: u64,
}

async fn mark_all_read(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<MarkAllReadResponse>, AppError> {
    let updated = NotificationService::mark_all_read(&state.db, auth_user.id).await?;
    Ok(Json(MarkAllReadResponse { updated }))
}
