use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use uuid::Uuid;

use crate::{error::AppError, state::AppState};
use entity::comment;
use req1_core::{
    PaginatedResponse, Pagination,
    service::comment::{CommentService, CreateCommentInput, UpdateCommentInput},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/objects/{object_id}/comments",
            get(list_comments).post(create_comment),
        )
        .route(
            "/objects/{object_id}/comments/{id}",
            get(get_comment)
                .patch(update_comment)
                .delete(delete_comment),
        )
}

async fn list_comments(
    State(state): State<AppState>,
    Path(object_id): Path<Uuid>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<PaginatedResponse<comment::Model>>, AppError> {
    let result =
        CommentService::list(&state.db, object_id, pagination.offset, pagination.limit).await?;
    Ok(Json(result))
}

async fn create_comment(
    State(state): State<AppState>,
    Path(object_id): Path<Uuid>,
    Json(body): Json<CreateCommentInput>,
) -> Result<(axum::http::StatusCode, Json<comment::Model>), AppError> {
    let input = CreateCommentInput { object_id, ..body };
    let result = CommentService::create(&state.db, input).await?;
    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

async fn get_comment(
    State(state): State<AppState>,
    Path((_object_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Json<comment::Model>, AppError> {
    let result = CommentService::get(&state.db, id).await?;
    Ok(Json(result))
}

async fn update_comment(
    State(state): State<AppState>,
    Path((_object_id, id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateCommentInput>,
) -> Result<Json<comment::Model>, AppError> {
    let result = CommentService::update(&state.db, id, body).await?;
    Ok(Json(result))
}

async fn delete_comment(
    State(state): State<AppState>,
    Path((_object_id, id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, AppError> {
    CommentService::delete(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
