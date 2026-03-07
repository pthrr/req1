use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    routing::{get, post},
};
use req1_core::auth::AuthUser;
use sea_orm::{
    ColumnTrait, EntityTrait, Order, PaginatorTrait, QueryFilter, QueryOrder, TransactionTrait,
};
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{error::AppError, state::AppState};
use entity::object_history;
use req1_core::{
    PaginatedResponse, Pagination,
    service::object::{
        CreateObjectInput, GlobalSearchResult, ListObjectsFilter, MoveObjectInput, ObjectService,
        UpdateObjectInput,
    },
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/search", get(search_global))
        .route(
            "/modules/{module_id}/objects",
            get(list_objects).post(create_object),
        )
        .route(
            "/modules/{module_id}/objects/{id}",
            get(get_object).patch(update_object).delete(delete_object),
        )
        .route("/modules/{module_id}/objects/{id}/move", post(move_object))
        .route(
            "/modules/{module_id}/objects/{id}/history",
            get(list_object_history),
        )
        .route(
            "/modules/{module_id}/objects/{id}/sync",
            post(sync_placeholder),
        )
        .route(
            "/modules/{module_id}/objects/{id}/break-link",
            post(break_placeholder_link),
        )
        .route(
            "/modules/{module_id}/sync-placeholders",
            post(sync_all_placeholders),
        )
}

#[utoipa::path(get, path = "/api/v1/modules/{module_id}/objects", tag = "Objects",
    security(("bearer_auth" = [])),
    params(
        ("module_id" = Uuid, Path, description = "Module ID"),
        ListObjectsFilter,
    ),
    responses((status = 200, body = PaginatedResponse<entity::object::Model>))
)]
pub(crate) async fn list_objects(
    State(state): State<AppState>,
    Path(module_id): Path<Uuid>,
    Query(filter): Query<ListObjectsFilter>,
) -> Result<Json<PaginatedResponse<entity::object::Model>>, AppError> {
    let result = ObjectService::list(&state.db, module_id, filter).await?;
    Ok(Json(result))
}

#[utoipa::path(post, path = "/api/v1/modules/{module_id}/objects", tag = "Objects",
    security(("bearer_auth" = [])),
    params(("module_id" = Uuid, Path, description = "Module ID")),
    request_body = CreateObjectInput,
    responses((status = 201, body = entity::object::Model))
)]
pub(crate) async fn create_object(
    State(state): State<AppState>,
    Path(module_id): Path<Uuid>,
    Extension(_auth_user): Extension<AuthUser>,
    Json(body): Json<CreateObjectInput>,
) -> Result<(axum::http::StatusCode, Json<entity::object::Model>), AppError> {
    let txn = state.db.begin().await?;
    let input = CreateObjectInput { module_id, ..body };
    let result = ObjectService::create(&txn, input).await?;
    txn.commit().await?;

    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

#[utoipa::path(get, path = "/api/v1/modules/{module_id}/objects/{id}", tag = "Objects",
    security(("bearer_auth" = [])),
    params(
        ("module_id" = Uuid, Path, description = "Module ID"),
        ("id" = Uuid, Path, description = "Object ID"),
    ),
    responses((status = 200, body = entity::object::Model), (status = 404, description = "Not found"))
)]
pub(crate) async fn get_object(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Json<entity::object::Model>, AppError> {
    let result = ObjectService::get(&state.db, id).await?;
    Ok(Json(result))
}

#[utoipa::path(patch, path = "/api/v1/modules/{module_id}/objects/{id}", tag = "Objects",
    security(("bearer_auth" = [])),
    params(
        ("module_id" = Uuid, Path, description = "Module ID"),
        ("id" = Uuid, Path, description = "Object ID"),
    ),
    request_body = UpdateObjectInput,
    responses((status = 200, body = entity::object::Model), (status = 404, description = "Not found"))
)]
pub(crate) async fn update_object(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateObjectInput>,
) -> Result<Json<entity::object::Model>, AppError> {
    let txn = state.db.begin().await?;
    let _ = ObjectService::update(&txn, id, body).await?;
    txn.commit().await?;

    let result = ObjectService::get(&state.db, id).await?;
    Ok(Json(result))
}

