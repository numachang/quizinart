use color_eyre::Result;

use crate::db::models::AuthUser;
use crate::db::Db;
use crate::email::ResendEmailSender;

// ---------------------------------------------------------------------------
// AuthRepository trait (DIP: service defines the abstraction it needs)
// ---------------------------------------------------------------------------

#[cfg_attr(test, mockall::automock)]
pub trait AuthRepository: Send + Sync {
    fn email_exists(&self, email: &str) -> impl std::future::Future<Output = Result<bool>> + Send;

    fn create_user(
        &self,
        email: &str,
        password: &str,
        display_name: &str,
    ) -> impl std::future::Future<Output = Result<i32>> + Send;

    fn create_user_session(
        &self,
        user_id: i32,
    ) -> impl std::future::Future<Output = Result<String>> + Send;

    fn create_unverified_user(
        &self,
        email: &str,
        password: &str,
        display_name: &str,
    ) -> impl std::future::Future<Output = Result<(i32, String)>> + Send;

    fn verify_user_password(
        &self,
        email: &str,
        password: &str,
    ) -> impl std::future::Future<Output = Result<bool>> + Send;

    fn is_email_verified(
        &self,
        email: &str,
    ) -> impl std::future::Future<Output = Result<bool>> + Send;

    fn find_user_by_email(
        &self,
        email: &str,
    ) -> impl std::future::Future<Output = Result<Option<AuthUser>>> + Send;

    fn delete_user_session(
        &self,
        session_id: &str,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    fn verify_email_token(
        &self,
        token: &str,
    ) -> impl std::future::Future<Output = Result<bool>> + Send;

    fn regenerate_verification_token(
        &self,
        email: &str,
    ) -> impl std::future::Future<Output = Result<Option<String>>> + Send;

    fn create_password_reset_token(
        &self,
        email: &str,
    ) -> impl std::future::Future<Output = Result<Option<String>>> + Send;

    fn validate_password_reset_token(
        &self,
        token: &str,
    ) -> impl std::future::Future<Output = Result<Option<String>>> + Send;

    fn reset_password_with_token(
        &self,
        token: &str,
        new_password: &str,
    ) -> impl std::future::Future<Output = Result<bool>> + Send;

    fn change_password(
        &self,
        user_id: i32,
        current_password: &str,
        new_password: &str,
    ) -> impl std::future::Future<Output = Result<bool>> + Send;
}

// ---------------------------------------------------------------------------
// EmailSender trait (DIP: service defines the abstraction it needs)
// ---------------------------------------------------------------------------

#[cfg_attr(test, mockall::automock)]
pub trait EmailSender: Send + Sync {
    /// Whether email sending is configured (false in dev mode).
    fn is_enabled(&self) -> bool;

    fn send_verification_email(
        &self,
        to_email: &str,
        verification_url: &str,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    fn send_password_reset_email(
        &self,
        to_email: &str,
        reset_url: &str,
    ) -> impl std::future::Future<Output = Result<()>> + Send;
}

// ---------------------------------------------------------------------------
// Outcome enums
// ---------------------------------------------------------------------------

pub enum RegisterOutcome {
    /// User created and session started (dev mode, no email verification).
    LoggedIn(String),
    /// Unverified user created, verification email sent (prod mode).
    VerificationSent(String),
    /// Unverified user created, but verification email failed after retries.
    VerificationEmailFailed(String),
    /// Required fields were empty.
    EmptyFields,
    /// Email already in use.
    EmailTaken,
    /// Password does not meet minimum requirements.
    WeakPassword,
}

pub enum LoginOutcome {
    /// Login succeeded. Contains the session token.
    Success(String),
    /// Password was incorrect (or email not found).
    InvalidCredentials,
    /// Credentials correct but email not yet verified.
    EmailNotVerified,
}

pub enum ResetPasswordOutcome {
    Success,
    EmptyPassword,
    WeakPassword,
    InvalidToken,
}

pub enum ChangePasswordOutcome {
    Success,
    EmptyFields,
    WeakPassword,
    IncorrectPassword,
    DemoUser,
}

const MIN_PASSWORD_LENGTH: usize = 8;

// ---------------------------------------------------------------------------
// AuthService
// ---------------------------------------------------------------------------

pub struct AuthService<R: AuthRepository = Db, E: EmailSender = ResendEmailSender> {
    repo: R,
    email: E,
    base_url: String,
}

impl<R: AuthRepository + Clone, E: EmailSender + Clone> Clone for AuthService<R, E> {
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
            email: self.email.clone(),
            base_url: self.base_url.clone(),
        }
    }
}

