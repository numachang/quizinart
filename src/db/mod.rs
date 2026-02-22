// Database module - provides data access layer

use color_eyre::Result;

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
    pool: sqlx::PgPool,
}

impl Db {
    pub async fn new(database_url: String) -> Result<Self> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(10)
            .connect(&database_url)
            .await?;

        // Verify connection
        let one: i32 = sqlx::query_scalar("SELECT 1").fetch_one(&pool).await?;
        assert_eq!(one, 1);

        // Run schema migrations
        migrations::run(&pool).await?;

        tracing::info!("database connection has been verified");

        let instance = Self { pool };

        // Run data migration: assign orphan quizzes/sessions to a default user
        instance.migrate_admin_to_user().await?;

        Ok(instance)
    }

    pub async fn migration_applied(&self, version: &str) -> Result<bool> {
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = $1)")
                .bind(version)
                .fetch_one(&self.pool)
                .await?;

        Ok(exists)
    }
}
