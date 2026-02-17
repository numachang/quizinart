// Database module - provides data access layer

use std::sync::Arc;
use color_eyre::{eyre::OptionExt, Result};

// Re-export models for convenience
pub mod models;
pub use models::*;

// Internal modules
mod schema;
mod admin;
mod quiz;
mod session;
mod question;
mod answer;
mod report;

// Main database handle
#[derive(Clone)]
pub struct Db {
    db: Arc<libsql::Database>,
}

impl Db {
    pub async fn new(url: String, auth_token: String) -> Result<Self> {
        let db = if url.starts_with("file:") {
            // Local SQLite file
            let path = url.strip_prefix("file:").unwrap_or(&url);
            libsql::Builder::new_local(path)
                .build()
                .await?
        } else {
            // Remote Turso database
            libsql::Builder::new_remote(url.to_owned(), auth_token)
                .build()
                .await?
        };

        let conn = db.connect()?;

        // Verify connection
        let one = conn
            .query("SELECT 1", ())
            .await?
            .next()
            .await?
            .ok_or_eyre("connection check failed")?
            .get::<i32>(0)?;
        assert_eq!(one, 1);

        // Initialize schema
        schema::create_schema(&conn).await?;

        tracing::info!("database connection has been verified");

        Ok(Self { db: Arc::new(db) })
    }
}
