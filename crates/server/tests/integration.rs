//! Integration tests for the FHIR Patient Server.
//!
//! These tests spin up a real PostgreSQL container (with the PGRX extension)
//! via testcontainers and exercise the HTTP endpoints through the Axum router.
//!
//! Prerequisites:
//!   make test-db-image   (builds the fhir-pg-test Docker image)

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use deadpool_postgres::{Config as PgConfig, Pool, Runtime};
use http_body_util::BodyExt;
use serde_json::Value as JsonValue;
use testcontainers::{
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
    ContainerAsync, GenericImage, ImageExt,
};
use tokio_postgres::NoTls;
use tower::ServiceExt;

use fhir_server::config::Config;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const TEST_API_KEY: &str = "test-secret-key";

/// Start a PostgreSQL container with the PGRX extension pre-installed.
async fn start_db() -> (ContainerAsync<GenericImage>, Pool) {
    let image = GenericImage::new("fhir-pg-test", "latest")
        .with_exposed_port(5432.tcp())
        .with_wait_for(WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        ))
        .with_env_var("POSTGRES_USER", "fhir")
        .with_env_var("POSTGRES_PASSWORD", "fhir")
        .with_env_var("POSTGRES_DB", "fhir");

    let container = image.start().await.expect("Failed to start test database");

    let port = container
        .get_host_port_ipv4(5432)
        .await
        .expect("Failed to get mapped port");

    let database_url = format!("postgres://fhir:fhir@127.0.0.1:{}/fhir", port);

    // Create connection pool
    let mut cfg = PgConfig::new();
    cfg.url = Some(database_url);
    let pool = cfg
        .create_pool(Some(Runtime::Tokio1), NoTls)
        .expect("Failed to create pool");

    // Wait for the pool to be ready and the extension to be loaded
    let mut retries = 0;
    loop {
        match pool.get().await {
            Ok(client) => {
                // Verify extension is loaded
                match client
                    .query_one("SELECT fhir_ext_version()", &[])
                    .await
                {
                    Ok(_) => break,
                    Err(e) => {
                        if retries >= 30 {
                            panic!("Extension not loaded after 30 retries: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                if retries >= 30 {
                    panic!("Database not ready after 30 retries: {}", e);
                }
            }
        }
        retries += 1;
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

    (container, pool)
}

/// Build the app router with test configuration.
fn test_app(pool: Pool) -> Router {
    let config = Config {
        database_url: String::new(), // unused — pool is already created
        bind_address: "0.0.0.0:0".to_string(),
        api_key: Some(TEST_API_KEY.to_string()),
        cors_origins: vec!["*".to_string()],
        rate_limit_rps: 1000,
        anthropic_api_key: None,
    };
    fhir_server::build_app(pool, &config)
}

/// Send a request to the app and return (status, body as JSON).
async fn request(app: &Router, req: Request<Body>) -> (StatusCode, JsonValue) {
    let response = app.clone().oneshot(req).await.expect("Request failed");
    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("Failed to read body")
        .to_bytes();

    let body = if bytes.is_empty() {
        JsonValue::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(JsonValue::Null)
    };

    (status, body)
}

/// Build a GET request with auth header.
fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(uri)
        .header("X-API-Key", TEST_API_KEY)
        .body(Body::empty())
        .unwrap()
}

/// Build a POST request with JSON body and auth header.
fn post(uri: &str, body: JsonValue) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header("Content-Type", "application/json")
        .header("X-API-Key", TEST_API_KEY)
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap()
}

/// Build a PUT request with JSON body and auth header.
fn put(uri: &str, body: JsonValue) -> Request<Body> {
    Request::builder()
        .method("PUT")
        .uri(uri)
        .header("Content-Type", "application/json")
        .header("X-API-Key", TEST_API_KEY)
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap()
}

/// Build a DELETE request with auth header.
fn delete(uri: &str) -> Request<Body> {
    Request::builder()
        .method("DELETE")
        .uri(uri)
        .header("X-API-Key", TEST_API_KEY)
        .body(Body::empty())
        .unwrap()
}

/// Helper: create a patient and return its UUID extracted from Location header.
async fn create_patient(app: &Router, patient: JsonValue) -> String {
    let response = app
        .clone()
        .oneshot(post("/fhir/Patient", patient))
        .await
        .expect("Create request failed");

    assert_eq!(response.status(), StatusCode::CREATED);

    let location = response
        .headers()
        .get("Location")
        .expect("Missing Location header")
        .to_str()
        .unwrap()
        .to_string();

    // Extract UUID from "/fhir/Patient/{id}"
    location.rsplit('/').next().unwrap().to_string()
}

/// Sample patient JSON for tests.
fn sample_patient(family: &str, given: &str, gender: &str, birth_date: &str) -> JsonValue {
    serde_json::json!({
        "resourceType": "Patient",
        "name": [{"family": family, "given": [given]}],
        "gender": gender,
        "birthDate": birth_date
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_metadata() {
    let (_container, pool) = start_db().await;
    let app = test_app(pool);

    // /metadata is a public route — no auth needed
    let req = Request::builder()
        .method("GET")
        .uri("/metadata")
        .body(Body::empty())
        .unwrap();

    let (status, body) = request(&app, req).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["resourceType"], "CapabilityStatement");
    assert_eq!(body["fhirVersion"], "4.3.0");
    assert_eq!(body["status"], "active");
}

#[tokio::test]
async fn test_health() {
    let (_container, pool) = start_db().await;
    let app = test_app(pool);

    let req = Request::builder()
        .method("GET")
        .uri("/health")
        .body(Body::empty())
        .unwrap();

    let (status, body) = request(&app, req).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "healthy");
}

#[tokio::test]
async fn test_crud_lifecycle() {
    let (_container, pool) = start_db().await;
    let app = test_app(pool);

    // 1. Create
    let patient = sample_patient("Smith", "John", "male", "1990-05-15");
    let id = create_patient(&app, patient).await;

    // 2. Read
    let (status, body) = request(&app, get(&format!("/fhir/Patient/{}", id))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["name"][0]["family"], "Smith");
    assert_eq!(body["gender"], "male");

    // 3. Update
    let updated = sample_patient("Smith", "John Michael", "male", "1990-05-15");
    let response = app
        .clone()
        .oneshot(put(&format!("/fhir/Patient/{}", id), updated))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Verify ETag changed
    let etag = response
        .headers()
        .get("ETag")
        .expect("Missing ETag")
        .to_str()
        .unwrap();
    assert!(etag.contains("2"), "ETag should reflect version 2");

    // 4. Read after update
    let (status, body) = request(&app, get(&format!("/fhir/Patient/{}", id))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["name"][0]["given"][0], "John Michael");

    // 5. Delete
    let (status, _) = request(&app, delete(&format!("/fhir/Patient/{}", id))).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // 6. Read after delete → 404
    let (status, body) = request(&app, get(&format!("/fhir/Patient/{}", id))).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["resourceType"], "OperationOutcome");
}

#[tokio::test]
async fn test_search() {
    let (_container, pool) = start_db().await;
    let app = test_app(pool);

    // Create 3 patients
    create_patient(
        &app,
        sample_patient("Zhang", "Wei", "male", "1985-03-10"),
    )
    .await;
    create_patient(
        &app,
        sample_patient("Garcia", "Maria", "female", "1995-07-22"),
    )
    .await;
    create_patient(
        &app,
        sample_patient("Zhang", "Li", "female", "2000-01-01"),
    )
    .await;

    // Search by name
    let (status, body) = request(&app, get("/fhir/Patient?name=Zhang")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["resourceType"], "Bundle");
    assert_eq!(body["type"], "searchset");
    assert_eq!(body["total"], 2);

    // Search by gender
    let (status, body) = request(&app, get("/fhir/Patient?gender=female")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], 2);

    // Search by birthdate range
    let (status, body) = request(&app, get("/fhir/Patient?birthdate=ge1990-01-01")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], 2); // Garcia (1995) and Zhang Li (2000)

    // Combined search
    let (status, body) = request(
        &app,
        get("/fhir/Patient?name=Zhang&gender=female"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], 1);
}

