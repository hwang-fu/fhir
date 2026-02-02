use deadpool_postgres::Pool;
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::error::AppError;

/// Repository for Patient CRUD operations
#[derive(Clone)]
pub struct PatientRepository {
    pool: Pool,
}
