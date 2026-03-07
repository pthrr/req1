use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use uuid::Uuid;

use crate::{error::AppError, state::AppState};
use entity::{link, link_type};
use req1_core::{
    PaginatedResponse,
    service::link::{
        CreateLinkInput, CreateLinkTypeInput, LinkService, ListLinksFilter, UpdateLinkInput,
    },
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/links", get(list_links).post(create_link))
        .route(
            "/links/{id}",
            get(get_link).patch(update_link).delete(delete_link),
        )
        .route("/link-types", get(list_link_types).post(create_link_type))
}

#[utoipa::path(get, path = "/api/v1/links", tag = "Links",
    security(("bearer_auth" = [])),
    params(ListLinksFilter),
    responses((status = 200, body = PaginatedResponse<link::Model>))
)]
pub(crate) async fn list_links(
    State(state): State<AppState>,
    Query(filter): Query<ListLinksFilter>,
) -> Result<Json<PaginatedResponse<link::Model>>, AppError> {
    let result = LinkService::list(&state.db, filter).await?;
    Ok(Json(result))
}

#[utoipa::path(post, path = "/api/v1/links", tag = "Links",
    security(("bearer_auth" = [])),
    request_body = CreateLinkInput,
    responses((status = 201, body = link::Model))
)]
pub(crate) async fn create_link(
    State(state): State<AppState>,
    Json(body): Json<CreateLinkInput>,
) -> Result<(axum::http::StatusCode, Json<link::Model>), AppError> {
    let result = LinkService::create(&state.db, body).await?;
    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

#[utoipa::path(get, path = "/api/v1/links/{id}", tag = "Links",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Link ID")),
    responses((status = 200, body = link::Model), (status = 404, description = "Not found"))
)]
pub(crate) async fn get_link(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<link::Model>, AppError> {
    let result = LinkService::get(&state.db, id).await?;
    Ok(Json(result))
}

#[utoipa::path(patch, path = "/api/v1/links/{id}", tag = "Links",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Link ID")),
    request_body = UpdateLinkInput,
    responses((status = 200, body = link::Model), (status = 404, description = "Not found"))
)]
pub(crate) async fn update_link(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateLinkInput>,
) -> Result<Json<link::Model>, AppError> {
    let result = LinkService::update(&state.db, id, body).await?;
    Ok(Json(result))
}

#[utoipa::path(delete, path = "/api/v1/links/{id}", tag = "Links",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Link ID")),
    responses((status = 204, description = "Deleted"), (status = 404, description = "Not found"))
)]
pub(crate) async fn delete_link(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    LinkService::delete(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[utoipa::path(get, path = "/api/v1/link-types", tag = "Links",
    security(("bearer_auth" = [])),
    responses((status = 200, body = Vec<link_type::Model>))
)]
pub(crate) async fn list_link_types(
    State(state): State<AppState>,
) -> Result<Json<Vec<link_type::Model>>, AppError> {
    let items = LinkService::list_link_types(&state.db).await?;
    Ok(Json(items))
}

#[utoipa::path(post, path = "/api/v1/link-types", tag = "Links",
    security(("bearer_auth" = [])),
    request_body = CreateLinkTypeInput,
    responses((status = 201, body = link_type::Model))
)]
pub(crate) async fn create_link_type(
    State(state): State<AppState>,
    Json(body): Json<CreateLinkTypeInput>,
) -> Result<(axum::http::StatusCode, Json<link_type::Model>), AppError> {
    let result = LinkService::create_link_type(&state.db, body).await?;
    Ok((axum::http::StatusCode::CREATED, Json(result)))
}
