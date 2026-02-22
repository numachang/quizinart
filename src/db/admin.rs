use color_eyre::Result;
use ulid::Ulid;

use super::Db;

impl Db {
    pub async fn admin_password(&self) -> Result<Option<String>> {
        let password: Option<String> =
            sqlx::query_scalar("SELECT password FROM admin WHERE id = 1")
                .fetch_optional(&self.pool)
                .await?;

        Ok(password)
    }

    pub async fn set_admin_password(&self, password: String) -> Result<()> {
        sqlx::query(
            "INSERT INTO admin (id, password) VALUES (1, $1) ON CONFLICT(id) DO UPDATE SET password = EXCLUDED.password"
        )
        .bind(&password)
        .execute(&self.pool)
        .await?;

        tracing::info!("new admin password set");
        Ok(())
    }

    pub async fn create_admin_session(&self) -> Result<String> {
        let session = Ulid::new().to_string();

        sqlx::query("INSERT INTO admin_sessions (id) VALUES ($1)")
            .bind(&session)
            .execute(&self.pool)
            .await?;

        tracing::info!("new admin session {session:?} created");
        Ok(session)
    }

    pub async fn admin_session_exists(&self, session: String) -> Result<bool> {
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM admin_sessions WHERE id = $1)")
                .bind(&session)
                .fetch_one(&self.pool)
                .await?;

        tracing::info!("admin session {session:?} exists: {exists}");
        Ok(exists)
    }
}
