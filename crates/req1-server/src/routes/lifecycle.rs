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

async fn list_lifecycle_models(
    State(state): State<AppState>,
    Path(module_id): Path<Uuid>,
) -> Result<axum::Json<Vec<entity::lifecycle_model::Model>>, AppError> {
    let items = LifecycleService::list(&state.db, module_id).await?;
    Ok(axum::Json(items))
}

async fn create_lifecycle_model(
    State(state): State<AppState>,
    Path(module_id): Path<Uuid>,
    axum::Json(input): axum::Json<CreateLifecycleModelInput>,
) -> Result<axum::Json<entity::lifecycle_model::Model>, AppError> {
    let model = LifecycleService::create(&state.db, module_id, input).await?;
    Ok(axum::Json(model))
}

async fn get_lifecycle_model(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
) -> Result<axum::Json<entity::lifecycle_model::Model>, AppError> {
    let model = LifecycleService::get(&state.db, id).await?;
    Ok(axum::Json(model))
}

async fn update_lifecycle_model(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
    axum::Json(input): axum::Json<UpdateLifecycleModelInput>,
) -> Result<axum::Json<entity::lifecycle_model::Model>, AppError> {
    let model = LifecycleService::update(&state.db, id, input).await?;
    Ok(axum::Json(model))
}

async fn delete_lifecycle_model(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
) -> Result<axum::Json<()>, AppError> {
    LifecycleService::delete(&state.db, id).await?;
    Ok(axum::Json(()))
}
