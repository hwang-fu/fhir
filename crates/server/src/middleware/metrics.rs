//! Prometheus metrics collection middleware
//!
//! Records `http_requests_total` (counter) and `http_request_duration_seconds`
//! (histogram) for every request, with method/path/status labels.

use axum::{extract::Request, middleware::Next, response::Response};
use std::time::Instant;

/// Normalize request paths to avoid high-cardinality labels.
/// Replaces UUID segments with `:id` so all per-resource requests share one label.
fn normalize_path(path: &str) -> String {
    path.split('/')
        .map(|seg| {
            if uuid::Uuid::try_parse(seg).is_ok() {
                ":id"
            } else {
                seg
            }
        })
        .collect::<Vec<_>>()
        .join("/")
}

/// Middleware that records request count and duration metrics.
pub async fn metrics_middleware(request: Request, next: Next) -> Response {
    let method = request.method().to_string();
    let path = normalize_path(request.uri().path());

    let start = Instant::now();
    let response = next.run(request).await;
    let duration = start.elapsed().as_secs_f64();

    let status = response.status().as_u16().to_string();

    metrics::counter!(
        "http_requests_total",
        "method" => method.clone(),
        "path" => path.clone(),
        "status" => status
    )
    .increment(1);

    metrics::histogram!(
        "http_request_duration_seconds",
        "method" => method,
        "path" => path
    )
    .record(duration);

    response
}
