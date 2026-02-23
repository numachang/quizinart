use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use color_eyre::Result;
use ulid::Ulid;

use super::models::AuthUser;
use super::Db;
use crate::services::auth::AuthRepository;

impl AuthRepository for Db {
    async fn email_exists(&self, email: &str) -> Result<bool> {
        self.email_exists(email).await
    }

    async fn create_user(&self, email: &str, password: &str, display_name: &str) -> Result<i32> {
        self.create_user(email, password, display_name).await
    }

    async fn create_user_session(&self, user_id: i32) -> Result<String> {
        self.create_user_session(user_id).await
    }

    async fn create_unverified_user(
        &self,
        email: &str,
        password: &str,
        display_name: &str,
    ) -> Result<(i32, String)> {
        self.create_unverified_user(email, password, display_name)
            .await
    }

    async fn verify_user_password(&self, email: &str, password: &str) -> Result<bool> {
        self.verify_user_password(email, password).await
    }

    async fn is_email_verified(&self, email: &str) -> Result<bool> {
        self.is_email_verified(email).await
    }

    async fn find_user_by_email(&self, email: &str) -> Result<Option<AuthUser>> {
        self.find_user_by_email(email).await
    }

    async fn delete_user_session(&self, session_id: &str) -> Result<()> {
        self.delete_user_session(session_id).await
    }

    async fn verify_email_token(&self, token: &str) -> Result<bool> {
        self.verify_email_token(token).await
    }

    async fn regenerate_verification_token(&self, email: &str) -> Result<Option<String>> {
        self.regenerate_verification_token(email).await
    }

    async fn create_password_reset_token(&self, email: &str) -> Result<Option<String>> {
        self.create_password_reset_token(email).await
    }

    async fn validate_password_reset_token(&self, token: &str) -> Result<Option<String>> {
        self.validate_password_reset_token(token).await
    }

    async fn reset_password_with_token(&self, token: &str, new_password: &str) -> Result<bool> {
        self.reset_password_with_token(token, new_password).await
    }

    async fn change_password(
        &self,
        user_id: i32,
        current_password: &str,
        new_password: &str,
    ) -> Result<bool> {
        self.change_password(user_id, current_password, new_password)
            .await
    }
}

impl Db {
    pub async fn create_user(
        &self,
        email: &str,
        password: &str,
        display_name: &str,
    ) -> Result<i32> {
        let password_hash = hash_password(password)?;

        let user_id: i32 = sqlx::query_scalar!(
            "INSERT INTO users (email, password_hash, display_name) VALUES ($1, $2, $3) RETURNING id",
            email,
            password_hash,
            display_name
        )
        .fetch_one(&self.pool)
        .await?;

        tracing::info!("new user created: id={user_id}, email={email}");
        Ok(user_id)
    }

