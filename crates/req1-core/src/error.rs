#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("unauthorized: {0}")]
    Unauthorized(String),
    #[error("internal: {0}")]
    Internal(String),
    #[error("reqif: {0}")]
    Reqif(String),
    #[error("db: {0}")]
    Db(#[from] sea_orm::DbErr),
}
