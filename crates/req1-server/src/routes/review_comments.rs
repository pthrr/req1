use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use uuid::Uuid;

use crate::{error::AppError, state::AppState};
use entity::review_comment;
use req1_core::{
    PaginatedResponse, Pagination,
    service::review_comment::{
        CreateReviewCommentInput, ReviewCommentService, UpdateReviewCommentInput,
    },
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/review-packages/{package_id}/comments",
            get(list_review_comments).post(create_review_comment),
        )
        .route(
            "/review-packages/{package_id}/comments/{id}",
            get(get_review_comment)
                .patch(update_review_comment)
                .delete(delete_review_comment),
        )
}

async fn list_review_comments(
    State(state): State<AppState>,
    Path(package_id): Path<Uuid>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<PaginatedResponse<review_comment::Model>>, AppError> {
    let result =
        ReviewCommentService::list(&state.db, package_id, pagination.offset, pagination.limit)
            .await?;
    Ok(Json(result))
}

async fn create_review_comment(
    State(state): State<AppState>,
    Path(package_id): Path<Uuid>,
    Json(body): Json<CreateReviewCommentInput>,
) -> Result<(axum::http::StatusCode, Json<review_comment::Model>), AppError> {
    let input = CreateReviewCommentInput {
        package_id,
        ..body
    };
    let result = ReviewCommentService::create(&state.db, input).await?;
    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

async fn get_review_comment(
    State(state): State<AppState>,
    Path((_package_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Json<review_comment::Model>, AppError> {
    let result = ReviewCommentService::get(&state.db, id).await?;
    Ok(Json(result))
}

async fn update_review_comment(
    State(state): State<AppState>,
    Path((_package_id, id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateReviewCommentInput>,
) -> Result<Json<review_comment::Model>, AppError> {
    let result = ReviewCommentService::update(&state.db, id, body).await?;
    Ok(Json(result))
}

async fn delete_review_comment(
    State(state): State<AppState>,
    Path((_package_id, id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, AppError> {
    ReviewCommentService::delete(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
