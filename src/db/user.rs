use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use color_eyre::{eyre::OptionExt, Result};
use libsql::params;
use ulid::Ulid;

use super::models::AuthUser;
use super::Db;

impl Db {
    pub async fn create_user(
        &self,
        email: &str,
        password: &str,
        display_name: &str,
    ) -> Result<i32> {
        let password_hash = hash_password(password)?;
        let conn = self.db.connect()?;

        let user_id = conn
            .query(
                "INSERT INTO users (email, password_hash, display_name) VALUES (?, ?, ?) RETURNING id",
                params![email, password_hash, display_name],
            )
            .await?
            .next()
            .await?
            .ok_or_eyre("could not get user id")?
            .get::<i32>(0)?;

        tracing::info!("new user created: id={user_id}, email={email}");
        Ok(user_id)
    }

    pub async fn find_user_by_email(&self, email: &str) -> Result<Option<AuthUser>> {
        let conn = self.db.connect()?;
        let row = conn
            .query(
                "SELECT id, email, display_name FROM users WHERE email = ?",
                params![email],
            )
            .await?
            .next()
            .await?;

        match row {
            Some(row) => Ok(Some(libsql::de::from_row::<AuthUser>(&row)?)),
            None => Ok(None),
        }
    }

    pub async fn verify_user_password(&self, email: &str, password: &str) -> Result<bool> {
        let conn = self.db.connect()?;
        let row = conn
            .query(
                "SELECT password_hash FROM users WHERE email = ?",
                params![email],
            )
            .await?
            .next()
            .await?;

        match row {
            Some(row) => {
                let stored_hash = row.get::<String>(0)?;
                Ok(verify_password(password, &stored_hash))
            }
            None => Ok(false),
        }
    }

    pub async fn create_user_session(&self, user_id: i32) -> Result<String> {
        let session = Ulid::new().to_string();
        let conn = self.db.connect()?;

        conn.execute(
            "INSERT INTO user_sessions (id, user_id) VALUES (?, ?)",
            params![session.clone(), user_id],
        )
        .await?;

        tracing::info!("new user session created for user_id={user_id}");
        Ok(session)
    }

    pub async fn get_user_by_session(&self, session_id: &str) -> Result<Option<AuthUser>> {
        let conn = self.db.connect()?;
        let row = conn
            .query(
                r#"
                SELECT u.id, u.email, u.display_name
                FROM user_sessions s
                JOIN users u ON u.id = s.user_id
                WHERE s.id = ?
                "#,
                params![session_id],
            )
            .await?
            .next()
            .await?;

        match row {
            Some(row) => Ok(Some(libsql::de::from_row::<AuthUser>(&row)?)),
            None => Ok(None),
        }
    }

    pub async fn delete_user_session(&self, session_id: &str) -> Result<()> {
        let conn = self.db.connect()?;
        conn.execute(
            "DELETE FROM user_sessions WHERE id = ?",
            params![session_id],
        )
        .await?;
        Ok(())
    }

    pub async fn email_exists(&self, email: &str) -> Result<bool> {
        let conn = self.db.connect()?;
        let row = conn
            .query("SELECT 1 FROM users WHERE email = ?", params![email])
            .await?
            .next()
            .await?;
        Ok(row.is_some())
    }

    /// Create a new user with email_verified = false and a verification token.
    /// Returns (user_id, token).
    pub async fn create_unverified_user(
        &self,
        email: &str,
        password: &str,
        display_name: &str,
    ) -> Result<(i32, String)> {
        let password_hash = hash_password(password)?;
        let token = Ulid::new().to_string();
        let conn = self.db.connect()?;

        let user_id = conn
            .query(
                r#"INSERT INTO users (email, password_hash, display_name, email_verified, verification_token, token_expires_at)
                   VALUES (?, ?, ?, FALSE, ?, datetime('now', '+24 hours'))
                   RETURNING id"#,
                params![email, password_hash, display_name, token.clone()],
            )
            .await?
            .next()
            .await?
            .ok_or_eyre("could not get user id")?
            .get::<i32>(0)?;

        tracing::info!("new unverified user created: id={user_id}, email={email}");
        Ok((user_id, token))
    }

