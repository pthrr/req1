use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use uuid::Uuid;

use crate::{error::AppError, state::AppState};
use req1_core::service::validation_service::{ValidationReport, ValidationService};

pub fn routes() -> Router<AppState> {
    Router::new().route("/modules/{module_id}/validate", get(validate_module))
}

#[utoipa::path(get, path = "/api/v1/modules/{module_id}/validate", tag = "Validation",
    security(("bearer_auth" = [])),
    params(("module_id" = Uuid, Path, description = "Module ID")),
    responses((status = 200, body = ValidationReport))
)]
pub(crate) async fn validate_module(
    State(state): State<AppState>,
    Path(module_id): Path<Uuid>,
) -> Result<Json<ValidationReport>, AppError> {
    let report = ValidationService::validate(&state.db, module_id).await?;
    Ok(Json(report))
}
