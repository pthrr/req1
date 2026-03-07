use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{error::AppError, state::AppState};
use entity::e_signature;
use req1_core::auth::AuthUser;
use req1_core::service::e_signature::{ESignatureService, SignInput};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/e-signatures", post(create_signature))
        .route(
            "/e-signatures/entity/{entity_type}/{entity_id}",
            get(list_signatures),
        )
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct CreateSignatureRequest {
    entity_type: String,
    entity_id: Uuid,
    password: String,
    meaning: String,
}

#[utoipa::path(post, path = "/api/v1/e-signatures", tag = "ESignatures",
    security(("bearer_auth" = [])),
    request_body = CreateSignatureRequest,
    responses((status = 201, body = e_signature::Model))
)]
pub(crate) async fn create_signature(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(body): Json<CreateSignatureRequest>,
) -> Result<(axum::http::StatusCode, Json<e_signature::Model>), AppError> {
    let input = SignInput {
        password: body.password,
        meaning: body.meaning,
        ip_address: None,
    };
    let result = ESignatureService::sign(
        &state.db,
        auth_user.id,
        &body.entity_type,
        body.entity_id,
        input,
    )
    .await?;
    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

#[utoipa::path(get, path = "/api/v1/e-signatures/entity/{entity_type}/{entity_id}",
    tag = "ESignatures",
    security(("bearer_auth" = [])),
    params(
        ("entity_type" = String, Path, description = "Entity type"),
        ("entity_id" = Uuid, Path, description = "Entity ID"),
    ),
    responses((status = 200, body = Vec<e_signature::Model>))
)]
pub(crate) async fn list_signatures(
    State(state): State<AppState>,
    Path((entity_type, entity_id)): Path<(String, Uuid)>,
) -> Result<Json<Vec<e_signature::Model>>, AppError> {
    let result = ESignatureService::list_for_entity(&state.db, &entity_type, entity_id).await?;
    Ok(Json(result))
}
