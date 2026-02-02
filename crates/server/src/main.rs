//! fhir-server: FHIR R4 HTTP Server
//!
//! An Axum-based HTTP server implementing FHIR R4 Patient resource endpoints.

mod config;
mod db;
mod error;
mod routes;

use axum::Router;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

    // Build application
    let app = Router::new()
        .nest("/fhir", routes::fhir_routes())
        .with_state(pool)
        .layer(TraceLayer::new_for_http());

    // Start server
    let addr: SocketAddr = config.bind_address.parse().expect("Invalid bind address");
    tracing::info!("Starting FHIR server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
