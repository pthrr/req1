use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{error::AppError, state::AppState};
use entity::{object, script};
use req1_core::PaginatedResponse;
use req1_core::Pagination;
use req1_core::scripting::engine::{Mutation, ScriptEngine, ScriptObject, TriggerContext};
use req1_core::service::object::load_world;
use req1_core::service::scheduler::SchedulerService;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/modules/{module_id}/scripts",
            get(list_scripts).post(create_script),
        )
        .route(
            "/modules/{module_id}/scripts/{id}",
            get(get_script).patch(update_script).delete(delete_script),
        )
        .route(
            "/modules/{module_id}/scripts/{id}/test",
            axum::routing::post(test_script),
        )
        .route(
            "/modules/{module_id}/scripts/{id}/execute",
            axum::routing::post(execute_script),
        )
        .route(
            "/modules/{module_id}/scripts/{id}/layout",
            axum::routing::post(batch_layout),
        )
        .route(
            "/modules/{module_id}/scripts/{id}/executions",
            get(list_executions),
        )
}

const VALID_SCRIPT_TYPES: &[&str] = &["trigger", "layout", "action"];
const VALID_HOOK_POINTS: &[&str] = &["pre_save", "post_save", "pre_delete", "post_delete"];

fn default_script_type() -> String {
    "trigger".to_owned()
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct CreateScriptRequest {
    name: String,
    #[serde(default = "default_script_type")]
    script_type: String,
    hook_point: Option<String>,
    source_code: String,
    enabled: Option<bool>,
    priority: Option<i32>,
    cron_expression: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct UpdateScriptRequest {
    name: Option<String>,
    script_type: Option<String>,
    hook_point: Option<String>,
    source_code: Option<String>,
    enabled: Option<bool>,
    priority: Option<i32>,
    cron_expression: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct TestScriptRequest {
    /// For trigger/layout scripts: the object to run against.
    object: Option<ScriptObject>,
    /// For trigger scripts: override the `hook_point`.
    hook_point: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ExecuteResult {
    output: Vec<String>,
    mutations_applied: usize,
}

#[utoipa::path(get, path = "/api/v1/modules/{module_id}/scripts", tag = "Scripts",
    security(("bearer_auth" = [])),
    params(("module_id" = Uuid, Path, description = "Module ID")),
    responses((status = 200, body = Vec<script::Model>))
)]
pub(crate) async fn list_scripts(
    State(state): State<AppState>,
    Path(module_id): Path<Uuid>,
) -> Result<Json<Vec<script::Model>>, AppError> {
    let items = script::Entity::find()
        .filter(script::Column::ModuleId.eq(module_id))
        .all(&state.db)
        .await?;
    Ok(Json(items))
}

#[utoipa::path(post, path = "/api/v1/modules/{module_id}/scripts", tag = "Scripts",
    security(("bearer_auth" = [])),
    params(("module_id" = Uuid, Path, description = "Module ID")),
    request_body = CreateScriptRequest,
    responses((status = 201, body = script::Model))
)]
pub(crate) async fn create_script(
    State(state): State<AppState>,
    Path(module_id): Path<Uuid>,
    Json(body): Json<CreateScriptRequest>,
) -> Result<(axum::http::StatusCode, Json<script::Model>), AppError> {
    if !VALID_SCRIPT_TYPES.contains(&body.script_type.as_str()) {
        return Err(AppError::bad_request(format!(
            "invalid script_type '{}', must be one of: {VALID_SCRIPT_TYPES:?}",
            body.script_type
        )));
    }

    if body.script_type == "trigger" {
        match &body.hook_point {
            None => {
                return Err(AppError::bad_request(
                    "hook_point is required for trigger scripts".to_owned(),
                ));
            }
            Some(hp) if !VALID_HOOK_POINTS.contains(&hp.as_str()) => {
                return Err(AppError::bad_request(format!(
                    "invalid hook_point '{hp}', must be one of: {VALID_HOOK_POINTS:?}"
                )));
            }
            _ => {}
        }
    }

    let next_run = if let Some(ref cron_expr) = body.cron_expression {
        SchedulerService::validate_cron(cron_expr)?;
        Some(SchedulerService::next_run_time(cron_expr)?)
    } else {
        None
    };

    let now = chrono::Utc::now().fixed_offset();
    let id = Uuid::now_v7();

    let model = script::ActiveModel {
        id: Set(id),
        module_id: Set(module_id),
        name: Set(body.name),
        script_type: Set(body.script_type),
        hook_point: Set(body.hook_point),
        source_code: Set(body.source_code),
        enabled: Set(body.enabled.unwrap_or(true)),
        priority: Set(body.priority.unwrap_or(100)),
        cron_expression: Set(body.cron_expression),
        last_run_at: Set(None),
        next_run_at: Set(next_run),
        created_at: Set(now),
        updated_at: Set(now),
    };

    let result = model.insert(&state.db).await?;
    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

#[utoipa::path(get, path = "/api/v1/modules/{module_id}/scripts/{id}", tag = "Scripts",
    security(("bearer_auth" = [])),
    params(
        ("module_id" = Uuid, Path, description = "Module ID"),
        ("id" = Uuid, Path, description = "Script ID"),
    ),
    responses((status = 200, body = script::Model), (status = 404, description = "Not found"))
)]
pub(crate) async fn get_script(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Json<script::Model>, AppError> {
    let s = script::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found(format!("script {id} not found")))?;
    Ok(Json(s))
}

