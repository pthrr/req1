use axum::{
    Router,
    extract::{Path, State},
    routing::{delete, get, patch},
};
use uuid::Uuid;

use crate::{error::AppError, state::AppState};
use req1_core::service::lifecycle::{
    CreateLifecycleModelInput, LifecycleService, UpdateLifecycleModelInput,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/modules/{module_id}/lifecycle-models",
            get(list_lifecycle_models).post(create_lifecycle_model),
        )
        .route(
            "/modules/{module_id}/lifecycle-models/{id}",
            get(get_lifecycle_model)
                .merge(patch(update_lifecycle_model))
                .merge(delete(delete_lifecycle_model)),
        )
}

#[utoipa::path(get, path = "/api/v1/modules/{module_id}/lifecycle-models", tag = "Lifecycle",
    security(("bearer_auth" = [])),
    params(("module_id" = Uuid, Path, description = "Module ID")),
    responses((status = 200, body = Vec<entity::lifecycle_model::Model>))
)]
pub(crate) async fn list_lifecycle_models(
    State(state): State<AppState>,
    Path(module_id): Path<Uuid>,
) -> Result<axum::Json<Vec<entity::lifecycle_model::Model>>, AppError> {
    let items = LifecycleService::list(&state.db, module_id).await?;
    Ok(axum::Json(items))
}

#[utoipa::path(post, path = "/api/v1/modules/{module_id}/lifecycle-models", tag = "Lifecycle",
    security(("bearer_auth" = [])),
    params(("module_id" = Uuid, Path, description = "Module ID")),
    request_body = CreateLifecycleModelInput,
    responses((status = 200, body = entity::lifecycle_model::Model))
)]
pub(crate) async fn create_lifecycle_model(
    State(state): State<AppState>,
    Path(module_id): Path<Uuid>,
    axum::Json(input): axum::Json<CreateLifecycleModelInput>,
) -> Result<axum::Json<entity::lifecycle_model::Model>, AppError> {
    let model = LifecycleService::create(&state.db, module_id, input).await?;
    Ok(axum::Json(model))
}

#[utoipa::path(get, path = "/api/v1/modules/{module_id}/lifecycle-models/{id}", tag = "Lifecycle",
    security(("bearer_auth" = [])),
    params(
        ("module_id" = Uuid, Path, description = "Module ID"),
        ("id" = Uuid, Path, description = "Lifecycle model ID"),
    ),
    responses(
        (status = 200, body = entity::lifecycle_model::Model),
        (status = 404, description = "Not found"),
    )
)]
pub(crate) async fn get_lifecycle_model(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
) -> Result<axum::Json<entity::lifecycle_model::Model>, AppError> {
    let model = LifecycleService::get(&state.db, id).await?;
    Ok(axum::Json(model))
}

#[utoipa::path(patch, path = "/api/v1/modules/{module_id}/lifecycle-models/{id}", tag = "Lifecycle",
    security(("bearer_auth" = [])),
    params(
        ("module_id" = Uuid, Path, description = "Module ID"),
        ("id" = Uuid, Path, description = "Lifecycle model ID"),
    ),
    request_body = UpdateLifecycleModelInput,
    responses(
        (status = 200, body = entity::lifecycle_model::Model),
        (status = 404, description = "Not found"),
    )
)]
pub(crate) async fn update_lifecycle_model(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
    axum::Json(input): axum::Json<UpdateLifecycleModelInput>,
) -> Result<axum::Json<entity::lifecycle_model::Model>, AppError> {
    let model = LifecycleService::update(&state.db, id, input).await?;
    Ok(axum::Json(model))
}

#[utoipa::path(delete, path = "/api/v1/modules/{module_id}/lifecycle-models/{id}",
    tag = "Lifecycle",
    security(("bearer_auth" = [])),
    params(
        ("module_id" = Uuid, Path, description = "Module ID"),
        ("id" = Uuid, Path, description = "Lifecycle model ID"),
    ),
    responses((status = 200, description = "Deleted"))
)]
pub(crate) async fn delete_lifecycle_model(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
) -> Result<axum::Json<()>, AppError> {
    LifecycleService::delete(&state.db, id).await?;
    Ok(axum::Json(()))
}