#[tokio::test]
async fn test_pagination() {
    let (_container, pool) = start_db().await;
    let app = test_app(pool);

    // Create 3 patients
    for i in 0..3 {
        create_patient(
            &app,
            sample_patient(&format!("Page{}", i), "Test", "male", "1990-01-01"),
        )
        .await;
    }

    // Request page 1 (count=1)
    let (status, body) = request(&app, get("/fhir/Patient?_count=1&_offset=0")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], 3);
    assert_eq!(body["entry"].as_array().unwrap().len(), 1);

    // Verify pagination links exist
    let links = body["link"].as_array().unwrap();
    let relations: Vec<&str> = links
        .iter()
        .map(|l| l["relation"].as_str().unwrap())
        .collect();
    assert!(relations.contains(&"self"));
    assert!(relations.contains(&"next"));

    // Request page 2
    let (status, body) = request(&app, get("/fhir/Patient?_count=1&_offset=1")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["entry"].as_array().unwrap().len(), 1);
    let relations: Vec<&str> = body["link"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| l["relation"].as_str().unwrap())
        .collect();
    assert!(relations.contains(&"previous"));
}

#[tokio::test]
async fn test_history() {
    let (_container, pool) = start_db().await;
    let app = test_app(pool);

    // Create a patient
    let patient = sample_patient("Doe", "Jane", "female", "1988-12-01");
    let id = create_patient(&app, patient).await;

    // Update it
    let updated = sample_patient("Doe", "Jane Marie", "female", "1988-12-01");
    let (status, _) = request(&app, put(&format!("/fhir/Patient/{}", id), updated)).await;
    assert_eq!(status, StatusCode::OK);

    // Get history
    let (status, body) = request(
        &app,
        get(&format!("/fhir/Patient/{}/_history", id)),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["resourceType"], "Bundle");
    assert_eq!(body["type"], "history");
    assert_eq!(body["total"], 2);

    // Entries should be ordered newest first (version 2, then version 1)
    let entries = body["entry"].as_array().unwrap();
    assert_eq!(entries.len(), 2);
}

