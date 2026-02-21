use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum AppError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("internal error: {0}")]
    Internal(String),

    #[error("database error: {0}")]
    DbError(#[from] sea_orm::DbErr),
}

impl From<req1_core::error::CoreError> for AppError {
    fn from(err: req1_core::error::CoreError) -> Self {
        match err {
            req1_core::error::CoreError::NotFound(msg) => Self::NotFound(msg),
            req1_core::error::CoreError::BadRequest(msg) => Self::BadRequest(msg),
            req1_core::error::CoreError::Internal(msg) => Self::Internal(msg),
            req1_core::error::CoreError::Db(db_err) => Self::DbError(db_err),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg.clone()),
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {msg}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "internal server error".to_string(),
                )
            }
            AppError::DbError(err) => {
                tracing::error!("Database error: {err}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "DB_ERROR",
                    "database error".to_string(),
                )
            }
        };

        let body = json!({
            "error": {
                "code": code,
                "message": message,
            }
        });

        (status, axum::Json(body)).into_response()
    }
}
