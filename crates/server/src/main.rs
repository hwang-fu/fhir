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
        .layer(Extension(auth));

    // Public routes (no auth required)
    let public_routes = Router::new().route("/metadata", get(routes::metadata::get));

    // Build application
    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(pool)
        .layer(axum_mw::from_fn(middleware::audit_middleware))
        .layer(axum_mw::from_fn(middleware::request_id_middleware))
        .layer(TraceLayer::new_for_http());

    // Start server
    let addr: SocketAddr = config.bind_address.parse().expect("Invalid bind address");
    tracing::info!("Starting FHIR server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
