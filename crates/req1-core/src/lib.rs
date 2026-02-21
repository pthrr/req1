use serde::{Deserialize, Serialize};

pub mod baseline;
pub mod error;
pub mod fingerprint;
pub mod history;
pub mod level;
pub mod scripting;
pub mod service;
pub mod suspect;
pub mod validation;

/// Common ID type used across the application.
pub type Id = uuid::Uuid;

/// Pagination parameters for list endpoints.
#[derive(Debug, Clone, Deserialize)]
pub struct Pagination {
    #[serde(default = "default_offset")]
    pub offset: u64,
    #[serde(default = "default_limit")]
    pub limit: u64,
}

const fn default_offset() -> u64 {
    0
}

const fn default_limit() -> u64 {
    50
}

/// Paginated response wrapper.
#[derive(Debug, Clone, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub items: Vec<T>,
    pub total: u64,
    pub offset: u64,
    pub limit: u64,
}