#[utoipa::path(delete, path = "/api/v1/modules/{module_id}/objects/{id}", tag = "Objects",
    security(("bearer_auth" = [])),
    params(
        ("module_id" = Uuid, Path, description = "Module ID"),
        ("id" = Uuid, Path, description = "Object ID"),
    ),
    responses((status = 204, description = "Deleted"), (status = 404, description = "Not found"))
)]
pub(crate) async fn delete_object(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, AppError> {
    let txn = state.db.begin().await?;
    ObjectService::delete(&txn, id).await?;
    txn.commit().await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[utoipa::path(post, path = "/api/v1/modules/{module_id}/objects/{id}/move", tag = "Objects",
    security(("bearer_auth" = [])),
    params(
        ("module_id" = Uuid, Path, description = "Module ID"),
        ("id" = Uuid, Path, description = "Object ID"),
    ),
    request_body = MoveObjectInput,
    responses((status = 200, body = entity::object::Model))
)]
pub(crate) async fn move_object(
    State(state): State<AppState>,
    Path((module_id, id)): Path<(Uuid, Uuid)>,
    Json(body): Json<MoveObjectInput>,
) -> Result<Json<entity::object::Model>, AppError> {
    let txn = state.db.begin().await?;
    let result = ObjectService::move_object(&txn, module_id, id, body).await?;
    txn.commit().await?;
    Ok(Json(result))
}

#[utoipa::path(get, path = "/api/v1/modules/{module_id}/objects/{id}/history", tag = "Objects",
    security(("bearer_auth" = [])),
    params(
        ("module_id" = Uuid, Path, description = "Module ID"),
        ("id" = Uuid, Path, description = "Object ID"),
        Pagination,
    ),
    responses((status = 200, body = PaginatedResponse<object_history::Model>))
)]
pub(crate) async fn list_object_history(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<PaginatedResponse<object_history::Model>>, AppError> {
    let paginator = object_history::Entity::find()
        .filter(object_history::Column::ObjectId.eq(id))
        .order_by(object_history::Column::Version, Order::Desc)
        .paginate(&state.db, pagination.limit);
    let total = paginator.num_items().await?;
    let page = pagination.offset / pagination.limit;
    let items = paginator.fetch_page(page).await?;

    Ok(Json(PaginatedResponse {
        items,
        total,
        offset: pagination.offset,
        limit: pagination.limit,
    }))
}

const fn default_search_limit() -> u64 {
    50
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct SearchQuery {
    q: String,
    #[serde(default = "default_search_limit")]
    limit: u64,
}

#[derive(serde::Serialize, ToSchema)]
pub(crate) struct SearchResponse {
    items: Vec<GlobalSearchResult>,
}

#[utoipa::path(get, path = "/api/v1/search", tag = "Objects",
    security(("bearer_auth" = [])),
    params(
        ("q" = String, Query, description = "Search query"),
        ("limit" = Option<u64>, Query, description = "Result limit"),
    ),
    responses((status = 200, body = SearchResponse))
)]
pub(crate) async fn search_global(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<SearchResponse>, AppError> {
    let results = ObjectService::search_global(&state.db, &query.q, query.limit).await?;
    Ok(Json(SearchResponse { items: results }))
}

#[utoipa::path(post, path = "/api/v1/modules/{module_id}/objects/{id}/sync", tag = "Objects",
    security(("bearer_auth" = [])),
    params(
        ("module_id" = Uuid, Path, description = "Module ID"),
        ("id" = Uuid, Path, description = "Object ID"),
    ),
    responses((status = 200, body = entity::object::Model))
)]
pub(crate) async fn sync_placeholder(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Json<entity::object::Model>, AppError> {
    let txn = state.db.begin().await?;
    let result = ObjectService::sync_placeholder(&txn, id).await?;
    txn.commit().await?;
    Ok(Json(result))
}

#[utoipa::path(post, path = "/api/v1/modules/{module_id}/objects/{id}/break-link", tag = "Objects",
    security(("bearer_auth" = [])),
    params(
        ("module_id" = Uuid, Path, description = "Module ID"),
        ("id" = Uuid, Path, description = "Object ID"),
    ),
    responses((status = 200, body = entity::object::Model))
)]
pub(crate) async fn break_placeholder_link(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Json<entity::object::Model>, AppError> {
    let result = ObjectService::break_placeholder_link(&state.db, id).await?;
    Ok(Json(result))
}

#[derive(serde::Serialize, ToSchema)]
pub(crate) struct SyncAllResponse {
    synced: u64,
}

#[utoipa::path(post, path = "/api/v1/modules/{module_id}/sync-placeholders", tag = "Objects",
    security(("bearer_auth" = [])),
    params(("module_id" = Uuid, Path, description = "Module ID")),
    responses((status = 200, body = SyncAllResponse))
)]
pub(crate) async fn sync_all_placeholders(
    State(state): State<AppState>,
    Path(module_id): Path<Uuid>,
) -> Result<Json<SyncAllResponse>, AppError> {
    let txn = state.db.begin().await?;
    let synced = ObjectService::sync_all_placeholders(&txn, module_id).await?;
    txn.commit().await?;
    Ok(Json(SyncAllResponse { synced }))
}