impl<R: AuthRepository, E: EmailSender> AuthService<R, E> {
    pub fn new(repo: R, email: E, base_url: String) -> Self {
        Self {
            repo,
            email,
            base_url,
        }
    }

    /// Whether email verification is enabled (production mode).
    pub fn email_enabled(&self) -> bool {
        self.email.is_enabled()
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<LoginOutcome> {
        let verified = self.repo.verify_user_password(email, password).await?;

        if !verified {
            return Ok(LoginOutcome::InvalidCredentials);
        }

        if self.email_enabled() {
            let email_verified = self.repo.is_email_verified(email).await?;
            if !email_verified {
                return Ok(LoginOutcome::EmailNotVerified);
            }
        }

        let user =
            self.repo.find_user_by_email(email).await?.ok_or_else(|| {
                color_eyre::eyre::eyre!("user not found after password verification")
            })?;

        let session_token = self.repo.create_user_session(user.id).await?;

        Ok(LoginOutcome::Success(session_token))
    }

    pub async fn register(
        &self,
        email: &str,
        password: &str,
        display_name: &str,
    ) -> Result<RegisterOutcome> {
        if email.is_empty() || password.is_empty() || display_name.is_empty() {
            return Ok(RegisterOutcome::EmptyFields);
        }

        if password.len() < MIN_PASSWORD_LENGTH {
            return Ok(RegisterOutcome::WeakPassword);
        }

        let exists = self.repo.email_exists(email).await?;
        if exists {
            return Ok(RegisterOutcome::EmailTaken);
        }

        if !self.email_enabled() {
            // Dev mode: create user and session immediately
            let user_id = self.repo.create_user(email, password, display_name).await?;
            let session_token = self.repo.create_user_session(user_id).await?;
            return Ok(RegisterOutcome::LoggedIn(session_token));
        }

        // Prod mode: create unverified user and send verification email
        let (_user_id, token) = self
            .repo
            .create_unverified_user(email, password, display_name)
            .await?;

        let verification_url = format!("{}/verify-email/{}", self.base_url, token);

        if let Err(e) = self
            .email
            .send_verification_email(email, &verification_url)
            .await
        {
            tracing::error!("failed to send verification email to {email}: {e}");
            return Ok(RegisterOutcome::VerificationEmailFailed(email.to_string()));
        }

        Ok(RegisterOutcome::VerificationSent(email.to_string()))
    }

    pub async fn logout(&self, session_id: &str) -> Result<()> {
        self.repo.delete_user_session(session_id).await
    }

    pub async fn verify_email(&self, token: &str) -> Result<bool> {
        self.repo.verify_email_token(token).await
    }

    pub async fn resend_verification(&self, email: &str) -> Result<()> {
        let token = self.repo.regenerate_verification_token(email).await?;

        if let Some(token) = token {
            let verification_url = format!("{}/verify-email/{}", self.base_url, token);
            self.email
                .send_verification_email(email, &verification_url)
                .await?;
        }

        Ok(())
    }

    pub async fn forgot_password(&self, email: &str) -> Result<bool> {
        if !self.email_enabled() {
            return Ok(false);
        }

        let token = self.repo.create_password_reset_token(email).await?;

        if let Some(token) = token {
            let reset_url = format!("{}/reset-password/{}", self.base_url, token);
            if let Err(e) = self
                .email
                .send_password_reset_email(email, &reset_url)
                .await
            {
                // Swallow error to avoid leaking whether the email exists.
                tracing::error!("failed to send password reset email to {email}: {e}");
            }
        }

        Ok(true)
    }

    pub async fn validate_reset_token(&self, token: &str) -> Result<bool> {
        let email = self.repo.validate_password_reset_token(token).await?;
        Ok(email.is_some())
    }

    pub async fn reset_password(
        &self,
        token: &str,
        new_password: &str,
    ) -> Result<ResetPasswordOutcome> {
        if new_password.is_empty() {
            return Ok(ResetPasswordOutcome::EmptyPassword);
        }

        if new_password.len() < MIN_PASSWORD_LENGTH {
            return Ok(ResetPasswordOutcome::WeakPassword);
        }

        let success = self
            .repo
            .reset_password_with_token(token, new_password)
            .await?;

        if success {
            Ok(ResetPasswordOutcome::Success)
        } else {
            Ok(ResetPasswordOutcome::InvalidToken)
        }
    }

    pub async fn change_password(
        &self,
        user_id: i32,
        is_demo: bool,
        current_password: &str,
        new_password: &str,
    ) -> Result<ChangePasswordOutcome> {
        if is_demo {
            return Ok(ChangePasswordOutcome::DemoUser);
        }

        if current_password.is_empty() || new_password.is_empty() {
            return Ok(ChangePasswordOutcome::EmptyFields);
        }

        if new_password.len() < MIN_PASSWORD_LENGTH {
            return Ok(ChangePasswordOutcome::WeakPassword);
        }

        let changed = self
            .repo
            .change_password(user_id, current_password, new_password)
            .await?;

        if changed {
            Ok(ChangePasswordOutcome::Success)
        } else {
            Ok(ChangePasswordOutcome::IncorrectPassword)
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn service(mock_repo: MockAuthRepository) -> AuthService<MockAuthRepository, MockEmailSender> {
        let mut mock_email = MockEmailSender::new();
        mock_email.expect_is_enabled().returning(|| false);
        AuthService::new(mock_repo, mock_email, "http://localhost".to_string())
    }

    fn service_with_email(
        mock_repo: MockAuthRepository,
        mock_email: MockEmailSender,
    ) -> AuthService<MockAuthRepository, MockEmailSender> {
        AuthService::new(mock_repo, mock_email, "http://localhost".to_string())
    }

    fn mock_email_ok() -> MockEmailSender {
        let mut mock = MockEmailSender::new();
        mock.expect_is_enabled().returning(|| true);
        mock.expect_send_verification_email()
            .returning(|_, _| Box::pin(async { Ok(()) }));
        mock.expect_send_password_reset_email()
            .returning(|_, _| Box::pin(async { Ok(()) }));
        mock
    }

    fn mock_email_fail() -> MockEmailSender {
        let mut mock = MockEmailSender::new();
        mock.expect_is_enabled().returning(|| true);
        mock.expect_send_verification_email()
            .returning(|_, _| Box::pin(async { Err(color_eyre::eyre::eyre!("send failed")) }));
        mock.expect_send_password_reset_email()
            .returning(|_, _| Box::pin(async { Err(color_eyre::eyre::eyre!("send failed")) }));
        mock
    }

    // ----- login tests -----

    #[tokio::test]
    async fn login_success_returns_session_token() {
        let mut mock = MockAuthRepository::new();
        mock.expect_verify_user_password()
            .returning(|_, _| Box::pin(async { Ok(true) }));
        mock.expect_find_user_by_email().returning(|_| {
            Box::pin(async {
                Ok(Some(AuthUser {
                    id: 1,
                    email: "test@example.com".to_string(),
                    display_name: "Test".to_string(),
                    is_admin: false,
                    is_demo: false,
                }))
            })
        });
        mock.expect_create_user_session()
            .returning(|_| Box::pin(async { Ok("session-token-123".to_string()) }));

        let svc = service(mock);
        let outcome = svc.login("test@example.com", "password").await.unwrap();

        assert!(matches!(outcome, LoginOutcome::Success(ref t) if t == "session-token-123"));
    }

    #[tokio::test]
    async fn login_wrong_password_returns_invalid_credentials() {
        let mut mock = MockAuthRepository::new();
        mock.expect_verify_user_password()
            .returning(|_, _| Box::pin(async { Ok(false) }));

        let svc = service(mock);
        let outcome = svc.login("test@example.com", "wrong").await.unwrap();

        assert!(matches!(outcome, LoginOutcome::InvalidCredentials));
    }

    #[tokio::test]
    async fn login_unverified_email_returns_email_not_verified() {
        let mut mock = MockAuthRepository::new();
        mock.expect_verify_user_password()
            .returning(|_, _| Box::pin(async { Ok(true) }));
        mock.expect_is_email_verified()
            .returning(|_| Box::pin(async { Ok(false) }));

        // email_enabled=true
        let svc = service_with_email(mock, mock_email_ok());
        let outcome = svc.login("test@example.com", "password").await.unwrap();

        assert!(matches!(outcome, LoginOutcome::EmailNotVerified));
    }

    // ----- register tests -----

    #[tokio::test]
    async fn register_empty_fields_returns_empty_fields() {
        let mock = MockAuthRepository::new();
        let svc = service(mock);

        let outcome = svc.register("", "pass", "name").await.unwrap();
        assert!(matches!(outcome, RegisterOutcome::EmptyFields));

        let mock = MockAuthRepository::new();
        let svc = service(mock);
        let outcome = svc.register("a@b.com", "", "name").await.unwrap();
        assert!(matches!(outcome, RegisterOutcome::EmptyFields));

        let mock = MockAuthRepository::new();
        let svc = service(mock);
        let outcome = svc.register("a@b.com", "pass", "").await.unwrap();
        assert!(matches!(outcome, RegisterOutcome::EmptyFields));
    }

    #[tokio::test]
    async fn register_email_taken_returns_email_taken() {
        let mut mock = MockAuthRepository::new();
        mock.expect_email_exists()
            .returning(|_| Box::pin(async { Ok(true) }));

        let svc = service(mock);
        let outcome = svc
            .register("taken@example.com", "password123", "name")
            .await
            .unwrap();

        assert!(matches!(outcome, RegisterOutcome::EmailTaken));
    }

    #[tokio::test]
    async fn register_dev_mode_returns_logged_in() {
        let mut mock = MockAuthRepository::new();
        mock.expect_email_exists()
            .returning(|_| Box::pin(async { Ok(false) }));
        mock.expect_create_user()
            .returning(|_, _, _| Box::pin(async { Ok(1) }));
        mock.expect_create_user_session()
            .returning(|_| Box::pin(async { Ok("session-abc".to_string()) }));

        // service() has email disabled → dev mode
        let svc = service(mock);
        let outcome = svc
            .register("new@example.com", "password123", "Name")
            .await
            .unwrap();

        assert!(matches!(outcome, RegisterOutcome::LoggedIn(ref t) if t == "session-abc"));
    }

    #[tokio::test]
    async fn register_prod_mode_returns_verification_sent() {
        let mut mock = MockAuthRepository::new();
        mock.expect_email_exists()
            .returning(|_| Box::pin(async { Ok(false) }));
        mock.expect_create_unverified_user()
            .returning(|_, _, _| Box::pin(async { Ok((1, "token-xyz".to_string())) }));

        let svc = service_with_email(mock, mock_email_ok());
        let outcome = svc
            .register("new@example.com", "password123", "Name")
            .await
            .unwrap();

        assert!(
            matches!(outcome, RegisterOutcome::VerificationSent(ref e) if e == "new@example.com")
        );
    }

    #[tokio::test]
    async fn register_prod_mode_email_failure_returns_verification_email_failed() {
        let mut mock = MockAuthRepository::new();
        mock.expect_email_exists()
            .returning(|_| Box::pin(async { Ok(false) }));
        mock.expect_create_unverified_user()
            .returning(|_, _, _| Box::pin(async { Ok((1, "token-xyz".to_string())) }));

        let svc = service_with_email(mock, mock_email_fail());
        let outcome = svc
            .register("new@example.com", "password123", "Name")
            .await
            .unwrap();

        assert!(
            matches!(outcome, RegisterOutcome::VerificationEmailFailed(ref e) if e == "new@example.com")
        );
    }

    // ----- logout tests -----

    #[tokio::test]
    async fn logout_deletes_session() {
        let mut mock = MockAuthRepository::new();
        mock.expect_delete_user_session()
            .withf(|id| id == "session-123")
            .returning(|_| Box::pin(async { Ok(()) }));

        let svc = service(mock);
        svc.logout("session-123").await.unwrap();
    }

    // ----- change_password tests -----

    #[tokio::test]
    async fn change_password_empty_fields_returns_empty_fields() {
        let mock = MockAuthRepository::new();
        let svc = service(mock);
        let outcome = svc.change_password(1, false, "", "new").await.unwrap();
        assert!(matches!(outcome, ChangePasswordOutcome::EmptyFields));

        let mock = MockAuthRepository::new();
        let svc = service(mock);
        let outcome = svc.change_password(1, false, "old", "").await.unwrap();
        assert!(matches!(outcome, ChangePasswordOutcome::EmptyFields));
    }

    #[tokio::test]
    async fn change_password_success() {
        let mut mock = MockAuthRepository::new();
        mock.expect_change_password()
            .returning(|_, _, _| Box::pin(async { Ok(true) }));

        let svc = service(mock);
        let outcome = svc
            .change_password(1, false, "oldpassword", "newpassword")
            .await
            .unwrap();
        assert!(matches!(outcome, ChangePasswordOutcome::Success));
    }

    #[tokio::test]
    async fn change_password_incorrect_returns_incorrect() {
        let mut mock = MockAuthRepository::new();
        mock.expect_change_password()
            .returning(|_, _, _| Box::pin(async { Ok(false) }));

        let svc = service(mock);
        let outcome = svc
            .change_password(1, false, "wrongpassword", "newpassword")
            .await
            .unwrap();
        assert!(matches!(outcome, ChangePasswordOutcome::IncorrectPassword));
    }

    #[tokio::test]
    async fn change_password_demo_user_returns_demo_user() {
        let mock = MockAuthRepository::new();
        let svc = service(mock);
        let outcome = svc
            .change_password(1, true, "oldpassword", "newpassword")
            .await
            .unwrap();
        assert!(matches!(outcome, ChangePasswordOutcome::DemoUser));
    }

    // ----- verify_email tests -----

    #[tokio::test]
    async fn verify_email_valid_token_returns_true() {
        let mut mock = MockAuthRepository::new();
        mock.expect_verify_email_token()
            .returning(|_| Box::pin(async { Ok(true) }));

        let svc = service(mock);
        assert!(svc.verify_email("valid-token").await.unwrap());
    }

    #[tokio::test]
    async fn verify_email_invalid_token_returns_false() {
        let mut mock = MockAuthRepository::new();
        mock.expect_verify_email_token()
            .returning(|_| Box::pin(async { Ok(false) }));

        let svc = service(mock);
        assert!(!svc.verify_email("expired-token").await.unwrap());
    }

    // ----- resend_verification tests -----

    #[tokio::test]
    async fn resend_verification_calls_regenerate_token() {
        let mut mock = MockAuthRepository::new();
        mock.expect_regenerate_verification_token()
            .returning(|_| Box::pin(async { Ok(None) }));

        let svc = service(mock);
        svc.resend_verification("test@example.com").await.unwrap();
    }

    #[tokio::test]
    async fn resend_verification_sends_email_on_token() {
        let mut mock = MockAuthRepository::new();
        mock.expect_regenerate_verification_token()
            .returning(|_| Box::pin(async { Ok(Some("new-token".to_string())) }));

        let svc = service_with_email(mock, mock_email_ok());
        svc.resend_verification("test@example.com").await.unwrap();
    }

    #[tokio::test]
    async fn resend_verification_email_failure_returns_error() {
        let mut mock = MockAuthRepository::new();
        mock.expect_regenerate_verification_token()
            .returning(|_| Box::pin(async { Ok(Some("new-token".to_string())) }));

        let svc = service_with_email(mock, mock_email_fail());
        let result = svc.resend_verification("test@example.com").await;
        assert!(result.is_err());
    }

    // ----- forgot_password tests -----

    #[tokio::test]
    async fn forgot_password_not_configured_returns_false() {
        let mock = MockAuthRepository::new();
        // service() has email disabled → not configured
        let svc = service(mock);
        assert!(!svc.forgot_password("test@example.com").await.unwrap());
    }

    #[tokio::test]
    async fn forgot_password_configured_returns_true() {
        let mut mock = MockAuthRepository::new();
        mock.expect_create_password_reset_token()
            .returning(|_| Box::pin(async { Ok(None) }));

        let svc = service_with_email(mock, mock_email_ok());
        assert!(svc.forgot_password("test@example.com").await.unwrap());
    }

    #[tokio::test]
    async fn forgot_password_email_failure_still_returns_true() {
        let mut mock = MockAuthRepository::new();
        mock.expect_create_password_reset_token()
            .returning(|_| Box::pin(async { Ok(Some("reset-token".to_string())) }));

        // Email fails, but forgot_password should swallow the error for security
        let svc = service_with_email(mock, mock_email_fail());
        assert!(svc.forgot_password("test@example.com").await.unwrap());
    }

    // ----- validate_reset_token tests -----

    #[tokio::test]
    async fn validate_reset_token_valid() {
        let mut mock = MockAuthRepository::new();
        mock.expect_validate_password_reset_token()
            .returning(|_| Box::pin(async { Ok(Some("user@example.com".to_string())) }));

        let svc = service(mock);
        assert!(svc.validate_reset_token("valid").await.unwrap());
    }

    #[tokio::test]
    async fn validate_reset_token_invalid() {
        let mut mock = MockAuthRepository::new();
        mock.expect_validate_password_reset_token()
            .returning(|_| Box::pin(async { Ok(None) }));

        let svc = service(mock);
        assert!(!svc.validate_reset_token("expired").await.unwrap());
    }

    // ----- reset_password tests -----

    #[tokio::test]
    async fn reset_password_empty_returns_empty_password() {
        let mock = MockAuthRepository::new();
        let svc = service(mock);
        let outcome = svc.reset_password("token", "").await.unwrap();
        assert!(matches!(outcome, ResetPasswordOutcome::EmptyPassword));
    }

    #[tokio::test]
    async fn reset_password_success() {
        let mut mock = MockAuthRepository::new();
        mock.expect_reset_password_with_token()
            .returning(|_, _| Box::pin(async { Ok(true) }));

        let svc = service(mock);
        let outcome = svc.reset_password("token", "newpassword").await.unwrap();
        assert!(matches!(outcome, ResetPasswordOutcome::Success));
    }

    #[tokio::test]
    async fn reset_password_invalid_token() {
        let mut mock = MockAuthRepository::new();
        mock.expect_reset_password_with_token()
            .returning(|_, _| Box::pin(async { Ok(false) }));

        let svc = service(mock);
        let outcome = svc
            .reset_password("bad-token", "newpassword")
            .await
            .unwrap();
        assert!(matches!(outcome, ResetPasswordOutcome::InvalidToken));
    }
}
