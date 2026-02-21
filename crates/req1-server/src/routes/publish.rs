use axum::{
    Router,
    extract::{Path, Query, State},
    response::{Html, IntoResponse, Response},
    routing::get,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{error::AppError, state::AppState};
use req1_core::service::publish::PublishService;

pub fn routes() -> Router<AppState> {
    Router::new().route("/modules/{module_id}/publish", get(publish_module))
}

#[derive(Debug, Deserialize)]
struct PublishQuery {
    #[serde(default = "default_format")]
    format: String,
}

fn default_format() -> String {
    "html".to_owned()
}

async fn publish_module(
    State(state): State<AppState>,
    Path(module_id): Path<Uuid>,
    Query(query): Query<PublishQuery>,
) -> Result<Response, AppError> {
    let html = PublishService::render_html(&state.db, module_id).await?;

    match query.format.as_str() {
        "html" => Ok(Html(html).into_response()),
        other => Err(AppError::BadRequest(format!(
            "unsupported format '{other}', supported: html"
        ))),
    }
}
