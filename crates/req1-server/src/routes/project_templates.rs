use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use uuid::Uuid;

use crate::{error::AppError, state::AppState};
use req1_core::auth::AuthUser;
use req1_core::service::project_template::{
    CreateTemplateInput, InstantiateInput, ProjectTemplateService, UpdateTemplateInput,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/project-templates",
            get(list_templates).post(create_template),
        )
        .route(
            "/project-templates/{id}",
            get(get_template)
                .patch(update_template)
                .delete(delete_template),
        )
        .route(
            "/project-templates/{id}/instantiate",
            axum::routing::post(instantiate_template),
        )
}

#[utoipa::path(get, path = "/api/v1/project-templates", tag = "ProjectTemplates",
    security(("bearer_auth" = [])),
    responses((status = 200, body = Vec<entity::project_template::Model>))
)]
pub(crate) async fn list_templates(
    State(state): State<AppState>,
) -> Result<Json<Vec<entity::project_template::Model>>, AppError> {
    let items = ProjectTemplateService::list(&state.db).await?;
    Ok(Json(items))
}

#[utoipa::path(post, path = "/api/v1/project-templates", tag = "ProjectTemplates",
    security(("bearer_auth" = [])),
    request_body = CreateTemplateInput,
    responses((status = 201, body = entity::project_template::Model))
)]
pub(crate) async fn create_template(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(mut body): Json<CreateTemplateInput>,
) -> Result<(StatusCode, Json<entity::project_template::Model>), AppError> {
    body.created_by = Some(auth_user.id);
    let result = ProjectTemplateService::create(&state.db, body).await?;
    Ok((StatusCode::CREATED, Json(result)))
}

#[utoipa::path(get, path = "/api/v1/project-templates/{id}", tag = "ProjectTemplates",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Template ID")),
    responses(
        (status = 200, body = entity::project_template::Model),
        (status = 404, description = "Not found"),
    )
)]
pub(crate) async fn get_template(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<entity::project_template::Model>, AppError> {
    let result = ProjectTemplateService::get(&state.db, id).await?;
    Ok(Json(result))
}

#[utoipa::path(patch, path = "/api/v1/project-templates/{id}", tag = "ProjectTemplates",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Template ID")),
    request_body = UpdateTemplateInput,
    responses(
        (status = 200, body = entity::project_template::Model),
        (status = 404, description = "Not found"),
    )
)]
pub(crate) async fn update_template(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateTemplateInput>,
) -> Result<Json<entity::project_template::Model>, AppError> {
    let result = ProjectTemplateService::update(&state.db, id, body).await?;
    Ok(Json(result))
}

#[utoipa::path(delete, path = "/api/v1/project-templates/{id}", tag = "ProjectTemplates",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Template ID")),
    responses((status = 204, description = "Deleted"), (status = 404, description = "Not found"))
)]
pub(crate) async fn delete_template(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    ProjectTemplateService::delete(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(post, path = "/api/v1/project-templates/{id}/instantiate",
    tag = "ProjectTemplates",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Template ID")),
    request_body = InstantiateInput,
    responses((status = 201, body = req1_core::service::project_template::InstantiateResult))
)]
pub(crate) async fn instantiate_template(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<InstantiateInput>,
) -> Result<
    (
        StatusCode,
        Json<req1_core::service::project_template::InstantiateResult>,
    ),
    AppError,
> {
    let result = ProjectTemplateService::instantiate(&state.db, id, body).await?;
    Ok((StatusCode::CREATED, Json(result)))
}
