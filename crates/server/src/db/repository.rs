use deadpool_postgres::Pool;
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::error::AppError;

/// Repository for Patient CRUD operations
#[derive(Clone)]
pub struct PatientRepository {
    pool: Pool,
}

impl PatientRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    /// Create a new patient
    pub async fn create(&self, data: JsonValue) -> Result<Uuid, AppError> {
        let client = self.pool.get().await?;
        let row = client
            .query_one("SELECT fhir_put('Patient', $1::jsonb)", &[&data])
            .await?;
        Ok(row.get(0))
    }

    /// Get a patient by ID
    pub async fn get(&self, id: Uuid) -> Result<Option<JsonValue>, AppError> {
        let client = self.pool.get().await?;
        let row = client
            .query_opt("SELECT fhir_get('Patient', $1::uuid)", &[&id])
            .await?;

        match row {
            Some(row) => Ok(row.get(0)),
            None => Ok(None),
        }
    }

    /// Update a patient
    pub async fn update(&self, id: Uuid, data: JsonValue) -> Result<Option<i32>, AppError> {
        let client = self.pool.get().await?;
        let row = client
            .query_opt(
                "SELECT fhir_update('Patient', $1::uuid, $2::jsonb)",
                &[&id, &data],
            )
            .await?;

        match row {
            Some(row) => Ok(row.get(0)),
            None => Ok(None),
        }
    }
}