    pub async fn find_user_by_email(&self, email: &str) -> Result<Option<AuthUser>> {
        let user = sqlx::query_as!(
            AuthUser,
            "SELECT id, email, display_name FROM users WHERE email = $1",
            email
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn verify_user_password(&self, email: &str, password: &str) -> Result<bool> {
        let stored_hash: Option<String> =
            sqlx::query_scalar!("SELECT password_hash FROM users WHERE email = $1", email)
                .fetch_optional(&self.pool)
                .await?;

        // Always run password verification to prevent timing-based user enumeration.
        // When the user doesn't exist, verify against a dummy hash so the response
        // time is indistinguishable from a real (but wrong) password check.
        let hash = stored_hash.unwrap_or_else(|| {
            "$argon2id$v=19$m=19456,t=2,p=1$AAAAAAAAAAAAAAAAAAAAAA$AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string()
        });
        Ok(verify_password(password, &hash))
    }

    pub async fn create_user_session(&self, user_id: i32) -> Result<String> {
        let session = Ulid::new().to_string();

        sqlx::query!(
            "INSERT INTO user_sessions (id, user_id) VALUES ($1, $2)",
            session,
            user_id
        )
        .execute(&self.pool)
        .await?;

        tracing::info!("new user session created for user_id={user_id}");
        Ok(session)
    }

    pub async fn get_user_by_session(&self, session_id: &str) -> Result<Option<AuthUser>> {
        let user = sqlx::query_as!(
            AuthUser,
            r#"
            SELECT u.id, u.email, u.display_name
            FROM user_sessions s
            JOIN users u ON u.id = s.user_id
            WHERE s.id = $1
            "#,
            session_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn delete_user_session(&self, session_id: &str) -> Result<()> {
        sqlx::query!("DELETE FROM user_sessions WHERE id = $1", session_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn email_exists(&self, email: &str) -> Result<bool> {
        let exists: bool =
            sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)", email)
                .fetch_one(&self.pool)
                .await?
                .unwrap_or(false);

        Ok(exists)
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

        let user_id: i32 = sqlx::query_scalar!(
            r#"INSERT INTO users (email, password_hash, display_name, email_verified, verification_token, token_expires_at)
               VALUES ($1, $2, $3, FALSE, $4, NOW() + INTERVAL '24 hours')
               RETURNING id"#,
            email,
            password_hash,
            display_name,
            token
        )
        .fetch_one(&self.pool)
        .await?;

        tracing::info!("new unverified user created: id={user_id}, email={email}");
        Ok((user_id, token))
    }

    /// Verify a user's email using their verification token.
    /// Returns true if verification succeeded, false if token is invalid/expired.
    pub async fn verify_email_token(&self, token: &str) -> Result<bool> {
        let result = sqlx::query!(
            r#"UPDATE users
               SET email_verified = TRUE, verification_token = NULL, token_expires_at = NULL
               WHERE verification_token = $1 AND token_expires_at > NOW()
               AND email_verified = FALSE"#,
            token
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Check if a user's email is verified.
    pub async fn is_email_verified(&self, email: &str) -> Result<bool> {
        let verified: Option<bool> =
            sqlx::query_scalar!("SELECT email_verified FROM users WHERE email = $1", email)
                .fetch_optional(&self.pool)
                .await?;

        Ok(verified.unwrap_or(false))
    }

    /// Regenerate the verification token for an unverified user. Returns the new token.
    pub async fn regenerate_verification_token(&self, email: &str) -> Result<Option<String>> {
        let token = Ulid::new().to_string();

        let result = sqlx::query!(
            r#"UPDATE users
               SET verification_token = $1, token_expires_at = NOW() + INTERVAL '24 hours'
               WHERE email = $2 AND email_verified = FALSE"#,
            token,
            email
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() > 0 {
            Ok(Some(token))
        } else {
            Ok(None)
        }
    }

    /// Create a password reset token for a verified user. Returns None if email not found or not verified.
    pub async fn create_password_reset_token(&self, email: &str) -> Result<Option<String>> {
        let token = Ulid::new().to_string();

        let result = sqlx::query!(
            r#"UPDATE users
               SET password_reset_token = $1, password_reset_expires_at = NOW() + INTERVAL '1 hour'
               WHERE email = $2 AND email_verified = TRUE"#,
            token,
            email
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() > 0 {
            Ok(Some(token))
        } else {
            Ok(None)
        }
    }

    /// Validate a password reset token. Returns the user's email if valid and not expired.
    pub async fn validate_password_reset_token(&self, token: &str) -> Result<Option<String>> {
        let email: Option<String> = sqlx::query_scalar!(
            r#"SELECT email FROM users
               WHERE password_reset_token = $1
               AND password_reset_expires_at > NOW()"#,
            token
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(email)
    }

    /// Reset a user's password using a valid token. Returns true if successful.
    pub async fn reset_password_with_token(&self, token: &str, new_password: &str) -> Result<bool> {
        let password_hash = hash_password(new_password)?;

        let result = sqlx::query!(
            r#"UPDATE users
               SET password_hash = $1, password_reset_token = NULL, password_reset_expires_at = NULL
               WHERE password_reset_token = $2
               AND password_reset_expires_at > NOW()"#,
            password_hash,
            token
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Change password for an authenticated user. Verifies current password first.
    pub async fn change_password(
        &self,
        user_id: i32,
        current_password: &str,
        new_password: &str,
    ) -> Result<bool> {
        let stored_hash: Option<String> =
            sqlx::query_scalar!("SELECT password_hash FROM users WHERE id = $1", user_id)
                .fetch_optional(&self.pool)
                .await?;

        let stored_hash = match stored_hash {
            Some(h) => h,
            None => return Ok(false),
        };

        if !verify_password(current_password, &stored_hash) {
            return Ok(false);
        }

        let new_hash = hash_password(new_password)?;
        sqlx::query!(
            "UPDATE users SET password_hash = $1 WHERE id = $2",
            new_hash,
            user_id
        )
        .execute(&self.pool)
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
        let user_id: i32 = sqlx::query_scalar!(
            "INSERT INTO users (email, password_hash, display_name) VALUES ($1, $2, $3) RETURNING id",
            email,
            password_hash,
            display_name
        )
        .fetch_one(&self.pool)
        .await?;

        tracing::info!("migration user created: id={user_id}, email={email}");
        Ok(user_id)
    }

    /// Migrate existing admin data to user system (V5 migration logic)
    pub async fn migrate_admin_to_user(&self) -> Result<()> {
        // One-time data migration — kept as runtime queries
        let needs_migration: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM quizzes WHERE owner_id IS NULL)")
                .fetch_one(&self.pool)
                .await?;

        if !needs_migration {
            tracing::info!("admin-to-user migration: no orphan data, skipping");
            return Ok(());
        }

        let admin_pw = self.admin_password().await?;

        let password_hash = match admin_pw {
            Some(pw) => hash_password(&pw)?,
            None => hash_password("admin")?,
        };

        let existing = self.find_user_by_email("admin@local").await?;
        let user_id = match existing {
            Some(user) => user.id,
            None => {
                self.create_user_with_hash("admin@local", &password_hash, "Admin")
                    .await?
            }
        };

        sqlx::query("UPDATE quizzes SET owner_id = $1 WHERE owner_id IS NULL")
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        // Ensure migrated quizzes are in user_quizzes library
        sqlx::query(
            "INSERT INTO user_quizzes (user_id, quiz_id) SELECT $1, id FROM quizzes WHERE owner_id = $1 ON CONFLICT DO NOTHING",
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        sqlx::query("UPDATE quiz_sessions SET user_id = $1 WHERE user_id IS NULL")
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        tracing::info!("admin-to-user migration complete: assigned to user_id={user_id}");
        Ok(())
    }

    /// Backfill public_id (ULID) for quizzes that don't have one yet.
    pub async fn backfill_quiz_public_ids(&self) -> Result<()> {
        // One-time data migration — kept as runtime queries
        let orphan_ids: Vec<i32> =
            sqlx::query_scalar("SELECT id FROM quizzes WHERE public_id IS NULL")
                .fetch_all(&self.pool)
                .await?;

        if orphan_ids.is_empty() {
            return Ok(());
        }

        for quiz_id in &orphan_ids {
            let public_id = Ulid::new().to_string();
            sqlx::query("UPDATE quizzes SET public_id = $1 WHERE id = $2")
                .bind(&public_id)
                .bind(quiz_id)
                .execute(&self.pool)
                .await?;
        }

        tracing::info!("backfilled public_id for {} quizzes", orphan_ids.len());
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
