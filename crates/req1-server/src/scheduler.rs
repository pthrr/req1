use sea_orm::DatabaseConnection;
use std::time::Duration;
use tokio::time;

use req1_core::scripting::engine::ScriptEngine;
use req1_core::service::object::load_world;
use req1_core::service::scheduler::SchedulerService;

pub fn spawn_scheduler(db: DatabaseConnection) {
    drop(tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_mins(1));

        loop {
            let _ = interval.tick().await;
            if let Err(e) = run_due_scripts(&db).await {
                tracing::error!("Scheduler error: {e}");
            }
        }
    }));
}

async fn run_due_scripts(db: &DatabaseConnection) -> Result<(), Box<dyn std::error::Error>> {
    let due_scripts = SchedulerService::find_due_scripts(db).await?;

    for script in due_scripts {
        let script_id = script.id;
        let module_id = script.module_id;
        let source = script.source_code.clone();

        tracing::info!("Executing scheduled script {} ({})", script.name, script_id);

        let execution = SchedulerService::record_execution_start(db, script_id).await?;

        let world = match load_world(db, module_id).await {
            Ok(w) => w,
            Err(e) => {
                let _ = SchedulerService::record_execution_finish(
                    db,
                    execution,
                    "error",
                    None,
                    Some(format!("failed to load world: {e}")),
                )
                .await;
                let _ = SchedulerService::update_script_run_times(db, script).await;
                continue;
            }
        };

        match ScriptEngine::run_action(&source, &world) {
            Ok(result) => {
                let output_text = if result.output.is_empty() {
                    None
                } else {
                    Some(result.output.join("\n"))
                };

                let _ = SchedulerService::record_execution_finish(
                    db, execution, "success", output_text, None,
                )
                .await;
            }
            Err(e) => {
                let _ = SchedulerService::record_execution_finish(
                    db,
                    execution,
                    "error",
                    None,
                    Some(e.to_string()),
                )
                .await;
            }
        }

        let _ = SchedulerService::update_script_run_times(db, script).await;
    }

    Ok(())
}
