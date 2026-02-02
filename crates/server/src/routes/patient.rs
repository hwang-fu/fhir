
use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
};
use deadpool_postgres::Pool;
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::db::PatientRepository;
use crate::error::AppError;

/// POST /fhir/Patient - Create a new patient
pub async fn create(
    State(pool): State<Pool>,
    Json(body): Json<JsonValue>,
) -> Result<impl IntoResponse, AppError> {
    let repo = PatientRepository::new(pool);
    let id = repo.create(body).await?;

    let mut headers = HeaderMap::new();
    headers.insert(
        header::LOCATION,
        format!("/fhir/Patient/{}", id).parse().unwrap(),
    );
    headers.insert("ETag", format!("W/\"1\"").parse().unwrap());

    Ok((StatusCode::CREATED, headers))
}
