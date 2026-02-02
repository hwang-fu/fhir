//! Audit logging middleware for mutations

use axum::{body::Body, extract::Request, http::Method, middleware::Next, response::Response};

use super::request_id::RequestId;

/// Middleware to log mutations (POST, PUT, DELETE) for audit purposes
pub async fn audit_middleware(request: Request<Body>, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().path().to_string();
    let request_id = request
        .extensions()
        .get::<RequestId>()
        .map(|r| r.0.clone())
        .unwrap_or_else(|| "unknown".to_string());

    // Run the request first to get the response status
    let response = next.run(request).await;

    // Only log mutations (POST, PUT, DELETE)
    if matches!(method, Method::POST | Method::PUT | Method::DELETE) {
        let status = response.status().as_u16();

        tracing::info!(
            target: "audit",
            request_id = %request_id,
            method = %method,
            path = %uri,
            status = %status,
            "Mutation request"
        );
    }

    response
}
