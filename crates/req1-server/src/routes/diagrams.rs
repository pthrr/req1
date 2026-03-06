use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use uuid::Uuid;

use crate::{error::AppError, state::AppState};
use req1_core::auth::AuthUser;
use req1_core::service::diagram::{
    CreateDiagramInput, DiagramService, ListDiagramsFilter, UpdateDiagramInput,
};
use req1_core::PaginatedResponse;

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

async fn list_diagrams(
    State(state): State<AppState>,
    Path(module_id): Path<Uuid>,
    Query(filter): Query<ListDiagramsFilter>,
) -> Result<Json<PaginatedResponse<entity::diagram::Model>>, AppError> {
    let result = DiagramService::list(&state.db, module_id, filter).await?;
    Ok(Json(result))
}

async fn create_diagram(
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

async fn get_diagram(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Json<entity::diagram::Model>, AppError> {
    let result = DiagramService::get(&state.db, id).await?;
    Ok(Json(result))
}

async fn update_diagram(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateDiagramInput>,
) -> Result<Json<entity::diagram::Model>, AppError> {
    let result = DiagramService::update(&state.db, id, body).await?;
    Ok(Json(result))
}

async fn delete_diagram(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, AppError> {
    DiagramService::delete(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
