use color_eyre::Result;
use libsql::params;
use ulid::Ulid;

use super::Db;

impl Db {
    pub async fn admin_password(&self) -> Result<Option<String>> {
        let conn = self.db.connect()?;
        let query = conn
            .query("SELECT password FROM admin WHERE id = 1", ())
            .await?
            .next()
            .await?;

        Ok(match query {
            Some(row) => Some(row.get::<String>(0)?),
            None => None,
        })
    }

    pub async fn set_admin_password(&self, password: String) -> Result<()> {
        let conn = self.db.connect()?;

        let rows = conn
            .execute("INSERT INTO admin (password) VALUES (?)", params![password])
            .await?;

        tracing::info!("new admin password set: {rows:?}");
        Ok(())
    }

    pub async fn create_admin_session(&self) -> Result<String> {
        let session = Ulid::new().to_string();
        let conn = self.db.connect()?;

        let rows = conn
            .execute(
                "INSERT INTO admin_sessions (id) VALUES (?)",
                params![session.clone()],
            )
            .await?;

        tracing::info!("new admin session {session:?} created : {rows:?}");
        Ok(session)
    }

    pub async fn admin_session_exists(&self, session: String) -> Result<bool> {
        let conn = self.db.connect()?;
        let exists = conn
            .query(
                "SELECT id FROM admin_sessions WHERE id = ?",
                params![session.clone()],
            )
            .await?
            .next()
            .await?
            .is_some();

        tracing::info!("admin session {session:?} exists: {exists}");
        Ok(exists)
    }
}
