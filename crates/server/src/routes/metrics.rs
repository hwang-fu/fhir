//! Prometheus metrics endpoint

use axum::{Extension, response::IntoResponse};
use metrics_exporter_prometheus::PrometheusHandle;

/// GET /metrics - Render collected metrics in Prometheus text format
pub async fn get(Extension(handle): Extension<PrometheusHandle>) -> impl IntoResponse {
    handle.render()
}
