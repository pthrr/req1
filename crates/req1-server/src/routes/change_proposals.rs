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

#[utoipa::path(get, path = "/api/v1/modules/{module_id}/change-proposals",
    tag = "ChangeProposals",
    security(("bearer_auth" = [])),
    params(
        ("module_id" = Uuid, Path, description = "Module ID"),
        Pagination,
    ),
    responses((status = 200, body = PaginatedResponse<change_proposal::Model>))
)]
pub(crate) async fn list_change_proposals(
    State(state): State<AppState>,
    Path(module_id): Path<Uuid>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<PaginatedResponse<change_proposal::Model>>, AppError> {
    let result =
        ChangeProposalService::list(&state.db, module_id, pagination.offset, pagination.limit)
            .await?;
    Ok(Json(result))
}

#[utoipa::path(post, path = "/api/v1/modules/{module_id}/change-proposals",
    tag = "ChangeProposals",
    security(("bearer_auth" = [])),
    params(("module_id" = Uuid, Path, description = "Module ID")),
    request_body = CreateChangeProposalInput,
    responses((status = 201, body = change_proposal::Model))
)]
pub(crate) async fn create_change_proposal(
    State(state): State<AppState>,
    Path(module_id): Path<Uuid>,
    Json(body): Json<CreateChangeProposalInput>,
) -> Result<(axum::http::StatusCode, Json<change_proposal::Model>), AppError> {
    let input = CreateChangeProposalInput { module_id, ..body };
    let result = ChangeProposalService::create(&state.db, input).await?;
    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

#[utoipa::path(get, path = "/api/v1/modules/{module_id}/change-proposals/{id}",
    tag = "ChangeProposals",
    security(("bearer_auth" = [])),
    params(
        ("module_id" = Uuid, Path, description = "Module ID"),
        ("id" = Uuid, Path, description = "Change proposal ID"),
    ),
    responses(
        (status = 200, body = change_proposal::Model),
        (status = 404, description = "Not found"),
    )
)]
pub(crate) async fn get_change_proposal(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Json<change_proposal::Model>, AppError> {
    let result = ChangeProposalService::get(&state.db, id).await?;
    Ok(Json(result))
}

#[utoipa::path(patch, path = "/api/v1/modules/{module_id}/change-proposals/{id}",
    tag = "ChangeProposals",
    security(("bearer_auth" = [])),
    params(
        ("module_id" = Uuid, Path, description = "Module ID"),
        ("id" = Uuid, Path, description = "Change proposal ID"),
    ),
    request_body = UpdateChangeProposalInput,
    responses(
        (status = 200, body = change_proposal::Model),
        (status = 404, description = "Not found"),
    )
)]
pub(crate) async fn update_change_proposal(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateChangeProposalInput>,
) -> Result<Json<change_proposal::Model>, AppError> {
    let result = ChangeProposalService::update(&state.db, id, body).await?;
    Ok(Json(result))
}

#[utoipa::path(delete, path = "/api/v1/modules/{module_id}/change-proposals/{id}",
    tag = "ChangeProposals",
    security(("bearer_auth" = [])),
    params(
        ("module_id" = Uuid, Path, description = "Module ID"),
        ("id" = Uuid, Path, description = "Change proposal ID"),
    ),
    responses(
        (status = 204, description = "Deleted"),
        (status = 404, description = "Not found"),
    )
)]
pub(crate) async fn delete_change_proposal(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, AppError> {
    ChangeProposalService::delete(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
