//! Server configuration

/// Server configuration loaded from environment variables
pub struct Config {
    pub database_url: String,
    pub bind_address: String,
    pub api_key: Option<String>,
    pub cors_origins: Vec<String>,
    pub rate_limit_rps: u32,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let cors_origins = std::env::var("CORS_ORIGINS")
            .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_else(|_| vec!["*".to_string()]);

        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "host=localhost user=postgres dbname=fhir".into());

        let bind_address = std::env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0:8080".into());

        let api_key = std::env::var("API_KEY").ok();

        let rate_limit_rps = std::env::var("RATE_LIMIT_RPS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(100);

        Self {
            database_url,
            bind_address,
            api_key,
            cors_origins,
            rate_limit_rps,
        }
    }
}
