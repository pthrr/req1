use axum::{Router, extract::State, http::StatusCode, routing::get};
use sea_orm::DatabaseConnection;
use serde::Serialize;
use serde_json::json;
use utoipa::ToSchema;

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health/live", get(liveness))
        .route("/health/ready", get(readiness))
}

#[derive(Serialize, ToSchema)]
pub(crate) struct HealthResponse {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    build_sha: Option<String>,
}

#[utoipa::path(get, path = "/health/live", tag = "Health",
    responses((status = 200, body = HealthResponse))
)]
pub(crate) async fn liveness(State(state): State<AppState>) -> axum::Json<serde_json::Value> {
    axum::Json(build_health_body(&state))
}

#[utoipa::path(get, path = "/health/ready", tag = "Health",
    responses(
        (status = 200, body = HealthResponse),
        (status = 503, description = "Service unavailable")
    )
)]
pub(crate) async fn readiness(
    State(state): State<AppState>,
) -> Result<axum::Json<serde_json::Value>, StatusCode> {
    ping_db(&state.db)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok(axum::Json(build_health_body(&state)))
}

fn build_health_body(state: &AppState) -> serde_json::Value {
    match &state.config.build_sha {
        Some(sha) => json!({"status": "ok", "build_sha": sha}),
        None => json!({"status": "ok"}),
    }
}

async fn ping_db(db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    use sea_orm::ConnectionTrait;
    let _ = db.execute_unprepared("SELECT 1").await?;
    Ok(())
}
