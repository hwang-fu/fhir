//! fhir-server: FHIR R4 HTTP Server
//!
//! An Axum-based HTTP server implementing FHIR R4 Patient resource endpoints.

fn main() {
    println!("{}", fhir_core::hello_core());
    println!("FHIR Server starting...");
}
