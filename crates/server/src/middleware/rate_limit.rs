use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;
use std::sync::Arc;

use fhir_core::OperationOutcome;

/// Rate limiter state (shared across requests)
pub type SharedRateLimiter =
    Arc<RateLimiter<(), governor::state::direct::NotKeyed, governor::clock::DefaultClock>>;