    /// Verify a user's email using their verification token.
    /// Returns true if verification succeeded, false if token is invalid/expired.
    pub async fn verify_email_token(&self, token: &str) -> Result<bool> {
        let conn = self.db.connect()?;
        let affected = conn
            .execute(
                r#"UPDATE users
                   SET email_verified = TRUE, verification_token = NULL, token_expires_at = NULL
                   WHERE verification_token = ? AND token_expires_at > datetime('now')
                   AND email_verified = FALSE"#,
                params![token],
            )
            .await?;

        Ok(affected > 0)
    }

    /// Check if a user's email is verified.
    pub async fn is_email_verified(&self, email: &str) -> Result<bool> {
        let conn = self.db.connect()?;
        let row = conn
            .query(
                "SELECT email_verified FROM users WHERE email = ?",
                params![email],
            )
            .await?
            .next()
            .await?;

        match row {
            Some(row) => Ok(row.get::<bool>(0)?),
            None => Ok(false),
        }
    }

    /// Regenerate the verification token for an unverified user. Returns the new token.
    pub async fn regenerate_verification_token(&self, email: &str) -> Result<Option<String>> {
        let token = Ulid::new().to_string();
        let conn = self.db.connect()?;
        let affected = conn
            .execute(
                r#"UPDATE users
                   SET verification_token = ?, token_expires_at = datetime('now', '+24 hours')
                   WHERE email = ? AND email_verified = FALSE"#,
                params![token.clone(), email],
            )
            .await?;

        if affected > 0 {
            Ok(Some(token))
        } else {
            Ok(None)
        }
    }

    /// Create a password reset token for a verified user. Returns None if email not found or not verified.
    pub async fn create_password_reset_token(&self, email: &str) -> Result<Option<String>> {
        let token = Ulid::new().to_string();
        let conn = self.db.connect()?;
        let affected = conn
            .execute(
                r#"UPDATE users
                   SET password_reset_token = ?, password_reset_expires_at = datetime('now', '+1 hour')
                   WHERE email = ? AND email_verified = TRUE"#,
                params![token.clone(), email],
            )
            .await?;

        if affected > 0 {
            Ok(Some(token))
        } else {
            Ok(None)
        }
    }

    /// Validate a password reset token. Returns the user's email if valid and not expired.
    pub async fn validate_password_reset_token(&self, token: &str) -> Result<Option<String>> {
        let conn = self.db.connect()?;
        let row = conn
            .query(
                r#"SELECT email FROM users
                   WHERE password_reset_token = ?
                   AND password_reset_expires_at > datetime('now')"#,
                params![token],
            )
            .await?
            .next()
            .await?;

        match row {
            Some(row) => Ok(Some(row.get::<String>(0)?)),
            None => Ok(None),
        }
    }

    /// Reset a user's password using a valid token. Returns true if successful.
    pub async fn reset_password_with_token(&self, token: &str, new_password: &str) -> Result<bool> {
        let password_hash = hash_password(new_password)?;
        let conn = self.db.connect()?;
        let affected = conn
            .execute(
                r#"UPDATE users
                   SET password_hash = ?, password_reset_token = NULL, password_reset_expires_at = NULL
                   WHERE password_reset_token = ?
                   AND password_reset_expires_at > datetime('now')"#,
                params![password_hash, token],
            )
            .await?;

        Ok(affected > 0)
    }

