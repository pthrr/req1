use axum::{
    Json, Router,
    body::Body,
    extract::{Multipart, Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{error::AppError, state::AppState};
use req1_core::service::attachment::AttachmentService;

const DEFAULT_UPLOAD_DIR: &str = "./uploads";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/objects/{object_id}/attachments",
            get(list_attachments).post(upload_attachment),
        )
        .route(
            "/objects/{object_id}/attachments/{id}",
            axum::routing::delete(delete_attachment),
        )
        .route(
            "/objects/{object_id}/attachments/{id}/download",
            get(download_attachment),
        )
        .route(
            "/objects/{object_id}/attachments/{id}/verify",
            get(verify_attachment),
        )
}

#[utoipa::path(get, path = "/api/v1/objects/{object_id}/attachments", tag = "Attachments",
    security(("bearer_auth" = [])),
    params(("object_id" = Uuid, Path, description = "Object ID")),
    responses((status = 200, body = Vec<entity::attachment::Model>))
)]
pub(crate) async fn list_attachments(
    State(state): State<AppState>,
    Path(object_id): Path<Uuid>,
) -> Result<Json<Vec<entity::attachment::Model>>, AppError> {
    let items = AttachmentService::list(&state.db, object_id).await?;
    Ok(Json(items))
}

#[utoipa::path(post, path = "/api/v1/objects/{object_id}/attachments", tag = "Attachments",
    security(("bearer_auth" = [])),
    params(("object_id" = Uuid, Path, description = "Object ID")),
    request_body(content_type = "multipart/form-data", content = String),
    responses((status = 201, body = entity::attachment::Model))
)]
pub(crate) async fn upload_attachment(
    State(state): State<AppState>,
    Path(object_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<entity::attachment::Model>), AppError> {
    let field = multipart
        .next_field()
        .await
        .map_err(|e| AppError::bad_request(format!("multipart error: {e}")))?
        .ok_or_else(|| AppError::bad_request("no file field in request".to_owned()))?;

    let file_name = field.file_name().unwrap_or("upload").to_owned();
    let content_type = field
        .content_type()
        .unwrap_or("application/octet-stream")
        .to_owned();
    let data = field
        .bytes()
        .await
        .map_err(|e| AppError::bad_request(format!("failed to read file: {e}")))?;

    let upload_dir =
        std::env::var("REQ1_UPLOAD_DIR").unwrap_or_else(|_| DEFAULT_UPLOAD_DIR.to_owned());

    let result = AttachmentService::create(
        &state.db,
        object_id,
        file_name,
        content_type,
        &data,
        &upload_dir,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(result)))
}

#[utoipa::path(get, path = "/api/v1/objects/{object_id}/attachments/{id}/download",
    tag = "Attachments",
    security(("bearer_auth" = [])),
    params(
        ("object_id" = Uuid, Path, description = "Object ID"),
        ("id" = Uuid, Path, description = "Attachment ID"),
    ),
    responses((status = 200, content_type = "application/octet-stream", body = Vec<u8>))
)]
pub(crate) async fn download_attachment(
    State(state): State<AppState>,
    Path((_object_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Response, AppError> {
    let attachment = AttachmentService::get(&state.db, id).await?;
    let data = AttachmentService::read_file(&attachment.storage_path)?;

    if !AttachmentService::verify_integrity(&attachment, &data) {
        return Err(AppError::internal(format!(
            "integrity check failed for attachment {}: SHA-256 mismatch",
            attachment.id
        )));
    }

    let mut headers = HeaderMap::new();
    if let Ok(ct) = HeaderValue::from_str(&attachment.content_type) {
        let _ = headers.insert(header::CONTENT_TYPE, ct);
    }
    let disposition = format!(
        "attachment; filename=\"{}\"",
        attachment.file_name.replace('"', "\\\"")
    );
    if let Ok(disp) = HeaderValue::from_str(&disposition) {
        let _ = headers.insert(header::CONTENT_DISPOSITION, disp);
    }

    Ok((headers, Body::from(data)).into_response())
}

#[derive(Serialize, ToSchema)]
pub(crate) struct VerifyResult {
    attachment_id: Uuid,
    file_name: String,
    expected_sha256: Option<String>,
    actual_sha256: String,
    valid: bool,
}

#[utoipa::path(get, path = "/api/v1/objects/{object_id}/attachments/{id}/verify",
    tag = "Attachments",
    security(("bearer_auth" = [])),
    params(
        ("object_id" = Uuid, Path, description = "Object ID"),
        ("id" = Uuid, Path, description = "Attachment ID"),
    ),
    responses((status = 200, body = VerifyResult))
)]
pub(crate) async fn verify_attachment(
    State(state): State<AppState>,
    Path((_object_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Json<VerifyResult>, AppError> {
    let attachment = AttachmentService::get(&state.db, id).await?;
    let data = AttachmentService::read_file(&attachment.storage_path)?;

    let mut hasher = Sha256::new();
    hasher.update(&data);
    let actual_sha256 = format!("{:x}", hasher.finalize());

    let valid = attachment
        .sha256
        .as_ref()
        .is_none_or(|expected| *expected == actual_sha256);

    Ok(Json(VerifyResult {
        attachment_id: attachment.id,
        file_name: attachment.file_name,
        expected_sha256: attachment.sha256,
        actual_sha256,
        valid,
    }))
}

#[utoipa::path(delete, path = "/api/v1/objects/{object_id}/attachments/{id}", tag = "Attachments",
    security(("bearer_auth" = [])),
    params(
        ("object_id" = Uuid, Path, description = "Object ID"),
        ("id" = Uuid, Path, description = "Attachment ID"),
    ),
    responses((status = 204, description = "Deleted"), (status = 404, description = "Not found"))
)]
pub(crate) async fn delete_attachment(
    State(state): State<AppState>,
    Path((_object_id, id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    AttachmentService::delete(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
