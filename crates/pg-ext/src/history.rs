//! FHIR resource version history functionality

use pgrx::datum::TimestampWithTimeZone;
use pgrx::prelude::*;

/// Retrieve all versions of a FHIR resource
///
/// Returns all historical versions ordered by version descending (newest first).
#[pg_extern]
fn fhir_history(
    resource_type: &str,
    resource_id: pgrx::Uuid,
) -> TableIterator<
    'static,
    (
        name!(version, i32),
        name!(data, pgrx::JsonB),
        name!(created_at, TimestampWithTimeZone),
    ),
> {
    let results = Spi::connect(|client| {
        let mut results = Vec::new();
        let tup_table = client.select(
            "SELECT version, data, created_at FROM fhir_history
               WHERE resource_id = $1 AND resource_type = $2
               ORDER BY version DESC",
            None,
            &[resource_id.into(), resource_type.into()],
        )?;

        for row in tup_table {
            let version: i32 = row.get(1)?.expect("version should not be null");
            let data: pgrx::JsonB = row.get(2)?.expect("data should not be null");
            let created_at: TimestampWithTimeZone =
                row.get(3)?.expect("created_at should not be null");
            results.push((version, data, created_at));
        }

        Ok::<_, pgrx::spi::SpiError>(results)
    })
    .expect("Failed to query history");

    TableIterator::new(results)
}

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
