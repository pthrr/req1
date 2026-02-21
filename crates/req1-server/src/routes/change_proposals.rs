use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use uuid::Uuid;

use crate::{error::AppError, state::AppState};
use entity::change_proposal;
use req1_core::{
    PaginatedResponse, Pagination,
    service::change_proposal::{
        ChangeProposalService, CreateChangeProposalInput, UpdateChangeProposalInput,
    },
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/modules/{module_id}/change-proposals",
            get(list_change_proposals).post(create_change_proposal),
        )
        .route(
            "/modules/{module_id}/change-proposals/{id}",
            get(get_change_proposal)
                .patch(update_change_proposal)
                .delete(delete_change_proposal),
        )
}

async fn list_change_proposals(
    State(state): State<AppState>,
    Path(module_id): Path<Uuid>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<PaginatedResponse<change_proposal::Model>>, AppError> {
    let result =
        ChangeProposalService::list(&state.db, module_id, pagination.offset, pagination.limit)
            .await?;
    Ok(Json(result))
}

async fn create_change_proposal(
    State(state): State<AppState>,
    Path(module_id): Path<Uuid>,
    Json(body): Json<CreateChangeProposalInput>,
) -> Result<(axum::http::StatusCode, Json<change_proposal::Model>), AppError> {
    let input = CreateChangeProposalInput { module_id, ..body };
    let result = ChangeProposalService::create(&state.db, input).await?;
    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

async fn get_change_proposal(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Json<change_proposal::Model>, AppError> {
    let result = ChangeProposalService::get(&state.db, id).await?;
    Ok(Json(result))
}

async fn update_change_proposal(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateChangeProposalInput>,
) -> Result<Json<change_proposal::Model>, AppError> {
    let result = ChangeProposalService::update(&state.db, id, body).await?;
    Ok(Json(result))
}

async fn delete_change_proposal(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, AppError> {
    ChangeProposalService::delete(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
