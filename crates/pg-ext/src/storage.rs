//! FHIR resource storage functions (CRUD operations)

use pgrx::prelude::*;
use uuid::Uuid;

/// Create a new FHIR resource
///
/// Inserts a new resource with version 1, also recording it in history.
/// Returns the generated UUID for the resource.
#[pg_extern]
fn fhir_put(resource_type: &str, data: pgrx::JsonB) -> pgrx::Uuid {
    let id = Uuid::new_v4();
    let id_bytes = *id.as_bytes();
    let version = 1_i32;

    // Clone the inner JSON value for the history insert
    let data_for_history = pgrx::JsonB(data.0.clone());

    // Insert into main resources table
    Spi::run_with_args(
        "INSERT INTO fhir_resources (id, resource_type, version, data) VALUES ($1, $2, $3, $4)",
        &[
            pgrx::Uuid::from_bytes(id_bytes).into(),
            resource_type.into(),
            version.into(),
            data.into(),
        ],
    )
    .expect("Failed to insert resource");

    // Insert into history table
    Spi::run_with_args(
        "INSERT INTO fhir_history (resource_id, resource_type, version, data) VALUES ($1, $2, $3, $4)",
        &[
            pgrx::Uuid::from_bytes(id_bytes).into(),
            resource_type.into(),
            version.into(),
            data_for_history.into(),
        ],
    )
    .expect("Failed to insert history");

    pgrx::Uuid::from_bytes(id_bytes)
}

/// Retrieve a FHIR resource by ID
///
/// Returns the resource data as JSONB, or None if not found or deleted.
#[pg_extern]
fn fhir_get(resource_type: &str, id: pgrx::Uuid) -> Option<pgrx::JsonB> {
    Spi::get_one_with_args(
        "SELECT data FROM fhir_resources WHERE id = $1 AND resource_type = $2 AND
  deleted_at IS NULL",
        &[id.into(), resource_type.into()],
    )
    .expect("Failed to query resource")
}
