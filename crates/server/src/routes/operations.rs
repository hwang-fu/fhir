//! AI-powered operation endpoints ($nl-search, $generate, $chat)

use axum::{Extension, Json, extract::State, http::StatusCode, response::IntoResponse};
use deadpool_postgres::Pool;
use fhir_core::{Bundle, BundleEntry};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::ai::ClaudeClient;
use crate::db::PatientRepository;
use crate::error::AppError;

/// Request body for natural language search
#[derive(Deserialize)]
pub struct NlSearchRequest {
    query: String,
}

/// Request body for patient generation
#[derive(Deserialize)]
pub struct GenerateRequest {
    count: Option<u32>,
}

/// Response body for patient generation
#[derive(Serialize)]
pub struct GenerateResponse {
    created: u32,
    resources: Vec<JsonValue>,
}

/// Request body for chat
#[derive(Deserialize)]
pub struct ChatRequest {
    message: String,
}

/// Response body for chat
#[derive(Serialize)]
pub struct ChatResponse {
    response: String,
}

/// POST /fhir/Patient/$nl-search — Natural language patient search
///
/// Accepts a plain-English query, uses Claude to convert it into FHIR search
/// parameters, executes the search, and returns a standard FHIR Bundle.
pub async fn nl_search(
    State(pool): State<Pool>,
    Extension(client): Extension<Option<ClaudeClient>>,
    Json(body): Json<NlSearchRequest>,
) -> Result<impl IntoResponse, AppError> {
    let client = client
        .ok_or_else(|| AppError::Internal("ANTHROPIC_API_KEY not configured".to_string()))?;

    tracing::info!(query = &body.query, "Natural language search");

    // Convert natural language to FHIR search params via Claude
    let params = crate::ai::nl_search::convert_to_params(&client, &body.query)
        .await
        .map_err(|e| AppError::Internal(format!("AI search conversion failed: {}", e)))?;

    tracing::info!(params = %params, "Converted NL query to FHIR params");

    // Execute the search
    let repo = PatientRepository::new(pool);
    let results = repo.search(params.clone()).await?;
    let total = repo.count(params).await? as u32;

    // Build bundle response
    let entries: Vec<BundleEntry> = results
        .into_iter()
        .map(|(id, data)| BundleEntry::new(Some(format!("/fhir/Patient/{}", id)), data))
        .collect();

    let bundle = Bundle::searchset(total, entries);
    Ok(Json(bundle))
}

/// POST /fhir/Patient/$generate — Generate synthetic patient data
///
/// Uses Claude to generate realistic FHIR R4 Patient resources, stores them
/// in the database, and returns the created resources.
pub async fn generate(
    State(pool): State<Pool>,
    Extension(client): Extension<Option<ClaudeClient>>,
    Json(body): Json<GenerateRequest>,
) -> Result<impl IntoResponse, AppError> {
    let client = client
        .ok_or_else(|| AppError::Internal("ANTHROPIC_API_KEY not configured".to_string()))?;

    let count = body.count.unwrap_or(5).min(50); // Cap at 50 to avoid abuse
    tracing::info!(count = count, "Generating synthetic patients");

    // Generate patients via Claude
    let patients = crate::ai::generator::generate_patients(&client, count)
        .await
        .map_err(|e| AppError::Internal(format!("AI generation failed: {}", e)))?;

    // Store each generated patient in the database
    let repo = PatientRepository::new(pool);
    let mut created = Vec::new();
    for patient in patients {
        match repo.create(patient.clone()).await {
            Ok(id) => {
                tracing::info!(patient_id = %id, "Generated patient stored");
                let mut resource = patient;
                if let Some(obj) = resource.as_object_mut() {
                    obj.insert("id".to_string(), JsonValue::String(id.to_string()));
                }
                created.push(resource);
            }
            Err(e) => {
                tracing::warn!(error = ?e, "Failed to store generated patient");
            }
        }
    }

    Ok((
        StatusCode::CREATED,
        Json(GenerateResponse {
            created: created.len() as u32,
            resources: created,
        }),
    ))
}

/// POST /fhir/$chat — AI chatbot with tool calling
///
/// Runs an agentic loop: Claude can call tools (search_patients, get_patient,
/// count_patients) to look up real data before composing a natural language answer.
pub async fn chat(
    State(pool): State<Pool>,
    Extension(client): Extension<Option<ClaudeClient>>,
    Json(body): Json<ChatRequest>,
) -> Result<impl IntoResponse, AppError> {
    let client = client
        .ok_or_else(|| AppError::Internal("ANTHROPIC_API_KEY not configured".to_string()))?;

    tracing::info!(message = &body.message, "Chat request");

    let repo = PatientRepository::new(pool);
    let response = crate::ai::chatbot::chat(&client, &repo, &body.message)
        .await
        .map_err(|e| AppError::Internal(format!("Chat failed: {}", e)))?;

    Ok(Json(ChatResponse { response }))
}
