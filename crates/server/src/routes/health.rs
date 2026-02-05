//! Health check endpoint

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use deadpool_postgres::Pool;
use serde::Serialize;

/// Health check response
#[derive(Serialize)]
pub struct HealthResponse {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
}

/// GET /health - Check database connectivity and return server health status
pub async fn check(State(pool): State<Pool>) -> impl IntoResponse {
    match pool.get().await {
        Ok(client) => match client.query_one("SELECT 1", &[]).await {
            Ok(_) => (
                StatusCode::OK,
                Json(HealthResponse {
                    status: "healthy".to_string(),
                    reason: None,
                }),
            ),
            Err(e) => {
                tracing::error!(error = %e, "Health check query failed");
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(HealthResponse {
                        status: "unhealthy".to_string(),
                        reason: Some(format!("Database query failed: {}", e)),
                    }),
                )
            }
        },
        Err(e) => {
            tracing::error!(error = %e, "Health check pool error");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(HealthResponse {
                    status: "unhealthy".to_string(),
                    reason: Some(format!("Database connection failed: {}", e)),
                }),
            )
        }
    }
}