    /// Change password for an authenticated user. Verifies current password first.
    pub async fn change_password(
        &self,
        user_id: i32,
        current_password: &str,
        new_password: &str,
    ) -> Result<bool> {
        let conn = self.db.connect()?;
        let row = conn
            .query(
                "SELECT password_hash FROM users WHERE id = ?",
                params![user_id],
            )
            .await?
            .next()
            .await?;

        let stored_hash = match row {
            Some(row) => row.get::<String>(0)?,
            None => return Ok(false),
        };

        if !verify_password(current_password, &stored_hash) {
            return Ok(false);
        }

        let new_hash = hash_password(new_password)?;
        conn.execute(
            "UPDATE users SET password_hash = ? WHERE id = ?",
            params![new_hash, user_id],
        )
        .await?;

        Ok(true)
    }

    /// Create a user with a pre-hashed password (for migration from admin table)
    pub async fn create_user_with_hash(
        &self,
        email: &str,
        password_hash: &str,
        display_name: &str,
    ) -> Result<i32> {
        let conn = self.db.connect()?;

        let user_id = conn
            .query(
                "INSERT INTO users (email, password_hash, display_name) VALUES (?, ?, ?) RETURNING id",
                params![email, password_hash, display_name],
            )
            .await?
            .next()
            .await?
            .ok_or_eyre("could not get user id")?
            .get::<i32>(0)?;

        tracing::info!("migration user created: id={user_id}, email={email}");
        Ok(user_id)
    }

    /// Migrate existing admin data to user system (V5 migration logic)
    pub async fn migrate_admin_to_user(&self) -> Result<()> {
        let conn = self.db.connect()?;

        // Check if migration is needed (any quizzes without user_id)
        let needs_migration = conn
            .query("SELECT 1 FROM quizzes WHERE user_id IS NULL LIMIT 1", ())
            .await?
            .next()
            .await?
            .is_some();

        if !needs_migration {
            tracing::info!("admin-to-user migration: no orphan data, skipping");
            return Ok(());
        }

        // Check if admin password exists
        let admin_pw = self.admin_password().await?;

        let password_hash = match admin_pw {
            Some(pw) => hash_password(&pw)?,
            None => hash_password("admin")?, // fallback password
        };

        // Check if migration user already exists
        let existing = self.find_user_by_email("admin@local").await?;
        let user_id = match existing {
            Some(user) => user.id,
            None => {
                self.create_user_with_hash("admin@local", &password_hash, "Admin")
                    .await?
            }
        };

        // Assign orphan quizzes and sessions to the migration user
        conn.execute(
            "UPDATE quizzes SET user_id = ? WHERE user_id IS NULL",
            params![user_id],
        )
        .await?;

        conn.execute(
            "UPDATE quiz_sessions SET user_id = ? WHERE user_id IS NULL",
            params![user_id],
        )
        .await?;

        tracing::info!("admin-to-user migration complete: assigned to user_id={user_id}");
        Ok(())
    }
}

/// Run argon2 hashing on a dedicated thread with a large stack to avoid
/// stack overflow in debug builds.
fn hash_password(password: &str) -> Result<String> {
    let password = password.to_string();
    std::thread::Builder::new()
        .stack_size(4 * 1024 * 1024) // 4 MB stack
        .spawn(move || {
            let salt = SaltString::generate(&mut OsRng);
            let argon2 = Argon2::default();
            argon2
                .hash_password(password.as_bytes(), &salt)
                .map(|h| h.to_string())
                .map_err(|e| color_eyre::eyre::eyre!("failed to hash password: {e}"))
        })?
        .join()
        .map_err(|_| color_eyre::eyre::eyre!("hash thread panicked"))?
}

fn verify_password(password: &str, hash: &str) -> bool {
    let password = password.to_string();
    let hash = hash.to_string();
    std::thread::Builder::new()
        .stack_size(4 * 1024 * 1024)
        .spawn(move || {
            let parsed_hash = match PasswordHash::new(&hash) {
                Ok(h) => h,
                Err(_) => return false,
            };
            Argon2::default()
                .verify_password(password.as_bytes(), &parsed_hash)
                .is_ok()
        })
        .map(|h| h.join().unwrap_or(false))
        .unwrap_or(false)
}
