use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use uuid::Uuid;

use crate::{error::AppError, state::AppState};
use entity::review_assignment;
use req1_core::{
    PaginatedResponse, Pagination,
    service::review_assignment::{
        CreateReviewAssignmentInput, ReviewAssignmentService, UpdateReviewAssignmentInput,
    },
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/review-packages/{package_id}/assignments",
            get(list_assignments).post(create_assignment),
        )
        .route(
            "/review-packages/{package_id}/assignments/{id}",
            get(get_assignment)
                .patch(update_assignment)
                .delete(delete_assignment),
        )
}

#[utoipa::path(get, path = "/api/v1/review-packages/{package_id}/assignments", tag = "ReviewAssignments",
    security(("bearer_auth" = [])),
    params(
        ("package_id" = Uuid, Path, description = "Review package ID"),
        Pagination,
    ),
    responses((status = 200, body = PaginatedResponse<review_assignment::Model>))
)]
pub(crate) async fn list_assignments(
    State(state): State<AppState>,
    Path(package_id): Path<Uuid>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<PaginatedResponse<review_assignment::Model>>, AppError> {
    let result =
        ReviewAssignmentService::list(&state.db, package_id, pagination.offset, pagination.limit)
            .await?;
    Ok(Json(result))
}

#[utoipa::path(post, path = "/api/v1/review-packages/{package_id}/assignments", tag = "ReviewAssignments",
    security(("bearer_auth" = [])),
    params(("package_id" = Uuid, Path, description = "Review package ID")),
    request_body = CreateReviewAssignmentInput,
    responses((status = 201, body = review_assignment::Model))
)]
pub(crate) async fn create_assignment(
    State(state): State<AppState>,
    Path(package_id): Path<Uuid>,
    Json(body): Json<CreateReviewAssignmentInput>,
) -> Result<(axum::http::StatusCode, Json<review_assignment::Model>), AppError> {
    let input = CreateReviewAssignmentInput { package_id, ..body };
    let result = ReviewAssignmentService::create(&state.db, input).await?;
    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

#[utoipa::path(get, path = "/api/v1/review-packages/{package_id}/assignments/{id}", tag = "ReviewAssignments",
    security(("bearer_auth" = [])),
    params(
        ("package_id" = Uuid, Path, description = "Review package ID"),
        ("id" = Uuid, Path, description = "Assignment ID"),
    ),
    responses((status = 200, body = review_assignment::Model), (status = 404, description = "Not found"))
)]
pub(crate) async fn get_assignment(
    State(state): State<AppState>,
    Path((_package_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Json<review_assignment::Model>, AppError> {
    let result = ReviewAssignmentService::get(&state.db, id).await?;
    Ok(Json(result))
}

#[utoipa::path(patch, path = "/api/v1/review-packages/{package_id}/assignments/{id}", tag = "ReviewAssignments",
    security(("bearer_auth" = [])),
    params(
        ("package_id" = Uuid, Path, description = "Review package ID"),
        ("id" = Uuid, Path, description = "Assignment ID"),
    ),
    request_body = UpdateReviewAssignmentInput,
    responses((status = 200, body = review_assignment::Model), (status = 404, description = "Not found"))
)]
pub(crate) async fn update_assignment(
    State(state): State<AppState>,
    Path((_package_id, id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateReviewAssignmentInput>,
) -> Result<Json<review_assignment::Model>, AppError> {
    let result = ReviewAssignmentService::update(&state.db, id, body).await?;
    Ok(Json(result))
}

#[utoipa::path(delete, path = "/api/v1/review-packages/{package_id}/assignments/{id}", tag = "ReviewAssignments",
    security(("bearer_auth" = [])),
    params(
        ("package_id" = Uuid, Path, description = "Review package ID"),
        ("id" = Uuid, Path, description = "Assignment ID"),
    ),
    responses((status = 204, description = "Deleted"), (status = 404, description = "Not found"))
)]
pub(crate) async fn delete_assignment(
    State(state): State<AppState>,
    Path((_package_id, id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, AppError> {
    ReviewAssignmentService::delete(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
