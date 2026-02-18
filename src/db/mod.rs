// Database module - provides data access layer

use color_eyre::{eyre::OptionExt, Result};
use std::sync::Arc;

// Re-export models for convenience
pub mod models;
pub use models::*;

// Internal modules
mod admin;
mod answer;
mod migrations;
mod question;
mod quiz;
mod report;
mod session;

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
            libsql::Builder::new_local(path).build().await?
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

        // Run schema migrations
        migrations::run(&conn).await?;

        tracing::info!("database connection has been verified");

        Ok(Self { db: Arc::new(db) })
    }

    pub async fn migration_applied(&self, version: &str) -> Result<bool> {
        let conn = self.db.connect()?;
        let exists = conn
            .query(
                "SELECT version FROM schema_migrations WHERE version = ?",
                libsql::params![version],
            )
            .await?
            .next()
            .await?
            .is_some();

        Ok(exists)
    }
}
