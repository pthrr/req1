use axum::{
    extract::Request,
    http::{HeaderValue, header},
    middleware::Next,
    response::Response,
};

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
