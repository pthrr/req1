use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use req1_core::error::CoreError;
use serde_json::json;

#[derive(Debug)]
pub struct AppError(pub CoreError);

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

impl From<CoreError> for AppError {
    fn from(err: CoreError) -> Self {
        Self(err)
    }
}

impl From<sea_orm::DbErr> for AppError {
    fn from(err: sea_orm::DbErr) -> Self {
        Self(CoreError::Db(err))
    }
}

// Convenience constructors — delegate to CoreError
impl AppError {
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self(CoreError::not_found(msg))
    }

    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self(CoreError::bad_request(msg))
    }

    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self(CoreError::unauthorized(msg))
    }

    pub fn forbidden(msg: impl Into<String>) -> Self {
        Self(CoreError::forbidden(msg))
    }

    pub fn conflict(msg: impl Into<String>) -> Self {
        Self(CoreError::conflict(msg))
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self(CoreError::internal(msg))
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self.0 {
            CoreError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone()),
            CoreError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg.clone()),
            CoreError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", msg.clone()),
            CoreError::Forbidden(msg) => (StatusCode::FORBIDDEN, "FORBIDDEN", msg.clone()),
            CoreError::Conflict(msg) => (StatusCode::CONFLICT, "CONFLICT", msg.clone()),
            CoreError::Internal(msg) => {
                tracing::error!("Internal error: {msg}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "internal server error".to_string(),
                )
            }
            CoreError::Db(err) => {
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
