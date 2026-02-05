//! fhir-server: FHIR R4 HTTP Server
//!
//! An Axum-based HTTP server implementing FHIR R4 Patient resource endpoints.

mod ai;
mod config;
mod db;
mod error;
mod middleware;
mod routes;

use axum::{Extension, Router, middleware as axum_mw, routing::get};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use middleware::ApiKeyAuth;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    // Load configuration
    let config = config::Config::from_env();

    // Create database pool
    let pool = db::create_pool(&config.database_url)
        .await
        .expect("Failed to create database pool");

    // Create auth state
    let auth = ApiKeyAuth::new(config.api_key.clone());

    // Create rate limiter
    let rate_limiter = middleware::create_rate_limiter(config.rate_limit_rps);
    tracing::info!("Rate limiting: {} requests/second", config.rate_limit_rps);

    // Log whether auth is enabled
    if config.api_key.is_some() {
        tracing::info!("API key authentication enabled");
    } else {
        tracing::warn!("API key authentication disabled (no API_KEY env var)");
    }

    // Create Claude client (None if ANTHROPIC_API_KEY not set)
    let claude_client: Option<ai::ClaudeClient> = if let Some(ref key) = config.anthropic_api_key {
        tracing::info!("Anthropic API key configured, AI features enabled");
        Some(ai::ClaudeClient::new(key.clone()))
    } else {
        tracing::warn!("ANTHROPIC_API_KEY not set, AI features disabled");
        None
    };

    // Protected routes (require auth)
    let protected_routes = Router::new()
        .nest("/fhir", routes::fhir_routes())
        .layer(axum_mw::from_fn(middleware::auth::auth_middleware))
        .layer(Extension(auth))
        .layer(Extension(claude_client))
        .layer(axum_mw::from_fn(middleware::rate_limit_middleware))
        .layer(Extension(rate_limiter));

    // Install Prometheus metrics recorder
    let prometheus_handle = metrics_exporter_prometheus::PrometheusBuilder::new()
        .install_recorder()
        .expect("Failed to install Prometheus recorder");

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
    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(pool)
        .layer(axum_mw::from_fn(middleware::audit_middleware))
        .layer(axum_mw::from_fn(middleware::request_id_middleware))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(axum_mw::from_fn(middleware::metrics_middleware));

    // Start server
    let addr: SocketAddr = config.bind_address.parse().expect("Invalid bind address");
    tracing::info!("Starting FHIR server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    tracing::info!("Server shutdown complete");
}

/// Wait for shutdown signal (SIGTERM or SIGINT)
async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C, starting graceful shutdown");
        }
        _ = terminate => {
            tracing::info!("Received SIGTERM, starting graceful shutdown");
        }
    }
}
