//! Patient resource HTTP handlers

use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
};
use deadpool_postgres::Pool;
use fhir_core::{Bundle, BundleEntry};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::db::PatientRepository;
use crate::error::AppError;

/// Query parameters for patient search
#[derive(Debug, Deserialize, Default)]
pub struct SearchParams {
    pub name: Option<String>,
    pub gender: Option<String>,
    pub birthdate: Option<String>,
    #[serde(rename = "_count")]
    pub count: Option<i64>,
    #[serde(rename = "_offset")]
    pub offset: Option<i64>,
    #[serde(rename = "_sort")]
    pub sort: Option<String>,
}

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
    headers.insert("ETag", "W/\"1\"".parse().unwrap());

    Ok((StatusCode::CREATED, headers))
}

/// GET /fhir/Patient/{id} - Read a patient
pub async fn read(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let repo = PatientRepository::new(pool);

    match repo.get(id).await? {
        Some(data) => {
            let mut headers = HeaderMap::new();
            // Extract version from meta if available, default to 1
            let version = data
                .get("meta")
                .and_then(|m| m.get("versionId"))
                .and_then(|v| v.as_str())
                .unwrap_or("1");
            headers.insert("ETag", format!("W/\"{}\"", version).parse().unwrap());

            Ok((StatusCode::OK, headers, Json(data)))
        }
        None => Err(AppError::NotFound(format!("Patient/{} not found", id))),
    }
}

/// PUT /fhir/Patient/{id} - Update a patient
pub async fn update(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
    Json(body): Json<JsonValue>,
) -> Result<impl IntoResponse, AppError> {
    let repo = PatientRepository::new(pool);

    match repo.update(id, body).await? {
        Some(version) => {
            let mut headers = HeaderMap::new();
            headers.insert("ETag", format!("W/\"{}\"", version).parse().unwrap());

            Ok((StatusCode::OK, headers))
        }
        None => Err(AppError::NotFound(format!("Patient/{} not found", id))),
    }
}

/// DELETE /fhir/Patient/{id} - Delete a patient
pub async fn delete(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let repo = PatientRepository::new(pool);

    if repo.delete(id).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound(format!("Patient/{} not found", id)))
    }
}

/// GET /fhir/Patient - Search patients (placeholder)
pub async fn search(State(_pool): State<Pool>) -> Result<impl IntoResponse, AppError> {
    // TODO: Implement search in Phase 6
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "resourceType": "Bundle",
            "type": "searchset",
            "total": 0,
            "entry": []
        })),
    ))
}
