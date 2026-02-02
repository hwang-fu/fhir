//! fhir-pg-ext: PostgreSQL extension for FHIR storage
//!
//! A PGRX-based PostgreSQL extension providing FHIR resource
//! storage, search, and history functionality.

use pgrx::prelude::*;

mod storage;

// Register this crate as a PostgreSQL extension
pgrx::pg_module_magic!();

// Load schema definitions (tables, indexes) when extension is created
extension_sql_file!("schema.sql", name = "schema", bootstrap);

/// Simple health check function to verify the extension is loaded
#[pg_extern]
fn fhir_ext_version() -> &'static str {
    "fhir-pg-ext 0.1.0"
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use super::*;

    #[pg_test]
    fn test_version() {
        assert_eq!(fhir_ext_version(), "fhir-pg-ext 0.1.0");
    }
}

/// Required by PGRX for extension packaging
#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {
        // No setup needed for now
    }

    #[must_use]
    pub fn postgresql_conf_options() -> Vec<&'static str> {
        vec![]
    }
}
