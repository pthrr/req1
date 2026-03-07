use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use uuid::Uuid;

use crate::{error::AppError, state::AppState};
use req1_core::PaginatedResponse;
use req1_core::auth::AuthUser;
use req1_core::service::diagram::{
    CreateDiagramInput, DiagramService, ListDiagramsFilter, UpdateDiagramInput,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/modules/{module_id}/diagrams",
            get(list_diagrams).post(create_diagram),
        )
        .route(
            "/modules/{module_id}/diagrams/{id}",
            get(get_diagram)
                .patch(update_diagram)
                .delete(delete_diagram),
        )
}

#[utoipa::path(get, path = "/api/v1/modules/{module_id}/diagrams", tag = "Diagrams",
    security(("bearer_auth" = [])),
    params(
        ("module_id" = Uuid, Path, description = "Module ID"),
        ListDiagramsFilter,
    ),
    responses((status = 200, body = PaginatedResponse<entity::diagram::Model>))
)]
pub(crate) async fn list_diagrams(
    State(state): State<AppState>,
    Path(module_id): Path<Uuid>,
    Query(filter): Query<ListDiagramsFilter>,
) -> Result<Json<PaginatedResponse<entity::diagram::Model>>, AppError> {
    let result = DiagramService::list(&state.db, module_id, filter).await?;
    Ok(Json(result))
}

#[utoipa::path(post, path = "/api/v1/modules/{module_id}/diagrams", tag = "Diagrams",
    security(("bearer_auth" = [])),
    params(("module_id" = Uuid, Path, description = "Module ID")),
    request_body = CreateDiagramInput,
    responses((status = 201, body = entity::diagram::Model))
)]
pub(crate) async fn create_diagram(
    State(state): State<AppState>,
    Path(module_id): Path<Uuid>,
    Extension(auth_user): Extension<AuthUser>,
    Json(body): Json<CreateDiagramInput>,
) -> Result<(axum::http::StatusCode, Json<entity::diagram::Model>), AppError> {
    let input = CreateDiagramInput {
        module_id,
        created_by: Some(auth_user.id),
        ..body
    };
    let result = DiagramService::create(&state.db, input).await?;
    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

#[utoipa::path(get, path = "/api/v1/modules/{module_id}/diagrams/{id}", tag = "Diagrams",
    security(("bearer_auth" = [])),
    params(
        ("module_id" = Uuid, Path, description = "Module ID"),
        ("id" = Uuid, Path, description = "Diagram ID"),
    ),
    responses((status = 200, body = entity::diagram::Model), (status = 404, description = "Not found"))
)]
pub(crate) async fn get_diagram(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Json<entity::diagram::Model>, AppError> {
    let result = DiagramService::get(&state.db, id).await?;
    Ok(Json(result))
}

#[utoipa::path(patch, path = "/api/v1/modules/{module_id}/diagrams/{id}", tag = "Diagrams",
    security(("bearer_auth" = [])),
    params(
        ("module_id" = Uuid, Path, description = "Module ID"),
        ("id" = Uuid, Path, description = "Diagram ID"),
    ),
    request_body = UpdateDiagramInput,
    responses((status = 200, body = entity::diagram::Model), (status = 404, description = "Not found"))
)]
pub(crate) async fn update_diagram(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateDiagramInput>,
) -> Result<Json<entity::diagram::Model>, AppError> {
    let result = DiagramService::update(&state.db, id, body).await?;
    Ok(Json(result))
}

#[utoipa::path(delete, path = "/api/v1/modules/{module_id}/diagrams/{id}", tag = "Diagrams",
    security(("bearer_auth" = [])),
    params(
        ("module_id" = Uuid, Path, description = "Module ID"),
        ("id" = Uuid, Path, description = "Diagram ID"),
    ),
    responses((status = 204, description = "Deleted"), (status = 404, description = "Not found"))
)]
pub(crate) async fn delete_diagram(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, AppError> {
    DiagramService::delete(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
