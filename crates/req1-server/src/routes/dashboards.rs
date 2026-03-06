use axum::{
    Extension, Json, Router,
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use std::fmt::Write as _;
use uuid::Uuid;

use crate::{error::AppError, state::AppState};
use req1_core::auth::AuthUser;
use req1_core::service::dashboard::{
    CreateDashboardInput, CreateWidgetInput, DashboardService, UpdateDashboardInput,
    UpdateWidgetInput,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/workspaces/{workspace_id}/dashboards",
            get(list_dashboards).post(create_dashboard),
        )
        .route(
            "/workspaces/{workspace_id}/dashboards/{id}",
            get(get_dashboard)
                .patch(update_dashboard)
                .delete(delete_dashboard),
        )
        .route(
            "/dashboards/{dashboard_id}/widgets",
            get(list_widgets).post(create_widget),
        )
        .route(
            "/dashboards/{dashboard_id}/widgets/{id}",
            get(get_widget)
                .patch(update_widget)
                .delete(delete_widget),
        )
        .route(
            "/dashboards/{dashboard_id}/widgets/{id}/data",
            get(get_widget_data),
        )
        .route(
            "/dashboards/{dashboard_id}/export/pdf",
            get(export_pdf),
        )
}

async fn list_dashboards(
    State(state): State<AppState>,
    Path(workspace_id): Path<Uuid>,
) -> Result<Json<Vec<entity::dashboard::Model>>, AppError> {
    let items = DashboardService::list_dashboards(&state.db, workspace_id).await?;
    Ok(Json(items))
}

async fn create_dashboard(
    State(state): State<AppState>,
    Path(workspace_id): Path<Uuid>,
    Extension(auth_user): Extension<AuthUser>,
    Json(body): Json<CreateDashboardInput>,
) -> Result<(StatusCode, Json<entity::dashboard::Model>), AppError> {
    let input = CreateDashboardInput {
        workspace_id,
        created_by: Some(auth_user.id),
        ..body
    };
    let result = DashboardService::create_dashboard(&state.db, input).await?;
    Ok((StatusCode::CREATED, Json(result)))
}

async fn get_dashboard(
    State(state): State<AppState>,
    Path((_workspace_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Json<entity::dashboard::Model>, AppError> {
    let result = DashboardService::get_dashboard(&state.db, id).await?;
    Ok(Json(result))
}

async fn update_dashboard(
    State(state): State<AppState>,
    Path((_workspace_id, id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateDashboardInput>,
) -> Result<Json<entity::dashboard::Model>, AppError> {
    let result = DashboardService::update_dashboard(&state.db, id, body).await?;
    Ok(Json(result))
}

async fn delete_dashboard(
    State(state): State<AppState>,
    Path((_workspace_id, id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    DashboardService::delete_dashboard(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_widgets(
    State(state): State<AppState>,
    Path(dashboard_id): Path<Uuid>,
) -> Result<Json<Vec<entity::dashboard_widget::Model>>, AppError> {
    let items = DashboardService::list_widgets(&state.db, dashboard_id).await?;
    Ok(Json(items))
}

async fn create_widget(
    State(state): State<AppState>,
    Path(dashboard_id): Path<Uuid>,
    Json(body): Json<CreateWidgetInput>,
) -> Result<(StatusCode, Json<entity::dashboard_widget::Model>), AppError> {
    let input = CreateWidgetInput {
        dashboard_id,
        ..body
    };
    let result = DashboardService::create_widget(&state.db, input).await?;
    Ok((StatusCode::CREATED, Json(result)))
}

async fn get_widget(
    State(state): State<AppState>,
    Path((_dashboard_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Json<entity::dashboard_widget::Model>, AppError> {
    let result = DashboardService::get_widget(&state.db, id).await?;
    Ok(Json(result))
}

async fn update_widget(
    State(state): State<AppState>,
    Path((_dashboard_id, id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateWidgetInput>,
) -> Result<Json<entity::dashboard_widget::Model>, AppError> {
    let result = DashboardService::update_widget(&state.db, id, body).await?;
    Ok(Json(result))
}

async fn delete_widget(
    State(state): State<AppState>,
    Path((_dashboard_id, id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    DashboardService::delete_widget(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_widget_data(
    State(state): State<AppState>,
    Path((_dashboard_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<req1_core::service::dashboard::WidgetDataEntry>>, AppError> {
    let data = DashboardService::get_widget_data(&state.db, id).await?;
    Ok(Json(data))
}

async fn export_pdf(
    State(state): State<AppState>,
    Path(dashboard_id): Path<Uuid>,
) -> Result<Response, AppError> {
    let dashboard = DashboardService::get_dashboard(&state.db, dashboard_id).await?;
    let widgets = DashboardService::list_widgets(&state.db, dashboard_id).await?;

    // Build simple HTML for PDF export
    let mut html = format!(
        r#"<!DOCTYPE html><html><head><meta charset="UTF-8"><title>{}</title>
        <style>body {{ font-family: sans-serif; margin: 20px; }}
        .widget {{ border: 1px solid #ccc; padding: 16px; margin: 8px 0; border-radius: 4px; }}
        h1 {{ color: #1976d2; }}</style></head><body>"#,
        dashboard.name
    );
    let _ = write!(html, "<h1>{}</h1>", dashboard.name);
    if let Some(ref desc) = dashboard.description {
        let _ = write!(html, "<p>{desc}</p>");
    }

    for widget in &widgets {
        let _ = write!(
            html,
            r#"<div class="widget"><h3>{}</h3><p>Type: {}</p></div>"#,
            widget.title, widget.widget_type
        );
    }

    html.push_str("</body></html>");

    // Try rendering to PDF using same approach as PublishService
    let pdf_result = tokio::task::spawn_blocking(move || {
        try_wkhtmltopdf_raw(&html)
    })
    .await
    .map_err(|e| AppError::Internal(format!("pdf task error: {e}")))?;

    match pdf_result {
        Ok(pdf_bytes) => {
            let mut headers = HeaderMap::new();
            let _ = headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/pdf"),
            );
            let _ = headers.insert(
                header::CONTENT_DISPOSITION,
                HeaderValue::from_static("inline; filename=\"dashboard.pdf\""),
            );
            Ok((headers, Body::from(pdf_bytes)).into_response())
        }
        Err(_) => Err(AppError::Internal(
            "PDF generation failed. Install wkhtmltopdf.".to_owned(),
        )),
    }
}

fn try_wkhtmltopdf_raw(html: &str) -> Result<Vec<u8>, String> {
    use std::io::Write as _;
    let mut child = std::process::Command::new("wkhtmltopdf")
        .args(["--quiet", "-", "-"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("wkhtmltopdf: {e}"))?;

    if let Some(ref mut stdin) = child.stdin {
        stdin
            .write_all(html.as_bytes())
            .map_err(|e| format!("wkhtmltopdf stdin: {e}"))?;
    }
    drop(child.stdin.take());

    let output = child
        .wait_with_output()
        .map_err(|e| format!("wkhtmltopdf wait: {e}"))?;

    if output.status.success() || !output.stdout.is_empty() {
        Ok(output.stdout)
    } else {
        Err(format!(
            "wkhtmltopdf failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}
