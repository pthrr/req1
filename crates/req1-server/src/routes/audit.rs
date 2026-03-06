use axum::{
    Extension, Json, Router,
    extract::{Query, State},
    routing::get,
};

use crate::{error::AppError, state::AppState};
use req1_core::auth::AuthUser;
use req1_core::service::audit::{AuditLogFilter, AuditService};
use req1_core::PaginatedResponse;

pub fn routes() -> Router<AppState> {
    Router::new().route("/audit-log", get(list_audit_logs))
}

async fn list_audit_logs(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(filter): Query<AuditLogFilter>,
) -> Result<Json<PaginatedResponse<entity::audit_log::Model>>, AppError> {
    if auth_user.role != "admin" {
        return Err(AppError::Forbidden(
            "only admins can view audit logs".to_string(),
        ));
    }
    let result = AuditService::list(&state.db, filter).await?;
    Ok(Json(result))
}
