//! fhir-server library crate
//!
//! Exposes `build_app` and `config` for integration tests.
//! The actual binary entrypoint is in `main.rs`.

mod ai;
pub mod config;
pub mod db;
mod error;
mod middleware;
mod routes;

use axum::{Extension, Router, middleware as axum_mw, routing::get};
use deadpool_postgres::Pool;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use config::Config;
use middleware::ApiKeyAuth;

/// Build the full application router with all routes and middleware.
///
/// Extracted from `main()` so integration tests can construct the app
/// without binding to a TCP port.
pub fn build_app(pool: Pool, config: &Config) -> Router {
    // Create auth state
    let auth = ApiKeyAuth::new(config.api_key.clone());

    // Create rate limiter
    let rate_limiter = middleware::create_rate_limiter(config.rate_limit_rps);

    // Create Claude client (None if ANTHROPIC_API_KEY not set)
    let claude_client: Option<ai::ClaudeClient> = config
        .anthropic_api_key
        .as_ref()
        .map(|key| ai::ClaudeClient::new(key.clone()));

    // Protected routes (require auth)
    let protected_routes = Router::new()
        .nest("/fhir", routes::fhir_routes())
        .layer(axum_mw::from_fn(middleware::auth::auth_middleware))
        .layer(Extension(auth))
        .layer(Extension(claude_client))
        .layer(axum_mw::from_fn(middleware::rate_limit_middleware))
        .layer(Extension(rate_limiter));

    // Install Prometheus metrics recorder.
    // Use build_recorder() + set_global_recorder() so that repeated calls
    // (e.g. in integration tests) don't panic — the second install is
    // silently ignored and we still get a valid handle for /metrics.
    let recorder = metrics_exporter_prometheus::PrometheusBuilder::new().build_recorder();
    let prometheus_handle = recorder.handle();
    let _ = metrics::set_global_recorder(recorder);

    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/metadata", get(routes::metadata::get))
        .route("/health", get(routes::health::check))
        .route("/metrics", get(routes::metrics::get))
        .layer(Extension(prometheus_handle));

    // Build CORS layer
    let cors = if config.cors_origins.iter().any(|o| o == "*") {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        let origins: Vec<_> = config
            .cors_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods(Any)
            .allow_headers(Any)
    };

    // Build application
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(pool)
        .layer(axum_mw::from_fn(middleware::audit_middleware))
        .layer(axum_mw::from_fn(middleware::request_id_middleware))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(axum_mw::from_fn(middleware::metrics_middleware))
}
