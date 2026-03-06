use axum::{
    Router,
    extract::{Path, State},
    routing::{delete, get, patch},
};
use uuid::Uuid;

use crate::{error::AppError, state::AppState};
use req1_core::service::test::{
    CreateTestCaseInput, CreateTestExecutionInput, TestService, UpdateTestCaseInput,
    UpdateTestExecutionInput,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/modules/{module_id}/test-cases",
            get(list_test_cases).post(create_test_case),
        )
        .route(
            "/modules/{module_id}/test-cases/{id}",
            get(get_test_case)
                .merge(patch(update_test_case))
                .merge(delete(delete_test_case)),
        )
        .route(
            "/test-cases/{test_case_id}/executions",
            get(list_test_executions).post(create_test_execution),
        )
        .route(
            "/test-cases/{test_case_id}/executions/{id}",
            get(get_test_execution)
                .merge(patch(update_test_execution))
                .merge(delete(delete_test_execution)),
        )
        .route(
            "/modules/{module_id}/test-coverage",
            get(get_test_coverage),
        )
        .route(
            "/modules/{module_id}/test-dashboard",
            get(get_test_dashboard),
        )
}

async fn list_test_cases(
    State(state): State<AppState>,
    Path(module_id): Path<Uuid>,
) -> Result<axum::Json<Vec<entity::test_case::Model>>, AppError> {
    let items = TestService::list_test_cases(&state.db, module_id).await?;
    Ok(axum::Json(items))
}

async fn create_test_case(
    State(state): State<AppState>,
    Path(module_id): Path<Uuid>,
    axum::Json(input): axum::Json<CreateTestCaseInput>,
) -> Result<axum::Json<entity::test_case::Model>, AppError> {
    let model = TestService::create_test_case(&state.db, module_id, input).await?;
    Ok(axum::Json(model))
}

async fn get_test_case(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
) -> Result<axum::Json<entity::test_case::Model>, AppError> {
    let model = TestService::get_test_case(&state.db, id).await?;
    Ok(axum::Json(model))
}

async fn update_test_case(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
    axum::Json(input): axum::Json<UpdateTestCaseInput>,
) -> Result<axum::Json<entity::test_case::Model>, AppError> {
    let model = TestService::update_test_case(&state.db, id, input).await?;
    Ok(axum::Json(model))
}

async fn delete_test_case(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
) -> Result<axum::Json<()>, AppError> {
    TestService::delete_test_case(&state.db, id).await?;
    Ok(axum::Json(()))
}

async fn list_test_executions(
    State(state): State<AppState>,
    Path(test_case_id): Path<Uuid>,
) -> Result<axum::Json<Vec<entity::test_execution::Model>>, AppError> {
    let items = TestService::list_test_executions(&state.db, test_case_id).await?;
    Ok(axum::Json(items))
}

async fn create_test_execution(
    State(state): State<AppState>,
    Path(test_case_id): Path<Uuid>,
    axum::Json(input): axum::Json<CreateTestExecutionInput>,
) -> Result<axum::Json<entity::test_execution::Model>, AppError> {
    let model = TestService::create_test_execution(&state.db, test_case_id, input).await?;
    Ok(axum::Json(model))
}

async fn get_test_execution(
    State(state): State<AppState>,
    Path((_test_case_id, id)): Path<(Uuid, Uuid)>,
) -> Result<axum::Json<entity::test_execution::Model>, AppError> {
    let model = TestService::get_test_execution(&state.db, id).await?;
    Ok(axum::Json(model))
}

async fn update_test_execution(
    State(state): State<AppState>,
    Path((_test_case_id, id)): Path<(Uuid, Uuid)>,
    axum::Json(input): axum::Json<UpdateTestExecutionInput>,
) -> Result<axum::Json<entity::test_execution::Model>, AppError> {
    let model = TestService::update_test_execution(&state.db, id, input).await?;
    Ok(axum::Json(model))
}

async fn delete_test_execution(
    State(state): State<AppState>,
    Path((_test_case_id, id)): Path<(Uuid, Uuid)>,
) -> Result<axum::Json<()>, AppError> {
    TestService::delete_test_execution(&state.db, id).await?;
    Ok(axum::Json(()))
}

async fn get_test_coverage(
    State(state): State<AppState>,
    Path(module_id): Path<Uuid>,
) -> Result<axum::Json<req1_core::service::test::TestCoverageResponse>, AppError> {
    let coverage = TestService::coverage(&state.db, module_id).await?;
    Ok(axum::Json(coverage))
}

async fn get_test_dashboard(
    State(state): State<AppState>,
    Path(module_id): Path<Uuid>,
) -> Result<axum::Json<req1_core::service::test::TestDashboardSummary>, AppError> {
    let dashboard = TestService::dashboard(&state.db, module_id).await?;
    Ok(axum::Json(dashboard))
}
