use axum::Router;

use crate::state::AppState;

pub mod app_users;
pub mod attribute_definitions;
pub mod baseline_sets;
pub mod baselines;
pub mod change_proposals;
pub mod comments;
pub mod health;
pub mod impact;
pub mod links;
pub mod modules;
pub mod object_types;
pub mod objects;
pub mod projects;
pub mod publish;
pub mod review_assignments;
pub mod review_packages;
pub mod scripts;
pub mod traceability;
pub mod validation;
pub mod views;
pub mod workspaces;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(health::routes())
        .nest("/api/v1", modules::routes())
        .nest("/api/v1", objects::routes())
        .nest("/api/v1", workspaces::routes())
        .nest("/api/v1", projects::routes())
        .nest("/api/v1", links::routes())
        .nest("/api/v1", baselines::routes())
        .nest("/api/v1", attribute_definitions::routes())
        .nest("/api/v1", traceability::routes())
        .nest("/api/v1", scripts::routes())
        .nest("/api/v1", publish::routes())
        .nest("/api/v1", validation::routes())
        .nest("/api/v1", views::routes())
        .nest("/api/v1", object_types::routes())
        .nest("/api/v1", comments::routes())
        .nest("/api/v1", app_users::routes())
        .nest("/api/v1", review_packages::routes())
        .nest("/api/v1", review_assignments::routes())
        .nest("/api/v1", change_proposals::routes())
        .nest("/api/v1", baseline_sets::routes())
        .nest("/api/v1", impact::routes())
}
