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
}
