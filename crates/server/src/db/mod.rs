mod repository;

pub use repository::PatientRepository;

use deadpool_postgres::{Config, Pool, Runtime};
use tokio_postgres::NoTls;

/// Create a connection pool from a database URL
pub async fn create_pool(database_url: &str) -> Result<Pool, deadpool_postgres::CreatePoolError> {
    let mut cfg = Config::new();
    cfg.url = Some(database_url.to_string());
    cfg.create_pool(Some(Runtime::Tokio1), NoTls)
}
