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

#[utoipa::path(get, path = "/api/v1/modules", tag = "Modules",
    security(("bearer_auth" = [])),
    params(ListModulesFilter),
    responses((status = 200, body = PaginatedResponse<module::Model>))
)]
pub(crate) async fn list_modules(
    State(state): State<AppState>,
    Query(filter): Query<ListModulesFilter>,
) -> Result<Json<PaginatedResponse<module::Model>>, AppError> {
    let result = ModuleService::list(&state.db, filter).await?;
    Ok(Json(result))
}

#[utoipa::path(post, path = "/api/v1/modules", tag = "Modules",
    security(("bearer_auth" = [])),
    request_body = CreateModuleInput,
    responses((status = 201, body = module::Model))
)]
pub(crate) async fn create_module(
    State(state): State<AppState>,
    Json(body): Json<CreateModuleInput>,
) -> Result<(axum::http::StatusCode, Json<module::Model>), AppError> {
    let result = ModuleService::create(&state.db, body).await?;
    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

#[utoipa::path(get, path = "/api/v1/modules/{id}", tag = "Modules",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Module ID")),
    responses((status = 200, body = module::Model), (status = 404, description = "Not found"))
)]
pub(crate) async fn get_module(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<module::Model>, AppError> {
    let result = ModuleService::get(&state.db, id).await?;
    Ok(Json(result))
}

#[utoipa::path(patch, path = "/api/v1/modules/{id}", tag = "Modules",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Module ID")),
    request_body = UpdateModuleInput,
    responses((status = 200, body = module::Model), (status = 404, description = "Not found"))
)]
pub(crate) async fn update_module(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateModuleInput>,
) -> Result<Json<module::Model>, AppError> {
    let result = ModuleService::update(&state.db, id, body).await?;
    Ok(Json(result))
}

#[utoipa::path(delete, path = "/api/v1/modules/{id}", tag = "Modules",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Module ID")),
    responses((status = 204, description = "Deleted"), (status = 404, description = "Not found"))
)]
pub(crate) async fn delete_module(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    ModuleService::delete(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[utoipa::path(post, path = "/api/v1/modules/from-template", tag = "Modules",
    security(("bearer_auth" = [])),
    request_body = CreateModuleFromTemplateInput,
    responses((status = 201, body = module::Model))
)]
pub(crate) async fn create_module_from_template(
    State(state): State<AppState>,
    Json(body): Json<CreateModuleFromTemplateInput>,
) -> Result<(axum::http::StatusCode, Json<module::Model>), AppError> {
    let result = ModuleService::create_from_template(&state.db, body).await?;
    Ok((axum::http::StatusCode::CREATED, Json(result)))
}
