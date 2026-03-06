use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    routing::{get, post},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{error::AppError, state::AppState};
use entity::review_package;
use req1_core::auth::AuthUser;
use req1_core::{
    PaginatedResponse, Pagination,
    service::review_package::{
        CreateReviewPackageInput, ReviewPackageService, UpdateReviewPackageInput, VotingSummary,
    },
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/modules/{module_id}/review-packages",
            get(list_review_packages).post(create_review_package),
        )
        .route(
            "/modules/{module_id}/review-packages/{id}",
            get(get_review_package)
                .patch(update_review_package)
                .delete(delete_review_package),
        )
        .route(
            "/modules/{module_id}/review-packages/{id}/transition",
            post(transition_status),
        )
        .route(
            "/modules/{module_id}/review-packages/voting-summary",
            get(voting_summary),
        )
}

async fn list_review_packages(
    State(state): State<AppState>,
    Path(module_id): Path<Uuid>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<PaginatedResponse<review_package::Model>>, AppError> {
    let result =
        ReviewPackageService::list(&state.db, module_id, pagination.offset, pagination.limit)
            .await?;
    Ok(Json(result))
}

async fn create_review_package(
    State(state): State<AppState>,
    Path(module_id): Path<Uuid>,
    Json(body): Json<CreateReviewPackageInput>,
) -> Result<(axum::http::StatusCode, Json<review_package::Model>), AppError> {
    let input = CreateReviewPackageInput { module_id, ..body };
    let result = ReviewPackageService::create(&state.db, input).await?;
    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

async fn get_review_package(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Json<review_package::Model>, AppError> {
    let result = ReviewPackageService::get(&state.db, id).await?;
    Ok(Json(result))
}

async fn update_review_package(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateReviewPackageInput>,
) -> Result<Json<review_package::Model>, AppError> {
    let result = ReviewPackageService::update(&state.db, id, body).await?;
    Ok(Json(result))
}

async fn delete_review_package(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, AppError> {
    ReviewPackageService::delete(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

async fn voting_summary(
    State(state): State<AppState>,
    Path(module_id): Path<Uuid>,
) -> Result<Json<Vec<VotingSummary>>, AppError> {
    let result = ReviewPackageService::voting_summary(&state.db, module_id).await?;
    Ok(Json(result))
}

#[derive(Debug, Deserialize)]
struct TransitionRequest {
    status: String,
    password: Option<String>,
    meaning: Option<String>,
}

async fn transition_status(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
    Json(body): Json<TransitionRequest>,
) -> Result<Json<review_package::Model>, AppError> {
    let sign_input = match (body.password, body.meaning) {
        (Some(password), Some(meaning)) => {
            Some(req1_core::service::e_signature::SignInput {
                password,
                meaning,
                ip_address: None,
            })
        }
        _ => None,
    };

    let result = ReviewPackageService::transition_status(
        &state.db,
        id,
        &body.status,
        auth_user.id,
        sign_input,
    )
    .await?;
    Ok(Json(result))
}
