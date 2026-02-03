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
    let version = 1 as i32;

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
    // Use ok().flatten() to convert "no rows" error to None
    Spi::get_one_with_args(
        "SELECT data FROM fhir_resources WHERE id = $1 AND resource_type = $2 AND deleted_at IS NULL",
        &[id.into(), resource_type.into()],
    )
    .ok()
    .flatten()
}

/// Soft-delete a FHIR resource
///
/// Sets deleted_at timestamp and records the deletion in history.
/// Returns true if a resource was deleted, false if not found.
#[pg_extern]
fn fhir_delete(resource_type: &str, id: pgrx::Uuid) -> bool {
    // Get current version before deletion
    let current_version: Option<i32> = Spi::get_one_with_args(
        "SELECT version FROM fhir_resources WHERE id = $1 AND resource_type = $2 AND deleted_at IS NULL",
        &[id.into(), resource_type.into()],
    )
    .ok()
    .flatten();

    let Some(version) = current_version else {
        return false;
    };

    // Soft delete the resource
    Spi::run_with_args(
        "UPDATE fhir_resources SET deleted_at = NOW() WHERE id = $1 AND resource_type =
  $2",
        &[id.into(), resource_type.into()],
    )
    .expect("Failed to delete resource");

    // Record deletion in history (store empty JSON to mark deletion)
    let new_version = version + 1;
    let empty_data = pgrx::JsonB(serde_json::json!({"deleted": true}));

    Spi::run_with_args(
        "INSERT INTO fhir_history (resource_id, resource_type, version, data) VALUES ($1,
  $2, $3, $4)",
        &[
            id.into(),
            resource_type.into(),
            new_version.into(),
            empty_data.into(),
        ],
    )
    .expect("Failed to insert history");

    true
}

/// Update an existing FHIR resource
///
/// Increments version and records the update in history.
/// Returns the new version number, or None if resource not found.
#[pg_extern]
fn fhir_update(resource_type: &str, id: pgrx::Uuid, data: pgrx::JsonB) -> Option<i32> {
    // Get current version
    let current_version: Option<i32> = Spi::get_one_with_args(
        "SELECT version FROM fhir_resources WHERE id = $1 AND resource_type = $2 AND deleted_at IS NULL",
        &[id.into(), resource_type.into()],
    )
    .ok()
    .flatten();

    let Some(version) = current_version else {
        return None;
    };

    let new_version = version + 1;
    let data_for_history = pgrx::JsonB(data.0.clone());

    // Update the resource
    Spi::run_with_args(
        "UPDATE fhir_resources SET data = $1, version = $2, updated_at = NOW() WHERE id =
  $3 AND resource_type = $4",
        &[
            data.into(),
            new_version.into(),
            id.into(),
            resource_type.into(),
        ],
    )
    .expect("Failed to update resource");

    // Record in history
    Spi::run_with_args(
        "INSERT INTO fhir_history (resource_id, resource_type, version, data) VALUES ($1,
  $2, $3, $4)",
        &[
            id.into(),
            resource_type.into(),
            new_version.into(),
            data_for_history.into(),
        ],
    )
    .expect("Failed to insert history");

    Some(new_version)
}
