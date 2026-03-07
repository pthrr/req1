use axum::{
    Extension, Json, Router,
    extract::State,
    routing::{get, post},
};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{error::AppError, state::AppState};
use req1_core::auth::AuthUser;
use req1_core::service::auth::AuthService;

pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
}

pub fn protected_routes() -> Router<AppState> {
    Router::new()
        .route("/auth/me", get(me))
        .route("/auth/change-password", post(change_password))
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct RegisterRequest {
    email: String,
    password: String,
    display_name: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct ChangePasswordRequest {
    old_password: String,
    new_password: String,
}

#[utoipa::path(post, path = "/api/v1/auth/register", tag = "Auth",
    request_body = RegisterRequest,
    responses(
        (status = 201, body = entity::app_user::Model),
        (status = 400, description = "Bad request")
    )
)]
pub(crate) async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> Result<(axum::http::StatusCode, Json<entity::app_user::Model>), AppError> {
    let user =
        AuthService::register(&state.db, &body.email, &body.password, &body.display_name).await?;
    Ok((axum::http::StatusCode::CREATED, Json(user)))
}

#[utoipa::path(post, path = "/api/v1/auth/login", tag = "Auth",
    request_body = LoginRequest,
    responses(
        (status = 200, body = req1_core::service::auth::LoginResponse),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<req1_core::service::auth::LoginResponse>, AppError> {
    let response = AuthService::login(
        &state.db,
        &body.email,
        &body.password,
        &state.config.jwt_secret,
        state.config.jwt_expiration_hours,
    )
    .await?;
    Ok(Json(response))
}

#[utoipa::path(get, path = "/api/v1/auth/me", tag = "Auth",
    security(("bearer_auth" = [])),
    responses((status = 200, body = AuthUser))
)]
pub(crate) async fn me(Extension(auth_user): Extension<AuthUser>) -> Json<AuthUser> {
    Json(auth_user)
}

#[utoipa::path(post, path = "/api/v1/auth/change-password", tag = "Auth",
    security(("bearer_auth" = [])),
    request_body = ChangePasswordRequest,
    responses(
        (status = 204, description = "Password changed"),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn change_password(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(body): Json<ChangePasswordRequest>,
) -> Result<axum::http::StatusCode, AppError> {
    AuthService::change_password(
        &state.db,
        auth_user.id,
        &body.old_password,
        &body.new_password,
    )
    .await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
