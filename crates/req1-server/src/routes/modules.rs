use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use uuid::Uuid;

use crate::{error::AppError, state::AppState};
use entity::module;
use req1_core::{
    PaginatedResponse,
    service::module::{
        CreateModuleFromTemplateInput, CreateModuleInput, ListModulesFilter, ModuleService,
        UpdateModuleInput,
    },
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/modules", get(list_modules).post(create_module))
        .route(
            "/modules/from-template",
            axum::routing::post(create_module_from_template),
        )
        .route(
            "/modules/{id}",
            get(get_module).patch(update_module).delete(delete_module),
        )
}

async fn list_modules(
    State(state): State<AppState>,
    Query(filter): Query<ListModulesFilter>,
) -> Result<Json<PaginatedResponse<module::Model>>, AppError> {
    let result = ModuleService::list(&state.db, filter).await?;
    Ok(Json(result))
}

async fn create_module(
    State(state): State<AppState>,
    Json(body): Json<CreateModuleInput>,
) -> Result<(axum::http::StatusCode, Json<module::Model>), AppError> {
    let result = ModuleService::create(&state.db, body).await?;
    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

async fn get_module(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<module::Model>, AppError> {
    let result = ModuleService::get(&state.db, id).await?;
    Ok(Json(result))
}

async fn update_module(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateModuleInput>,
) -> Result<Json<module::Model>, AppError> {
    let result = ModuleService::update(&state.db, id, body).await?;
    Ok(Json(result))
}

async fn delete_module(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    ModuleService::delete(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

async fn create_module_from_template(
    State(state): State<AppState>,
    Json(body): Json<CreateModuleFromTemplateInput>,
) -> Result<(axum::http::StatusCode, Json<module::Model>), AppError> {
    let result = ModuleService::create_from_template(&state.db, body).await?;
    Ok((axum::http::StatusCode::CREATED, Json(result)))
}
