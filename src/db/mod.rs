// Database module - provides data access layer

use color_eyre::{eyre::OptionExt, Result};
use std::sync::Arc;

// Re-export models for convenience
pub mod models;
pub use models::*;

// Internal modules
mod admin;
mod answer;
pub mod helpers;
mod migrations;
mod question;
mod quiz;
mod report;
mod session;
mod user;

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

        if url.starts_with("file:") {
            // Enable WAL mode and busy timeout for concurrent access
            conn.query("PRAGMA journal_mode=WAL", ()).await?;
            conn.query("PRAGMA busy_timeout=5000", ()).await?;
        }

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

        let instance = Self { db: Arc::new(db) };

        // Run data migration: assign orphan quizzes/sessions to a default user
        instance.migrate_admin_to_user().await?;

        Ok(instance)
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
