use pgrx::prelude::*;

/// Retrieve a specific version of a FHIR resource
///
/// Returns the resource data at the specified version, or None if not found.
#[pg_extern]
fn fhir_get_version(
    resource_type: &str,
    resource_id: pgrx::Uuid,
    version: i32,
) -> Option<pgrx::JsonB> {
    Spi::get_one_with_args(
        "SELECT data FROM fhir_history
           WHERE resource_id = $1 AND resource_type = $2 AND version = $3",
        &[resource_id.into(), resource_type.into(), version.into()],
    )
    .ok()
    .flatten()
}
