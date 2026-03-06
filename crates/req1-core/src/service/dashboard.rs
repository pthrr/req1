use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, Order, QueryFilter, QueryOrder,
    Set,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use entity::{dashboard, dashboard_widget};

use crate::error::CoreError;

const VALID_WIDGET_TYPES: &[&str] = &[
    "coverage_chart",
    "suspect_link_count",
    "lifecycle_distribution",
    "test_status",
];

#[derive(Debug, Deserialize)]
pub struct CreateDashboardInput {
    #[serde(default)]
    pub workspace_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_by: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDashboardInput {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateWidgetInput {
    #[serde(default)]
    pub dashboard_id: Uuid,
    pub widget_type: String,
    pub title: String,
    pub config: Option<serde_json::Value>,
    pub position_x: Option<i32>,
    pub position_y: Option<i32>,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWidgetInput {
    pub widget_type: Option<String>,
    pub title: Option<String>,
    pub config: Option<serde_json::Value>,
    pub position_x: Option<i32>,
    pub position_y: Option<i32>,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct WidgetDataEntry {
    pub label: String,
    pub value: i64,
    pub extra: Option<serde_json::Value>,
}

pub struct DashboardService;

impl DashboardService {
    // --- Dashboard CRUD ---

    pub async fn list_dashboards(
        db: &impl ConnectionTrait,
        workspace_id: Uuid,
    ) -> Result<Vec<dashboard::Model>, CoreError> {
        let items = dashboard::Entity::find()
            .filter(dashboard::Column::WorkspaceId.eq(workspace_id))
            .order_by(dashboard::Column::CreatedAt, Order::Desc)
            .all(db)
            .await?;
        Ok(items)
    }

    pub async fn create_dashboard(
        db: &impl ConnectionTrait,
        input: CreateDashboardInput,
    ) -> Result<dashboard::Model, CoreError> {
        let now = chrono::Utc::now().fixed_offset();
        let id = Uuid::now_v7();

        let model = dashboard::ActiveModel {
            id: Set(id),
            workspace_id: Set(input.workspace_id),
            name: Set(input.name),
            description: Set(input.description),
            layout: Set(serde_json::json!([])),
            created_by: Set(input.created_by),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let result = model.insert(db).await?;
        Ok(result)
    }

    pub async fn get_dashboard(
        db: &impl ConnectionTrait,
        id: Uuid,
    ) -> Result<dashboard::Model, CoreError> {
        dashboard::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("dashboard {id} not found")))
    }

    pub async fn update_dashboard(
        db: &impl ConnectionTrait,
        id: Uuid,
        input: UpdateDashboardInput,
    ) -> Result<dashboard::Model, CoreError> {
        let existing = Self::get_dashboard(db, id).await?;
        let mut active: dashboard::ActiveModel = existing.into();

        if let Some(name) = input.name {
            active.name = Set(name);
        }
        if let Some(description) = input.description {
            active.description = Set(Some(description));
        }
        active.updated_at = Set(chrono::Utc::now().fixed_offset());

        let result = active.update(db).await?;
        Ok(result)
    }

    pub async fn delete_dashboard(
        db: &impl ConnectionTrait,
        id: Uuid,
    ) -> Result<(), CoreError> {
        let result = dashboard::Entity::delete_by_id(id).exec(db).await?;
        if result.rows_affected == 0 {
            return Err(CoreError::NotFound(format!("dashboard {id} not found")));
        }
        Ok(())
    }

    // --- Widget CRUD ---

    pub async fn list_widgets(
        db: &impl ConnectionTrait,
        dashboard_id: Uuid,
    ) -> Result<Vec<dashboard_widget::Model>, CoreError> {
        let items = dashboard_widget::Entity::find()
            .filter(dashboard_widget::Column::DashboardId.eq(dashboard_id))
            .order_by(dashboard_widget::Column::PositionY, Order::Asc)
            .order_by(dashboard_widget::Column::PositionX, Order::Asc)
            .all(db)
            .await?;
        Ok(items)
    }

    pub async fn create_widget(
        db: &impl ConnectionTrait,
        input: CreateWidgetInput,
    ) -> Result<dashboard_widget::Model, CoreError> {
        if !VALID_WIDGET_TYPES.contains(&input.widget_type.as_str()) {
            return Err(CoreError::BadRequest(format!(
                "invalid widget_type '{}', must be one of: {VALID_WIDGET_TYPES:?}",
                input.widget_type
            )));
        }

        let now = chrono::Utc::now().fixed_offset();
        let id = Uuid::now_v7();

        let model = dashboard_widget::ActiveModel {
            id: Set(id),
            dashboard_id: Set(input.dashboard_id),
            widget_type: Set(input.widget_type),
            title: Set(input.title),
            config: Set(input.config.unwrap_or(serde_json::json!({}))),
            position_x: Set(input.position_x.unwrap_or(0)),
            position_y: Set(input.position_y.unwrap_or(0)),
            width: Set(input.width.unwrap_or(4)),
            height: Set(input.height.unwrap_or(3)),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let result = model.insert(db).await?;
        Ok(result)
    }

    pub async fn get_widget(
        db: &impl ConnectionTrait,
        id: Uuid,
    ) -> Result<dashboard_widget::Model, CoreError> {
        dashboard_widget::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("widget {id} not found")))
    }

    pub async fn update_widget(
        db: &impl ConnectionTrait,
        id: Uuid,
        input: UpdateWidgetInput,
    ) -> Result<dashboard_widget::Model, CoreError> {
        let existing = Self::get_widget(db, id).await?;

        if let Some(ref wt) = input.widget_type
            && !VALID_WIDGET_TYPES.contains(&wt.as_str())
        {
            return Err(CoreError::BadRequest(format!(
                "invalid widget_type '{wt}', must be one of: {VALID_WIDGET_TYPES:?}"
            )));
        }

        let mut active: dashboard_widget::ActiveModel = existing.into();
        if let Some(widget_type) = input.widget_type {
            active.widget_type = Set(widget_type);
        }
        if let Some(title) = input.title {
            active.title = Set(title);
        }
        if let Some(config) = input.config {
            active.config = Set(config);
        }
        if let Some(px) = input.position_x {
            active.position_x = Set(px);
        }
        if let Some(py) = input.position_y {
            active.position_y = Set(py);
        }
        if let Some(w) = input.width {
            active.width = Set(w);
        }
        if let Some(h) = input.height {
            active.height = Set(h);
        }
        active.updated_at = Set(chrono::Utc::now().fixed_offset());

        let result = active.update(db).await?;
        Ok(result)
    }

    pub async fn delete_widget(
        db: &impl ConnectionTrait,
        id: Uuid,
    ) -> Result<(), CoreError> {
        let result = dashboard_widget::Entity::delete_by_id(id).exec(db).await?;
        if result.rows_affected == 0 {
            return Err(CoreError::NotFound(format!("widget {id} not found")));
        }
        Ok(())
    }

    // --- Widget Data ---

    pub async fn get_widget_data(
        db: &impl ConnectionTrait,
        widget_id: Uuid,
    ) -> Result<Vec<WidgetDataEntry>, CoreError> {
        let widget = Self::get_widget(db, widget_id).await?;

        let module_ids = extract_module_ids(&widget.config, db).await?;

        match widget.widget_type.as_str() {
            "coverage_chart" => coverage_data(db, &module_ids).await,
            "suspect_link_count" => suspect_link_data(db, &module_ids).await,
            "lifecycle_distribution" => lifecycle_data(db, &module_ids).await,
            "test_status" => test_status_data(db, &module_ids).await,
            other => Err(CoreError::BadRequest(format!(
                "unknown widget_type: {other}"
            ))),
        }
    }
}

async fn extract_module_ids(
    config: &serde_json::Value,
    db: &impl ConnectionTrait,
) -> Result<Vec<Uuid>, CoreError> {
    // Direct module_ids from config
    if let Some(ids) = config.get("module_ids").and_then(|v| v.as_array()) {
        let module_ids: Vec<Uuid> = ids
            .iter()
            .filter_map(|v| v.as_str().and_then(|s| s.parse().ok()))
            .collect();
        if !module_ids.is_empty() {
            return Ok(module_ids);
        }
    }

    // Resolve from project_ids
    if let Some(pids) = config.get("project_ids").and_then(|v| v.as_array()) {
        let project_ids: Vec<Uuid> = pids
            .iter()
            .filter_map(|v| v.as_str().and_then(|s| s.parse().ok()))
            .collect();
        if !project_ids.is_empty() {
            let modules = entity::module::Entity::find()
                .filter(entity::module::Column::ProjectId.is_in(project_ids))
                .all(db)
                .await?;
            return Ok(modules.iter().map(|m| m.id).collect());
        }
    }

    // No filter: all modules
    let modules = entity::module::Entity::find().all(db).await?;
    Ok(modules.iter().map(|m| m.id).collect())
}

#[allow(clippy::cast_possible_wrap)]
async fn coverage_data(
    db: &impl ConnectionTrait,
    module_ids: &[Uuid],
) -> Result<Vec<WidgetDataEntry>, CoreError> {
    let mut entries = Vec::new();
    for &mid in module_ids {
        let module = entity::module::Entity::find_by_id(mid).one(db).await?;
        let module_name = module.map_or_else(|| mid.to_string(), |m| m.name);

        let objects = entity::object::Entity::find()
            .filter(entity::object::Column::ModuleId.eq(mid))
            .filter(entity::object::Column::DeletedAt.is_null())
            .all(db)
            .await?;

        let total = objects.len() as i64;
        let obj_ids: Vec<Uuid> = objects.iter().map(|o| o.id).collect();

        let upstream = if obj_ids.is_empty() {
            0i64
        } else {
            entity::link::Entity::find()
                .filter(entity::link::Column::TargetObjectId.is_in(obj_ids.clone()))
                .all(db)
                .await?
                .iter()
                .map(|l| l.target_object_id)
                .collect::<std::collections::HashSet<_>>()
                .len() as i64
        };

        let downstream = if obj_ids.is_empty() {
            0i64
        } else {
            entity::link::Entity::find()
                .filter(entity::link::Column::SourceObjectId.is_in(obj_ids))
                .all(db)
                .await?
                .iter()
                .map(|l| l.source_object_id)
                .collect::<std::collections::HashSet<_>>()
                .len() as i64
        };

        entries.push(WidgetDataEntry {
            label: module_name,
            value: total,
            extra: Some(serde_json::json!({
                "with_upstream": upstream,
                "with_downstream": downstream,
            })),
        });
    }
    Ok(entries)
}

#[allow(clippy::cast_possible_wrap)]
async fn suspect_link_data(
    db: &impl ConnectionTrait,
    module_ids: &[Uuid],
) -> Result<Vec<WidgetDataEntry>, CoreError> {
    let mut entries = Vec::new();
    for &mid in module_ids {
        let module = entity::module::Entity::find_by_id(mid).one(db).await?;
        let module_name = module.map_or_else(|| mid.to_string(), |m| m.name);

        let objects = entity::object::Entity::find()
            .filter(entity::object::Column::ModuleId.eq(mid))
            .filter(entity::object::Column::DeletedAt.is_null())
            .all(db)
            .await?;

        let obj_ids: Vec<Uuid> = objects.iter().map(|o| o.id).collect();

        let count = if obj_ids.is_empty() {
            0i64
        } else {
            entity::link::Entity::find()
                .filter(entity::link::Column::SourceObjectId.is_in(obj_ids.clone()))
                .filter(entity::link::Column::Suspect.eq(true))
                .all(db)
                .await?
                .len() as i64
                + entity::link::Entity::find()
                    .filter(entity::link::Column::TargetObjectId.is_in(obj_ids))
                    .filter(entity::link::Column::Suspect.eq(true))
                    .all(db)
                    .await?
                    .len() as i64
        };

        entries.push(WidgetDataEntry {
            label: module_name,
            value: count,
            extra: None,
        });
    }
    Ok(entries)
}

async fn lifecycle_data(
    db: &impl ConnectionTrait,
    module_ids: &[Uuid],
) -> Result<Vec<WidgetDataEntry>, CoreError> {
    let objects = entity::object::Entity::find()
        .filter(entity::object::Column::ModuleId.is_in(module_ids.to_vec()))
        .filter(entity::object::Column::DeletedAt.is_null())
        .all(db)
        .await?;

    let mut state_counts: std::collections::HashMap<String, i64> =
        std::collections::HashMap::new();
    for obj in &objects {
        let state = obj
            .lifecycle_state
            .as_deref()
            .unwrap_or("(none)")
            .to_string();
        *state_counts.entry(state).or_insert(0) += 1;
    }

    let entries = state_counts
        .into_iter()
        .map(|(label, value)| WidgetDataEntry {
            label,
            value,
            extra: None,
        })
        .collect();

    Ok(entries)
}

async fn test_status_data(
    db: &impl ConnectionTrait,
    module_ids: &[Uuid],
) -> Result<Vec<WidgetDataEntry>, CoreError> {
    let test_cases = entity::test_case::Entity::find()
        .filter(entity::test_case::Column::ModuleId.is_in(module_ids.to_vec()))
        .all(db)
        .await?;

    let mut status_counts: std::collections::HashMap<String, i64> =
        std::collections::HashMap::new();

    for tc in &test_cases {
        let executions = entity::test_execution::Entity::find()
            .filter(entity::test_execution::Column::TestCaseId.eq(tc.id))
            .order_by(entity::test_execution::Column::CreatedAt, Order::Desc)
            .all(db)
            .await?;

        let status = executions
            .first()
            .map_or_else(|| "not_run".to_string(), |e| e.status.clone());

        *status_counts.entry(status).or_insert(0) += 1;
    }

    let entries = status_counts
        .into_iter()
        .map(|(label, value)| WidgetDataEntry {
            label,
            value,
            extra: None,
        })
        .collect();

    Ok(entries)
}