#[utoipa::path(patch, path = "/api/v1/modules/{module_id}/scripts/{id}", tag = "Scripts",
    security(("bearer_auth" = [])),
    params(
        ("module_id" = Uuid, Path, description = "Module ID"),
        ("id" = Uuid, Path, description = "Script ID"),
    ),
    request_body = UpdateScriptRequest,
    responses((status = 200, body = script::Model), (status = 404, description = "Not found"))
)]
pub(crate) async fn update_script(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateScriptRequest>,
) -> Result<Json<script::Model>, AppError> {
    let existing = script::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found(format!("script {id} not found")))?;

    let final_type = body.script_type.as_deref().unwrap_or(&existing.script_type);
    let final_hook = body
        .hook_point
        .as_deref()
        .or(existing.hook_point.as_deref());

    if let Some(st) = body
        .script_type
        .as_deref()
        .filter(|st| !VALID_SCRIPT_TYPES.contains(st))
    {
        return Err(AppError::bad_request(format!(
            "invalid script_type '{st}', must be one of: {VALID_SCRIPT_TYPES:?}"
        )));
    }

    if final_type == "trigger" {
        match final_hook {
            None => {
                return Err(AppError::bad_request(
                    "hook_point is required for trigger scripts".to_owned(),
                ));
            }
            Some(hp) if !VALID_HOOK_POINTS.contains(&hp) => {
                return Err(AppError::bad_request(format!(
                    "invalid hook_point '{hp}', must be one of: {VALID_HOOK_POINTS:?}"
                )));
            }
            _ => {}
        }
    }

    let mut active: script::ActiveModel = existing.into();
    if let Some(name) = body.name {
        active.name = Set(name);
    }
    if let Some(script_type) = body.script_type {
        active.script_type = Set(script_type);
    }
    if let Some(hook_point) = body.hook_point {
        active.hook_point = Set(Some(hook_point));
    }
    if let Some(source_code) = body.source_code {
        active.source_code = Set(source_code);
    }
    if let Some(enabled) = body.enabled {
        active.enabled = Set(enabled);
    }
    if let Some(priority) = body.priority {
        active.priority = Set(priority);
    }
    if let Some(ref cron_expression) = body.cron_expression {
        SchedulerService::validate_cron(cron_expression)?;
        let next_run = SchedulerService::next_run_time(cron_expression)?;
        active.cron_expression = Set(Some(cron_expression.clone()));
        active.next_run_at = Set(Some(next_run));
    }
    active.updated_at = Set(chrono::Utc::now().fixed_offset());

    let result = active.update(&state.db).await?;
    Ok(Json(result))
}

