use axum::{Router, extract::State, http::StatusCode, routing::get};
use sea_orm::DatabaseConnection;
use serde_json::json;

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health/live", get(liveness))
        .route("/health/ready", get(readiness))
}

async fn liveness() -> axum::Json<serde_json::Value> {
    axum::Json(json!({"status": "ok"}))
}

async fn readiness(
    State(state): State<AppState>,
) -> Result<axum::Json<serde_json::Value>, StatusCode> {
    ping_db(&state.db)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok(axum::Json(json!({"status": "ok"})))
}

async fn ping_db(db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    use sea_orm::ConnectionTrait;
    let _ = db.execute_unprepared("SELECT 1").await?;
    Ok(())
}
