use std::str::FromStr;

use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, Order, PaginatorTrait,
    QueryFilter, QueryOrder, Set,
};
use uuid::Uuid;

use entity::{script, script_execution};

use crate::PaginatedResponse;
use crate::error::CoreError;

pub struct SchedulerService;

impl SchedulerService {
    pub fn validate_cron(expression: &str) -> Result<(), CoreError> {
        let _ = cron::Schedule::from_str(expression)
            .map_err(|e| CoreError::bad_request(format!("invalid cron expression: {e}")))?;
        Ok(())
    }

    pub fn next_run_time(
        expression: &str,
    ) -> Result<chrono::DateTime<chrono::FixedOffset>, CoreError> {
        let schedule = cron::Schedule::from_str(expression)
            .map_err(|e| CoreError::bad_request(format!("invalid cron expression: {e}")))?;

        let next = schedule
            .upcoming(chrono::Utc)
            .next()
            .ok_or_else(|| CoreError::internal("no upcoming schedule time".to_string()))?;

        Ok(next.fixed_offset())
    }

    pub async fn find_due_scripts(
        db: &impl ConnectionTrait,
    ) -> Result<Vec<script::Model>, CoreError> {
        let now = chrono::Utc::now().fixed_offset();

        let scripts = script::Entity::find()
            .filter(script::Column::Enabled.eq(true))
            .filter(script::Column::ScriptType.eq("action"))
            .filter(script::Column::CronExpression.is_not_null())
            .filter(script::Column::NextRunAt.lte(now))
            .all(db)
            .await?;

        Ok(scripts)
    }

    pub async fn record_execution_start(
        db: &impl ConnectionTrait,
        script_id: Uuid,
    ) -> Result<script_execution::Model, CoreError> {
        let now = chrono::Utc::now().fixed_offset();
        let id = Uuid::now_v7();

        let model = script_execution::ActiveModel {
            id: Set(id),
            script_id: Set(script_id),
            status: Set("running".to_string()),
            started_at: Set(now),
            finished_at: Set(None),
            duration_ms: Set(None),
            output: Set(None),
            error_message: Set(None),
            created_at: Set(now),
        };

        let result = model.insert(db).await?;
        Ok(result)
    }

    pub async fn record_execution_finish(
        db: &impl ConnectionTrait,
        execution: script_execution::Model,
        status: &str,
        output: Option<String>,
        error_message: Option<String>,
    ) -> Result<script_execution::Model, CoreError> {
        let now = chrono::Utc::now().fixed_offset();
        let duration = (now - execution.started_at).num_milliseconds();

        let mut active: script_execution::ActiveModel = execution.into();
        active.status = Set(status.to_string());
        active.finished_at = Set(Some(now));
        active.duration_ms = Set(Some(duration));
        active.output = Set(output);
        active.error_message = Set(error_message);

        let result = active.update(db).await?;
        Ok(result)
    }

    pub async fn update_script_run_times(
        db: &impl ConnectionTrait,
        script_model: script::Model,
    ) -> Result<(), CoreError> {
        let now = chrono::Utc::now().fixed_offset();
        let next = script_model
            .cron_expression
            .as_deref()
            .and_then(|expr| Self::next_run_time(expr).ok());

        let mut active: script::ActiveModel = script_model.into();
        active.last_run_at = Set(Some(now));
        active.next_run_at = Set(next);
        active.updated_at = Set(now);
        let _ = active.update(db).await?;
        Ok(())
    }

    pub async fn list_executions(
        db: &impl ConnectionTrait,
        script_id: Uuid,
        offset: u64,
        limit: u64,
    ) -> Result<PaginatedResponse<script_execution::Model>, CoreError> {
        let select = script_execution::Entity::find()
            .filter(script_execution::Column::ScriptId.eq(script_id))
            .order_by(script_execution::Column::StartedAt, Order::Desc);

        let paginator = select.paginate(db, limit);
        let total = paginator.num_items().await?;
        let page = offset.checked_div(limit).unwrap_or(0);
        let items = paginator.fetch_page(page).await?;

        Ok(PaginatedResponse {
            items,
            total,
            offset,
            limit,
        })
    }
}
