//! fhir-server: FHIR R4 HTTP Server
//!
//! An Axum-based HTTP server implementing FHIR R4 Patient resource endpoints.

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
        .with(tracing_subscriber::fmt::layer())
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

    // Protected routes (require auth)
    let protected_routes = Router::new()
        .nest("/fhir", routes::fhir_routes())
        .layer(axum_mw::from_fn(middleware::auth::auth_middleware))
        .layer(Extension(auth))
        .layer(axum_mw::from_fn(middleware::rate_limit_middleware))
        .layer(Extension(rate_limiter));

    // Public routes (no auth required)
    let public_routes = Router::new().route("/metadata", get(routes::metadata::get));

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
        .layer(TraceLayer::new_for_http());

    // Start server
    let addr: SocketAddr = config.bind_address.parse().expect("Invalid bind address");
    tracing::info!("Starting FHIR server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
