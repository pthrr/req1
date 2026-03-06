use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{error::AppError, state::AppState};
use req1_core::service::webhook::{
    CreateWebhookInput, UpdateWebhookInput, WebhookService,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/modules/{module_id}/webhooks",
            get(list_webhooks).post(create_webhook),
        )
        .route(
            "/modules/{module_id}/webhooks/{id}",
            get(get_webhook)
                .patch(update_webhook)
                .delete(delete_webhook),
        )
}

#[derive(Debug, Deserialize)]
struct CreateWebhookRequest {
    name: String,
    url: String,
    secret: Option<String>,
    events: Option<String>,
    active: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct UpdateWebhookRequest {
    name: Option<String>,
    url: Option<String>,
    secret: Option<String>,
    events: Option<String>,
    active: Option<bool>,
}

async fn list_webhooks(
    State(state): State<AppState>,
    Path(module_id): Path<Uuid>,
) -> Result<Json<Vec<entity::webhook::Model>>, AppError> {
    let items = WebhookService::list(&state.db, module_id).await?;
    Ok(Json(items))
}

async fn create_webhook(
    State(state): State<AppState>,
    Path(module_id): Path<Uuid>,
    Json(body): Json<CreateWebhookRequest>,
) -> Result<(StatusCode, Json<entity::webhook::Model>), AppError> {
    let result = WebhookService::create(
        &state.db,
        CreateWebhookInput {
            module_id,
            name: body.name,
            url: body.url,
            secret: body.secret,
            events: body.events,
            active: body.active,
        },
    )
    .await?;
    Ok((StatusCode::CREATED, Json(result)))
}

async fn get_webhook(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Json<entity::webhook::Model>, AppError> {
    let result = WebhookService::get(&state.db, id).await?;
    Ok(Json(result))
}

async fn update_webhook(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateWebhookRequest>,
) -> Result<Json<entity::webhook::Model>, AppError> {
    let result = WebhookService::update(
        &state.db,
        id,
        UpdateWebhookInput {
            name: body.name,
            url: body.url,
            secret: body.secret,
            events: body.events,
            active: body.active,
        },
    )
    .await?;
    Ok(Json(result))
}

async fn delete_webhook(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    WebhookService::delete(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
