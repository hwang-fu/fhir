use axum::{
    Json,
    body::Body,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;
use std::sync::Arc;

use fhir_core::OperationOutcome;

/// Rate limiter state (shared across requests)
pub type SharedRateLimiter =
    Arc<RateLimiter<(), governor::state::direct::NotKeyed, governor::clock::DefaultClock>>;

/// Create a new rate limiter with specified requests per second
pub fn create_rate_limiter(requests_per_second: u32) -> SharedRateLimiter {
    let quota = Quota::per_second(NonZeroU32::new(requests_per_second).unwrap());
    Arc::new(RateLimiter::direct(quota))
}
