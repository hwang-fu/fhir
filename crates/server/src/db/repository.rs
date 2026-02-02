//! Patient repository for database operations

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

    /// Delete a patient
    pub async fn delete(&self, id: Uuid) -> Result<bool, AppError> {
        let client = self.pool.get().await?;
        let row = client
            .query_one("SELECT fhir_delete('Patient', $1::uuid)", &[&id])
            .await?;
        Ok(row.get(0))
    }

    /// Search for patients
    pub async fn search(&self, params: JsonValue) -> Result<Vec<(Uuid, JsonValue)>, AppError> {
        let client = self.pool.get().await?;
        let rows = client
            .query(
                "SELECT id, data FROM fhir_search('Patient', $1::jsonb)",
                &[&params],
            )
            .await?;

        let results = rows.iter().map(|row| (row.get(0), row.get(1))).collect();

        Ok(results)
    }

    /// Count total patients matching search criteria (for pagination)
    pub async fn count(&self, params: JsonValue) -> Result<i64, AppError> {
        let client = self.pool.get().await?;
        // Remove pagination params for counting
        let mut count_params = params.clone();
        if let Some(obj) = count_params.as_object_mut() {
            obj.remove("_count");
            obj.remove("_offset");
        }

        let row = client
            .query_one(
                "SELECT COUNT(*) FROM fhir_search('Patient', $1::jsonb)",
                &[&count_params],
            )
            .await?;

        Ok(row.get(0))
    }

    /// Get all versions of a patient (history)
    pub async fn history(&self, id: Uuid) -> Result<Vec<(i32, JsonValue)>, AppError> {
        let client = self.pool.get().await?;
        let rows = client
            .query(
                "SELECT version, data FROM fhir_history('Patient', $1::uuid)",
                &[&id],
            )
            .await?;

        let results = rows.iter().map(|row| (row.get(0), row.get(1))).collect();

        Ok(results)
    }
}