#[tokio::test]
async fn test_validate() {
    let (_container, pool) = start_db().await;
    let app = test_app(pool);

    // Valid patient
    let valid = serde_json::json!({"resourceType": "Patient", "name": [{"family": "Test"}]});
    let (status, body) = request(&app, post("/fhir/Patient/$validate", valid)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["resourceType"], "OperationOutcome");
    assert_eq!(body["issue"][0]["severity"], "information");

    // Invalid — wrong resourceType
    let invalid = serde_json::json!({"resourceType": "Observation"});
    let (status, body) = request(&app, post("/fhir/Patient/$validate", invalid)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["resourceType"], "OperationOutcome");
    assert_eq!(body["issue"][0]["severity"], "error");

    // Invalid — missing resourceType
    let missing = serde_json::json!({"name": [{"family": "Test"}]});
    let (status, _) = request(&app, post("/fhir/Patient/$validate", missing)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_auth() {
    let (_container, pool) = start_db().await;
    let app = test_app(pool);

    // No API key → 401
    let req = Request::builder()
        .method("GET")
        .uri("/fhir/Patient")
        .body(Body::empty())
        .unwrap();
    let (status, body) = request(&app, req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["resourceType"], "OperationOutcome");

    // Wrong API key → 401
    let req = Request::builder()
        .method("GET")
        .uri("/fhir/Patient")
        .header("X-API-Key", "wrong-key")
        .body(Body::empty())
        .unwrap();
    let (status, _) = request(&app, req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // Correct API key → 200
    let (status, _) = request(&app, get("/fhir/Patient")).await;
    assert_eq!(status, StatusCode::OK);
}
