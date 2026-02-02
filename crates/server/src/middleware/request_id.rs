use axum::{body::Body, extract::Request, http::HeaderValue, middleware::Next, response::Response};
use uuid::Uuid;

/// Header name for request ID
pub const REQUEST_ID_HEADER: &str = "X-Request-ID";
