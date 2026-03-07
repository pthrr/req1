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

#[utoipa::path(get, path = "/api/v1/objects/{object_id}/comments", tag = "Comments",
    security(("bearer_auth" = [])),
    params(
        ("object_id" = Uuid, Path, description = "Object ID"),
        Pagination,
    ),
    responses((status = 200, body = PaginatedResponse<comment::Model>))
)]
pub(crate) async fn list_comments(
    State(state): State<AppState>,
    Path(object_id): Path<Uuid>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<PaginatedResponse<comment::Model>>, AppError> {
    let result =
        CommentService::list(&state.db, object_id, pagination.offset, pagination.limit).await?;
    Ok(Json(result))
}

#[utoipa::path(post, path = "/api/v1/objects/{object_id}/comments", tag = "Comments",
    security(("bearer_auth" = [])),
    params(("object_id" = Uuid, Path, description = "Object ID")),
    request_body = CreateCommentInput,
    responses((status = 201, body = comment::Model))
)]
pub(crate) async fn create_comment(
    State(state): State<AppState>,
    Path(object_id): Path<Uuid>,
    Json(body): Json<CreateCommentInput>,
) -> Result<(axum::http::StatusCode, Json<comment::Model>), AppError> {
    let input = CreateCommentInput { object_id, ..body };
    let result = CommentService::create(&state.db, input).await?;
    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

#[utoipa::path(get, path = "/api/v1/objects/{object_id}/comments/{id}", tag = "Comments",
    security(("bearer_auth" = [])),
    params(
        ("object_id" = Uuid, Path, description = "Object ID"),
        ("id" = Uuid, Path, description = "Comment ID"),
    ),
    responses((status = 200, body = comment::Model), (status = 404, description = "Not found"))
)]
pub(crate) async fn get_comment(
    State(state): State<AppState>,
    Path((_object_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Json<comment::Model>, AppError> {
    let result = CommentService::get(&state.db, id).await?;
    Ok(Json(result))
}

#[utoipa::path(patch, path = "/api/v1/objects/{object_id}/comments/{id}", tag = "Comments",
    security(("bearer_auth" = [])),
    params(
        ("object_id" = Uuid, Path, description = "Object ID"),
        ("id" = Uuid, Path, description = "Comment ID"),
    ),
    request_body = UpdateCommentInput,
    responses((status = 200, body = comment::Model), (status = 404, description = "Not found"))
)]
pub(crate) async fn update_comment(
    State(state): State<AppState>,
    Path((_object_id, id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateCommentInput>,
) -> Result<Json<comment::Model>, AppError> {
    let result = CommentService::update(&state.db, id, body).await?;
    Ok(Json(result))
}

#[utoipa::path(delete, path = "/api/v1/objects/{object_id}/comments/{id}", tag = "Comments",
    security(("bearer_auth" = [])),
    params(
        ("object_id" = Uuid, Path, description = "Object ID"),
        ("id" = Uuid, Path, description = "Comment ID"),
    ),
    responses((status = 204, description = "Deleted"), (status = 404, description = "Not found"))
)]
pub(crate) async fn delete_comment(
    State(state): State<AppState>,
    Path((_object_id, id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, AppError> {
    CommentService::delete(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
