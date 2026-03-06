use axum::{
    Router,
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{error::AppError, state::AppState};
use req1_core::service::csv_import::CsvImportService;
use req1_core::service::docx_import::{DocxImportInput, DocxImportService};
use req1_core::service::publish::PublishService;
use req1_core::service::xlsx_import::XlsxImportService;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/modules/{module_id}/publish", get(publish_module))
        .route("/modules/{module_id}/import/csv", post(import_csv))
        .route("/modules/{module_id}/import/xlsx", post(import_xlsx))
        .route(
            "/modules/{module_id}/import/docx/preview",
            post(preview_docx),
        )
        .route("/modules/{module_id}/import/docx", post(import_docx))
}

#[derive(Debug, Deserialize)]
struct PublishQuery {
    #[serde(default = "default_format")]
    format: String,
}

fn default_format() -> String {
    "html".to_owned()
}

fn content_type_header(ct: &'static str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let _ = headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(ct));
    headers
}

async fn publish_module(
    State(state): State<AppState>,
    Path(module_id): Path<Uuid>,
    Query(query): Query<PublishQuery>,
) -> Result<Response, AppError> {
    match query.format.as_str() {
        "html" => {
            let html = PublishService::render_html(&state.db, module_id).await?;
            Ok(Html(html).into_response())
        }
        "md" | "markdown" => {
            let md = PublishService::render_markdown(&state.db, module_id).await?;
            let headers = content_type_header("text/markdown; charset=utf-8");
            Ok((headers, md).into_response())
        }
        "latex" | "tex" => {
            let tex = PublishService::render_latex(&state.db, module_id).await?;
            let mut headers = content_type_header("text/plain; charset=utf-8");
            let _ = headers.insert(
                header::CONTENT_DISPOSITION,
                HeaderValue::from_static("attachment; filename=\"document.tex\""),
            );
            Ok((headers, tex).into_response())
        }
        "txt" | "text" => {
            let txt = PublishService::render_text(&state.db, module_id).await?;
            let headers = content_type_header("text/plain; charset=utf-8");
            Ok((headers, txt).into_response())
        }
        "csv" => {
            let csv_str = PublishService::render_csv(&state.db, module_id).await?;
            let mut headers = content_type_header("text/csv; charset=utf-8");
            let _ = headers.insert(
                header::CONTENT_DISPOSITION,
                HeaderValue::from_static("attachment; filename=\"objects.csv\""),
            );
            Ok((headers, Body::from(csv_str)).into_response())
        }
        "yaml" => {
            let yaml_str = PublishService::render_yaml(&state.db, module_id).await?;
            let headers = content_type_header("text/yaml; charset=utf-8");
            Ok((headers, yaml_str).into_response())
        }
        "pdf" => {
            let pdf_bytes = PublishService::render_pdf(&state.db, module_id).await?;
            let mut headers = content_type_header("application/pdf");
            let _ = headers.insert(
                header::CONTENT_DISPOSITION,
                HeaderValue::from_static("inline; filename=\"document.pdf\""),
            );
            Ok((headers, Body::from(pdf_bytes)).into_response())
        }
        "docx" | "word" => {
            let docx_bytes = PublishService::render_docx(&state.db, module_id).await?;
            let mut headers = content_type_header(
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            );
            let _ = headers.insert(
                header::CONTENT_DISPOSITION,
                HeaderValue::from_static("attachment; filename=\"document.docx\""),
            );
            Ok((headers, Body::from(docx_bytes)).into_response())
        }
        "xlsx" | "excel" => {
            let xlsx_bytes = PublishService::render_xlsx(&state.db, module_id).await?;
            let mut headers = content_type_header(
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            );
            let _ = headers.insert(
                header::CONTENT_DISPOSITION,
                HeaderValue::from_static("attachment; filename=\"objects.xlsx\""),
            );
            Ok((headers, Body::from(xlsx_bytes)).into_response())
        }
        other => Err(AppError::BadRequest(format!(
            "unsupported format '{other}', supported: html, md, latex, txt, csv, yaml, pdf, xlsx, docx"
        ))),
    }
}

#[derive(Debug, serde::Serialize)]
struct CsvImportResponse {
    objects_created: usize,
}

async fn import_csv(
    State(state): State<AppState>,
    Path(module_id): Path<Uuid>,
    body: String,
) -> Result<axum::Json<CsvImportResponse>, AppError> {
    let result = CsvImportService::import_csv(&state.db, module_id, &body).await?;
    Ok(axum::Json(CsvImportResponse {
        objects_created: result.objects_created,
    }))
}

#[derive(Debug, serde::Serialize)]
struct XlsxImportResponse {
    objects_created: usize,
    objects_updated: usize,
}

async fn import_xlsx(
    State(state): State<AppState>,
    Path(module_id): Path<Uuid>,
    body: axum::body::Bytes,
) -> Result<axum::Json<XlsxImportResponse>, AppError> {
    let result = XlsxImportService::import_xlsx(&state.db, module_id, &body).await?;
    Ok(axum::Json(XlsxImportResponse {
        objects_created: result.objects_created,
        objects_updated: result.objects_updated,
    }))
}

async fn preview_docx(
    State(state): State<AppState>,
    Path(module_id): Path<Uuid>,
    body: axum::body::Bytes,
) -> Result<axum::Json<req1_core::service::docx_import::DocxPreviewResult>, AppError> {
    let result = DocxImportService::preview_docx(&state.db, module_id, &body).await?;
    Ok(axum::Json(result))
}

async fn import_docx(
    State(state): State<AppState>,
    Path(module_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<(StatusCode, axum::Json<req1_core::service::docx_import::DocxImportResult>), AppError> {
    let mut file_data: Option<Vec<u8>> = None;
    let mut mapping_json: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("multipart error: {e}")))?
    {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" => {
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::BadRequest(format!("file read error: {e}")))?;
                file_data = Some(bytes.to_vec());
            }
            "mapping" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| AppError::BadRequest(format!("mapping read error: {e}")))?;
                mapping_json = Some(text);
            }
            _ => {}
        }
    }

    let data = file_data.ok_or_else(|| AppError::BadRequest("missing 'file' field".to_owned()))?;
    let input: DocxImportInput = if let Some(json) = mapping_json {
        serde_json::from_str(&json)
            .map_err(|e| AppError::BadRequest(format!("invalid mapping JSON: {e}")))?
    } else {
        DocxImportInput {
            style_mappings: Vec::new(),
        }
    };

    let result = DocxImportService::import_docx(&state.db, module_id, &data, input).await?;
    Ok((StatusCode::CREATED, axum::Json(result)))
}
