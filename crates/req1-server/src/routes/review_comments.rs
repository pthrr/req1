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

#[utoipa::path(get, path = "/api/v1/review-packages/{package_id}/comments", tag = "ReviewComments",
    security(("bearer_auth" = [])),
    params(
        ("package_id" = Uuid, Path, description = "Review package ID"),
        Pagination,
    ),
    responses((status = 200, body = PaginatedResponse<review_comment::Model>))
)]
pub(crate) async fn list_review_comments(
    State(state): State<AppState>,
    Path(package_id): Path<Uuid>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<PaginatedResponse<review_comment::Model>>, AppError> {
    let result =
        ReviewCommentService::list(&state.db, package_id, pagination.offset, pagination.limit)
            .await?;
    Ok(Json(result))
}

#[utoipa::path(post, path = "/api/v1/review-packages/{package_id}/comments", tag = "ReviewComments",
    security(("bearer_auth" = [])),
    params(("package_id" = Uuid, Path, description = "Review package ID")),
    request_body = CreateReviewCommentInput,
    responses((status = 201, body = review_comment::Model))
)]
pub(crate) async fn create_review_comment(
    State(state): State<AppState>,
    Path(package_id): Path<Uuid>,
    Json(body): Json<CreateReviewCommentInput>,
) -> Result<(axum::http::StatusCode, Json<review_comment::Model>), AppError> {
    let input = CreateReviewCommentInput { package_id, ..body };
    let result = ReviewCommentService::create(&state.db, input).await?;
    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

#[utoipa::path(get, path = "/api/v1/review-packages/{package_id}/comments/{id}", tag = "ReviewComments",
    security(("bearer_auth" = [])),
    params(
        ("package_id" = Uuid, Path, description = "Review package ID"),
        ("id" = Uuid, Path, description = "Comment ID"),
    ),
    responses((status = 200, body = review_comment::Model), (status = 404, description = "Not found"))
)]
pub(crate) async fn get_review_comment(
    State(state): State<AppState>,
    Path((_package_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Json<review_comment::Model>, AppError> {
    let result = ReviewCommentService::get(&state.db, id).await?;
    Ok(Json(result))
}

#[utoipa::path(patch, path = "/api/v1/review-packages/{package_id}/comments/{id}", tag = "ReviewComments",
    security(("bearer_auth" = [])),
    params(
        ("package_id" = Uuid, Path, description = "Review package ID"),
        ("id" = Uuid, Path, description = "Comment ID"),
    ),
    request_body = UpdateReviewCommentInput,
    responses((status = 200, body = review_comment::Model), (status = 404, description = "Not found"))
)]
pub(crate) async fn update_review_comment(
    State(state): State<AppState>,
    Path((_package_id, id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateReviewCommentInput>,
) -> Result<Json<review_comment::Model>, AppError> {
    let result = ReviewCommentService::update(&state.db, id, body).await?;
    Ok(Json(result))
}

#[utoipa::path(delete, path = "/api/v1/review-packages/{package_id}/comments/{id}", tag = "ReviewComments",
    security(("bearer_auth" = [])),
    params(
        ("package_id" = Uuid, Path, description = "Review package ID"),
        ("id" = Uuid, Path, description = "Comment ID"),
    ),
    responses((status = 204, description = "Deleted"), (status = 404, description = "Not found"))
)]
pub(crate) async fn delete_review_comment(
    State(state): State<AppState>,
    Path((_package_id, id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, AppError> {
    ReviewCommentService::delete(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
