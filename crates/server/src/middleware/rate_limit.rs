//! Rate limiting middleware

use axum::{
    Json,
    body::Body,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use governor::{Quota, RateLimiter, clock::DefaultClock, state::InMemoryState};
use std::num::NonZeroU32;
use std::sync::Arc;

use fhir_core::OperationOutcome;

/// Rate limiter state (shared across requests)
pub type SharedRateLimiter =
    Arc<RateLimiter<governor::state::NotKeyed, InMemoryState, DefaultClock>>;

/// Create a new rate limiter with specified requests per second
pub fn create_rate_limiter(requests_per_second: u32) -> SharedRateLimiter {
    let quota = Quota::per_second(NonZeroU32::new(requests_per_second).unwrap());
    Arc::new(RateLimiter::direct(quota))
}

/// Rate limiting middleware
pub async fn rate_limit_middleware(request: Request<Body>, next: Next) -> Response {
    // Get rate limiter from extensions
    let limiter = request.extensions().get::<SharedRateLimiter>().cloned();

    if let Some(limiter) = limiter {
        // Check if request is allowed
        if limiter.check().is_err() {
            let outcome = OperationOutcome::error(
                fhir_core::IssueType::Throttled,
                "Rate limit exceeded. Please try again later.",
            );
            return (StatusCode::TOO_MANY_REQUESTS, Json(outcome)).into_response();
        }
    }

    next.run(request).await
}
