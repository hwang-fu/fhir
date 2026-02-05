//! HTTP route definitions

pub mod health;
pub mod metadata;
pub mod metrics;
mod operations;
mod patient;

use axum::{
    Router,
    routing::{get, post},
};
use deadpool_postgres::Pool;

/// Build FHIR routes
pub fn fhir_routes() -> Router<Pool> {
    Router::new()
        .route("/Patient", get(patient::search).post(patient::create))
        .route(
            "/Patient/{id}",
            get(patient::read)
                .put(patient::update)
                .delete(patient::delete),
        )
        .route("/Patient/{id}/_history", get(patient::history))
        .route("/Patient/$validate", post(patient::validate))
        .route("/Patient/$nl-search", post(operations::nl_search))
        .route("/Patient/$generate", post(operations::generate))
        .route("/$chat", post(operations::chat))
}
