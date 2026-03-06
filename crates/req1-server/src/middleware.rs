use axum::{
    extract::{Request, State},
    http::{HeaderValue, header},
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::error::AppError;
use crate::state::AppState;

const IMMUTABLE: HeaderValue = HeaderValue::from_static("public, max-age=31536000, immutable");
const NO_CACHE: HeaderValue = HeaderValue::from_static("no-cache");

pub async fn cache_control(request: Request, next: Next) -> Response {
    let path = request.uri().path().to_string();
    let mut response = next.run(request).await;

    if path.starts_with("/assets/") {
        let _ = response
            .headers_mut()
            .insert(header::CACHE_CONTROL, IMMUTABLE.clone());
    } else if path == "/"
        || std::path::Path::new(&path)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("html"))
    {
        let _ = response
            .headers_mut()
            .insert(header::CACHE_CONTROL, NO_CACHE.clone());
    }

    response
}

pub async fn require_auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .map(ToString::to_string);

    let token = match auth_header {
        Some(ref h) if h.starts_with("Bearer ") => &h[7..],
        _ => {
            return AppError::Unauthorized("missing or invalid Authorization header".to_string())
                .into_response();
        }
    };

    match req1_core::service::auth::AuthService::verify_token(token, &state.config.jwt_secret) {
        Ok(auth_user) => {
            let _ = request.extensions_mut().insert(auth_user);
            next.run(request).await
        }
        Err(_) => {
            AppError::Unauthorized("invalid or expired token".to_string()).into_response()
        }
    }
}
