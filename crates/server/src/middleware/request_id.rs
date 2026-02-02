use axum::{body::Body, extract::Request, http::HeaderValue, middleware::Next, response::Response};
use uuid::Uuid;

/// Header name for request ID
pub const REQUEST_ID_HEADER: &str = "X-Request-ID";

/// Middleware to add request ID to each request/response
pub async fn request_id_middleware(mut request: Request<Body>, next: Next) -> Response {
    // Get existing request ID or generate new one
    let request_id = request
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // Store request ID in extensions for logging
    request
        .extensions_mut()
        .insert(RequestId(request_id.clone()));

    // Log the request with ID
    tracing::info!(
        request_id = %request_id,
        method = %request.method(),
        uri = %request.uri(),
        "Incoming request"
    );

    // Run the request
    let mut response = next.run(request).await;

    // Add request ID to response headers
    response.headers_mut().insert(
        REQUEST_ID_HEADER,
        HeaderValue::from_str(&request_id).unwrap(),
    );

    response
}
