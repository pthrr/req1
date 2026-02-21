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

async fn list_links(
    State(state): State<AppState>,
    Query(filter): Query<ListLinksFilter>,
) -> Result<Json<PaginatedResponse<link::Model>>, AppError> {
    let result = LinkService::list(&state.db, filter).await?;
    Ok(Json(result))
}

async fn create_link(
    State(state): State<AppState>,
    Json(body): Json<CreateLinkInput>,
) -> Result<(axum::http::StatusCode, Json<link::Model>), AppError> {
    let result = LinkService::create(&state.db, body).await?;
    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

async fn get_link(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<link::Model>, AppError> {
    let result = LinkService::get(&state.db, id).await?;
    Ok(Json(result))
}

async fn update_link(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateLinkInput>,
) -> Result<Json<link::Model>, AppError> {
    let result = LinkService::update(&state.db, id, body).await?;
    Ok(Json(result))
}

async fn delete_link(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    LinkService::delete(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

async fn list_link_types(
    State(state): State<AppState>,
) -> Result<Json<Vec<link_type::Model>>, AppError> {
    let items = LinkService::list_link_types(&state.db).await?;
    Ok(Json(items))
}

async fn create_link_type(
    State(state): State<AppState>,
    Json(body): Json<CreateLinkTypeInput>,
) -> Result<(axum::http::StatusCode, Json<link_type::Model>), AppError> {
    let result = LinkService::create_link_type(&state.db, body).await?;
    Ok((axum::http::StatusCode::CREATED, Json(result)))
}
