use sea_orm::DatabaseConnection;

use crate::config::Config;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub config: Config,
}