#[utoipa::path(delete, path = "/api/v1/modules/{module_id}/scripts/{id}", tag = "Scripts",
    security(("bearer_auth" = [])),
    params(
        ("module_id" = Uuid, Path, description = "Module ID"),
        ("id" = Uuid, Path, description = "Script ID"),
    ),
    responses((status = 204, description = "Deleted"), (status = 404, description = "Not found"))
)]
pub(crate) async fn delete_script(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, AppError> {
    let result = script::Entity::delete_by_id(id).exec(&state.db).await?;
    if result.rows_affected == 0 {
        return Err(AppError::not_found(format!("script {id} not found")));
    }
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[utoipa::path(post, path = "/api/v1/modules/{module_id}/scripts/{id}/test", tag = "Scripts",
    security(("bearer_auth" = [])),
    params(
        ("module_id" = Uuid, Path, description = "Module ID"),
        ("id" = Uuid, Path, description = "Script ID"),
    ),
    request_body = TestScriptRequest,
    responses((status = 200, body = Object))
)]
pub(crate) async fn test_script(
    State(state): State<AppState>,
    Path((module_id, id)): Path<(Uuid, Uuid)>,
    Json(body): Json<TestScriptRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let s = script::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found(format!("script {id} not found")))?;

    let world = load_world(&state.db, module_id).await?;

    let result = match s.script_type.as_str() {
        "trigger" => {
            let obj = body.object.ok_or_else(|| {
                AppError::bad_request("object is required for trigger test".to_owned())
            })?;
            let hook = body.hook_point.or(s.hook_point.clone()).ok_or_else(|| {
                AppError::bad_request("hook_point is required for trigger test".to_owned())
            })?;
            let ctx = TriggerContext {
                hook_point: hook,
                object: obj,
            };
            let r = ScriptEngine::run_trigger(&s.source_code, &world, &ctx)?;
            serde_json::json!({
                "script_type": "trigger",
                "rejected": r.rejected,
                "reason": r.reason,
                "mutations": serde_json::to_value(&r.mutations).unwrap_or_default(),
            })
        }
        "layout" => {
            let obj = body.object.ok_or_else(|| {
                AppError::bad_request("object is required for layout test".to_owned())
            })?;
            let r = ScriptEngine::run_layout(&s.source_code, &world, &obj)?;
            serde_json::json!({
                "script_type": "layout",
                "value": r.value,
            })
        }
        "action" => {
            let r = ScriptEngine::run_action(&s.source_code, &world)?;
            serde_json::json!({
                "script_type": "action",
                "output": r.output,
                "mutations": serde_json::to_value(&r.mutations).unwrap_or_default(),
            })
        }
        other => {
            return Err(AppError::bad_request(format!(
                "unknown script_type '{other}'"
            )));
        }
    };

    Ok(Json(result))
}

#[utoipa::path(post, path = "/api/v1/modules/{module_id}/scripts/{id}/execute", tag = "Scripts",
    security(("bearer_auth" = [])),
    params(
        ("module_id" = Uuid, Path, description = "Module ID"),
        ("id" = Uuid, Path, description = "Script ID"),
    ),
    responses((status = 200, body = ExecuteResult))
)]
pub(crate) async fn execute_script(
    State(state): State<AppState>,
    Path((module_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ExecuteResult>, AppError> {
    let s = script::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found(format!("script {id} not found")))?;

    if s.script_type != "action" {
        return Err(AppError::bad_request(
            "only action scripts can be executed".to_owned(),
        ));
    }

    let world = load_world(&state.db, module_id).await?;
    let result = ScriptEngine::run_action(&s.source_code, &world)?;

    let mutation_count = result.mutations.len();

    if !result.mutations.is_empty() {
        let txn = state.db.begin().await?;
        apply_action_mutations(&txn, &result.mutations).await?;
        txn.commit().await?;
    }

    Ok(Json(ExecuteResult {
        output: result.output,
        mutations_applied: mutation_count,
    }))
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct LayoutEntry {
    object_id: Uuid,
    value: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct BatchLayoutResponse {
    results: Vec<LayoutEntry>,
}

#[utoipa::path(post, path = "/api/v1/modules/{module_id}/scripts/{id}/layout", tag = "Scripts",
    security(("bearer_auth" = [])),
    params(
        ("module_id" = Uuid, Path, description = "Module ID"),
        ("id" = Uuid, Path, description = "Script ID"),
    ),
    responses((status = 200, body = BatchLayoutResponse))
)]
pub(crate) async fn batch_layout(
    State(state): State<AppState>,
    Path((module_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Json<BatchLayoutResponse>, AppError> {
    let s = script::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found(format!("script {id} not found")))?;

    if s.script_type != "layout" {
        return Err(AppError::bad_request(
            "only layout scripts can be used with this endpoint".to_owned(),
        ));
    }

    let world = load_world(&state.db, module_id).await?;

    let mut results = Vec::new();
    for obj in &world.objects {
        let layout_result = ScriptEngine::run_layout(&s.source_code, &world, obj)?;
        let oid = obj
            .id
            .parse::<Uuid>()
            .map_err(|e| AppError::internal(format!("invalid object UUID: {e}")))?;
        results.push(LayoutEntry {
            object_id: oid,
            value: layout_result.value,
        });
    }

    Ok(Json(BatchLayoutResponse { results }))
}

#[utoipa::path(get, path = "/api/v1/modules/{module_id}/scripts/{id}/executions", tag = "Scripts",
    security(("bearer_auth" = [])),
    params(
        ("module_id" = Uuid, Path, description = "Module ID"),
        ("id" = Uuid, Path, description = "Script ID"),
        Pagination,
    ),
    responses((status = 200, body = PaginatedResponse<entity::script_execution::Model>))
)]
pub(crate) async fn list_executions(
    State(state): State<AppState>,
    Path((_module_id, id)): Path<(Uuid, Uuid)>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<PaginatedResponse<entity::script_execution::Model>>, AppError> {
    let result =
        SchedulerService::list_executions(&state.db, id, pagination.offset, pagination.limit)
            .await?;
    Ok(Json(result))
}

async fn apply_action_mutations(
    db: &impl ConnectionTrait,
    mutations: &[Mutation],
) -> Result<(), AppError> {
    use std::collections::HashMap;

    let mut grouped: HashMap<Uuid, Vec<(&String, &serde_json::Value)>> = HashMap::new();
    for m in mutations {
        match m {
            Mutation::SetAttribute {
                object_id,
                key,
                value,
            } => {
                grouped.entry(*object_id).or_default().push((key, value));
            }
        }
    }

    for (oid, changes) in &grouped {
        let obj = object::Entity::find_by_id(*oid)
            .one(db)
            .await?
            .ok_or_else(|| AppError::not_found(format!("object {oid} not found")))?;

        let mut attrs = obj
            .attributes
            .clone()
            .unwrap_or(serde_json::Value::Object(serde_json::Map::default()));
        for (key, value) in changes {
            if let Some(map) = attrs.as_object_mut() {
                let _ = map.insert((*key).clone(), (*value).clone());
            }
        }

        let mut active: object::ActiveModel = obj.into();
        active.attributes = Set(Some(attrs));
        active.updated_at = Set(chrono::Utc::now().fixed_offset());
        let _ = active.update(db).await?;
    }

    Ok(())
}
