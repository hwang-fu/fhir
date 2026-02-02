//! Patient resource HTTP handlers

use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
};
use deadpool_postgres::Pool;
use fhir_core::{Bundle, BundleEntry};
use serde::Deserialize;
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

impl SearchParams {
    /// Convert to JSON for the PGRX search function
    fn to_json(&self) -> JsonValue {
        let mut map = serde_json::Map::new();

        if let Some(ref name) = self.name {
            map.insert("name".to_string(), JsonValue::String(name.clone()));
        }
        if let Some(ref gender) = self.gender {
            map.insert("gender".to_string(), JsonValue::String(gender.clone()));
        }
        if let Some(ref birthdate) = self.birthdate {
            map.insert(
                "birthdate".to_string(),
                JsonValue::String(birthdate.clone()),
            );
        }
        if let Some(count) = self.count {
            map.insert("_count".to_string(), JsonValue::Number(count.into()));
        }
        if let Some(offset) = self.offset {
            map.insert("_offset".to_string(), JsonValue::Number(offset.into()));
        }
        if let Some(ref sort) = self.sort {
            map.insert("_sort".to_string(), JsonValue::String(sort.clone()));
        }

        JsonValue::Object(map)
    }
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

/// GET /fhir/Patient - Search patients
pub async fn search(
    State(pool): State<Pool>,
    Query(params): Query<SearchParams>,
) -> Result<impl IntoResponse, AppError> {
    let repo = PatientRepository::new(pool);
    let json_params = params.to_json();

    // Get search results
    let results = repo.search(json_params.clone()).await?;

    // Get total count for pagination
    let total = repo.count(json_params).await? as u32;

    // Build bundle entries
    let entries: Vec<BundleEntry> = results
        .into_iter()
        .map(|(id, data)| BundleEntry::new(Some(format!("/fhir/Patient/{}", id)), data))
        .collect();

    // Create bundle response
    let bundle = Bundle::searchset(total, entries);

    Ok(Json(bundle))
}

/// GET /fhir/Patient/{id}/_history - Get patient history
pub async fn history(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let repo = PatientRepository::new(pool);
    let versions = repo.history(id).await?;

    // If no history found, the resource doesn't exist
    if versions.is_empty() {
        return Err(AppError::NotFound(format!("Patient/{} not found", id)));
    }

    // Build bundle entries with versioned URLs
    let entries: Vec<BundleEntry> = versions
        .into_iter()
        .map(|(version, data)| {
            BundleEntry::new(
                Some(format!("/fhir/Patient/{}/_history/{}", id, version)),
                data,
            )
        })
        .collect();

    // Create history bundle
    let bundle = Bundle::history(entries);

    Ok(Json(bundle))
}

/// POST /fhir/Patient/$validate - Validate a patient without storing
pub async fn validate(Json(body): Json<JsonValue>) -> impl IntoResponse {
    // Check resourceType is present and correct
    let resource_type = body.get("resourceType").and_then(|v| v.as_str());

    match resource_type {
        Some("Patient") => {
            // Try to deserialize into fhir-sdk Patient type for validation
            match serde_json::from_value::<fhir_core::Patient>(body) {
                Ok(_) => {
                    // Valid Patient resource
                    let outcome = fhir_core::OperationOutcome::success("Patient resource is valid");
                    (StatusCode::OK, Json(outcome))
                }
                Err(e) => {
                    // Deserialization failed - validation error
                    let outcome =
                        fhir_core::OperationOutcome::invalid(&format!("Validation failed: {}", e));
                    (StatusCode::BAD_REQUEST, Json(outcome))
                }
            }
        }
        Some(other) => {
            let outcome = fhir_core::OperationOutcome::invalid(&format!(
                "Expected resourceType 'Patient', got '{}'",
                other
            ));
            (StatusCode::BAD_REQUEST, Json(outcome))
        }
        None => {
            let outcome =
                fhir_core::OperationOutcome::invalid("Missing required field: resourceType");
            (StatusCode::BAD_REQUEST, Json(outcome))
        }
    }
}
