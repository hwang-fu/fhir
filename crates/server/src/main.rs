//! fhir-server: FHIR R4 HTTP Server binary entrypoint.

use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use fhir_server::config::Config;

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
    let config = Config::from_env();

    // Create database pool
    let pool = fhir_server::db::create_pool(&config.database_url)
        .await
        .expect("Failed to create database pool");

    // Log startup info
    if config.api_key.is_some() {
        tracing::info!("API key authentication enabled");
    } else {
        tracing::warn!("API key authentication disabled (no API_KEY env var)");
    }
    if config.anthropic_api_key.is_some() {
        tracing::info!("Anthropic API key configured, AI features enabled");
    } else {
        tracing::warn!("ANTHROPIC_API_KEY not set, AI features disabled");
    }
    tracing::info!("Rate limiting: {} requests/second", config.rate_limit_rps);

    // Build application
    let app = fhir_server::build_app(pool, &config);

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
