//! Server configuration

/// Server configuration loaded from environment variables
pub struct Config {
    pub database_url: String,
    pub bind_address: String,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "host=localhost user=postgres dbname=fhir".into()),
            bind_address: std::env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0:8080".into()),
        }
    }
}
