use std::time::Duration;

use color_eyre::Result;
use serde::Serialize;

use crate::services::auth::EmailSender;

const MAX_RETRIES: u32 = 3;
const RESEND_API_URL: &str = "https://api.resend.com/emails";
const FROM_ADDRESS: &str = "Quizinart <noreply@quizinart.numachang.com>";

#[derive(Serialize)]
struct SendEmailRequest {
    from: String,
    to: Vec<String>,
    subject: String,
    html: String,
}

/// Concrete email sender using the Resend API with exponential backoff retry.
#[derive(Clone)]
pub struct ResendEmailSender {
    api_key: String,
}

impl ResendEmailSender {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

impl EmailSender for ResendEmailSender {
    fn is_enabled(&self) -> bool {
        !self.api_key.is_empty()
    }

    async fn send_verification_email(&self, to_email: &str, verification_url: &str) -> Result<()> {
        let subject = "Verify your Quizinart account";
        let html = format!(
            r#"<h2>Welcome to Quizinart!</h2>
<p>Click the link below to verify your email address:</p>
<p><a href="{verification_url}">{verification_url}</a></p>
<p>This link expires in 24 hours.</p>"#
        );
        send_email_with_retry(&self.api_key, to_email, subject, &html).await?;
        tracing::info!("verification email sent to {to_email}");
        Ok(())
    }

    async fn send_password_reset_email(&self, to_email: &str, reset_url: &str) -> Result<()> {
        let subject = "Reset your Quizinart password";
        let html = format!(
            r#"<h2>Password Reset</h2>
<p>Click the link below to reset your password:</p>
<p><a href="{reset_url}">{reset_url}</a></p>
<p>This link expires in 1 hour.</p>
<p>If you did not request this, you can safely ignore this email.</p>"#
        );
        send_email_with_retry(&self.api_key, to_email, subject, &html).await?;
        tracing::info!("password reset email sent to {to_email}");
        Ok(())
    }
}

/// Send an email via Resend API with exponential backoff retry (3 attempts: 1s, 2s, 4s).
async fn send_email_with_retry(
    api_key: &str,
    to_email: &str,
    subject: &str,
    html: &str,
) -> Result<()> {
    let mut last_err = None;

    for attempt in 0..MAX_RETRIES {
        match send_email_request(api_key, to_email, subject, html).await {
            Ok(()) => return Ok(()),
            Err(e) => {
                let attempt_num = attempt + 1;
                tracing::warn!(
                    attempt = attempt_num,
                    max_retries = MAX_RETRIES,
                    "email send failed: {e}"
                );
                last_err = Some(e);
                if attempt < MAX_RETRIES - 1 {
                    let delay = Duration::from_secs(1 << attempt); // 1s, 2s, 4s
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    Err(last_err.expect("at least one attempt was made"))
}

/// Send a single email request to the Resend API (no retry).
async fn send_email_request(
    api_key: &str,
    to_email: &str,
    subject: &str,
    html: &str,
) -> Result<()> {
    let client = reqwest::Client::new();

    let body = SendEmailRequest {
        from: FROM_ADDRESS.to_string(),
        to: vec![to_email.to_string()],
        subject: subject.to_string(),
        html: html.to_string(),
    };

    let resp = client
        .post(RESEND_API_URL)
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        tracing::error!("Resend API error: {status} - {text}");
        color_eyre::eyre::bail!("Resend API returned {status}");
    }

    Ok(())
}
