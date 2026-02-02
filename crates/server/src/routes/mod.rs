//! HTTP route definitions

mod patient;

use axum::{Router, routing::get};
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
}
