#![allow(unused_qualifications)]

use std::io::Cursor;

use axum::{
    Router,
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{error::AppError, state::AppState};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/projects/{project_id}/reqif/import",
            post(import_reqif_handler),
        )
        .route(
            "/modules/{module_id}/reqif/export",
            get(export_reqif_handler),
        )
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ImportResponse {
    module_id: Uuid,
    objects_created: usize,
    links_created: usize,
    attribute_definitions_created: usize,
    object_types_created: usize,
    link_types_created: usize,
}

#[utoipa::path(post, path = "/api/v1/projects/{project_id}/reqif/import", tag = "ReqIF",
    security(("bearer_auth" = [])),
    params(("project_id" = Uuid, Path, description = "Project ID")),
    request_body(content_type = "multipart/form-data", content = String),
    responses((status = 201, body = ImportResponse))
)]
pub(crate) async fn import_reqif_handler(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<Response, AppError> {
    let field = multipart
        .next_field()
        .await
        .map_err(|e| AppError::bad_request(format!("multipart error: {e}")))?
        .ok_or_else(|| AppError::bad_request("no file field in request".to_owned()))?;

    let filename = field.file_name().unwrap_or("upload.reqif").to_owned();
    let data = field
        .bytes()
        .await
        .map_err(|e| AppError::bad_request(format!("failed to read file: {e}")))?;

    let is_reqifz = filename.ends_with(".reqifz");

    let doc = if is_reqifz {
        let cursor = Cursor::new(&data);
        req1_reqif::from_reqifz(cursor)
            .map_err(|e| AppError::bad_request(format!("invalid reqifz: {e}")))?
    } else {
        let xml = std::str::from_utf8(&data)
            .map_err(|e| AppError::bad_request(format!("invalid UTF-8: {e}")))?;
        req1_reqif::from_xml_str(xml)
            .map_err(|e| AppError::bad_request(format!("invalid reqif XML: {e}")))?
    };

    let result = req1_core::reqif::import::import_reqif(&state.db, project_id, &doc).await?;

    let response = ImportResponse {
        module_id: result.module_id,
        objects_created: result.objects_created,
        links_created: result.links_created,
        attribute_definitions_created: result.attribute_definitions_created,
        object_types_created: result.object_types_created,
        link_types_created: result.link_types_created,
    };

    Ok((StatusCode::CREATED, axum::Json(response)).into_response())
}

#[derive(Debug, Deserialize, IntoParams)]
pub(crate) struct ExportQuery {
    #[serde(default = "default_export_format")]
    format: String,
}

fn default_export_format() -> String {
    "reqif".to_owned()
}

#[utoipa::path(get, path = "/api/v1/modules/{module_id}/reqif/export", tag = "ReqIF",
    security(("bearer_auth" = [])),
    params(
        ("module_id" = Uuid, Path, description = "Module ID"),
        ExportQuery,
    ),
    responses(
        (status = 200, content_type = "application/octet-stream", body = Vec<u8>),
    )
)]
pub(crate) async fn export_reqif_handler(
    State(state): State<AppState>,
    Path(module_id): Path<Uuid>,
    Query(query): Query<ExportQuery>,
) -> Result<Response, AppError> {
    let result = req1_core::reqif::export::export_reqif(&state.db, module_id).await?;

    match query.format.as_str() {
        "reqif" => {
            let xml = req1_reqif::to_xml_string(&result.document)
                .map_err(|e| AppError::internal(format!("XML serialization failed: {e}")))?;

            let mut headers = HeaderMap::new();
            let _ = headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/xml"),
            );
            let _ = headers.insert(
                header::CONTENT_DISPOSITION,
                HeaderValue::from_str(&format!("attachment; filename=\"{module_id}.reqif\""))
                    .unwrap_or_else(|_| {
                        HeaderValue::from_static("attachment; filename=\"export.reqif\"")
                    }),
            );

            Ok((headers, xml).into_response())
        }
        "reqifz" => {
            let mut cursor = Cursor::new(Vec::new());
            let filename = format!("{module_id}.reqif");
            req1_reqif::to_reqifz(&mut cursor, &result.document, &filename)
                .map_err(|e| AppError::internal(format!("reqifz serialization failed: {e}")))?;

            let bytes = cursor.into_inner();
            let mut headers = HeaderMap::new();
            let _ = headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/zip"),
            );
            let _ = headers.insert(
                header::CONTENT_DISPOSITION,
                HeaderValue::from_str(&format!("attachment; filename=\"{module_id}.reqifz\""))
                    .unwrap_or_else(|_| {
                        HeaderValue::from_static("attachment; filename=\"export.reqifz\"")
                    }),
            );

            Ok((headers, Body::from(bytes)).into_response())
        }
        other => Err(AppError::bad_request(format!(
            "unsupported format '{other}', supported: reqif, reqifz"
        ))),
    }
}
