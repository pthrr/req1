use axum::Router;
use axum::middleware::from_fn_with_state;

use crate::middleware;
use crate::state::AppState;

pub mod app_users;
pub mod attachments;
pub mod attribute_definitions;
pub mod audit;
pub mod auth;
pub mod baseline_sets;
pub mod baselines;
pub mod change_proposals;
pub mod comments;
pub mod dashboards;
pub mod diagrams;
pub mod e_signatures;
pub mod health;
pub mod impact;
pub mod lifecycle;
pub mod links;
pub mod modules;
pub mod notifications;
pub mod object_types;
pub mod objects;
pub mod project_templates;
pub mod projects;
pub mod publish;
pub mod reqif;
pub mod review_assignments;
pub mod review_comments;
pub mod review_packages;
pub mod scripts;
pub mod tests;
pub mod traceability;
pub mod validation;
pub mod views;
pub mod webhooks;
pub mod workspaces;

pub fn router(state: AppState) -> Router {
    let public = Router::new()
        .merge(health::routes())
        .nest("/api/v1", auth::public_routes())
        .with_state(state.clone());

    let protected = Router::new()
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
        .nest("/api/v1", review_comments::routes())
        .nest("/api/v1", change_proposals::routes())
        .nest("/api/v1", baseline_sets::routes())
        .nest("/api/v1", impact::routes())
        .nest("/api/v1", attachments::routes())
        .nest("/api/v1", webhooks::routes())
        .nest("/api/v1", lifecycle::routes())
        .nest("/api/v1", reqif::routes())
        .nest("/api/v1", tests::routes())
        .nest("/api/v1", auth::protected_routes())
        .nest("/api/v1", audit::routes())
        .nest("/api/v1", diagrams::routes())
        .nest("/api/v1", notifications::routes())
        .nest("/api/v1", e_signatures::routes())
        .nest("/api/v1", dashboards::routes())
        .nest("/api/v1", project_templates::routes())
        .route_layer(from_fn_with_state(state.clone(), middleware::require_auth))
        .with_state(state);

    public.merge(protected)
}
