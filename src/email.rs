use color_eyre::Result;
use serde::Serialize;

#[derive(Serialize)]
struct SendEmailRequest {
    from: String,
    to: Vec<String>,
    subject: String,
    html: String,
}

/// Send a verification email via Resend API.
pub async fn send_verification_email(
    api_key: &str,
    to_email: &str,
    verification_url: &str,
) -> Result<()> {
    let client = reqwest::Client::new();

    let body = SendEmailRequest {
        from: "Quizinart <noreply@quizinart.numachang.com>".to_string(),
        to: vec![to_email.to_string()],
        subject: "Verify your Quizinart account".to_string(),
        html: format!(
            r#"<h2>Welcome to Quizinart!</h2>
<p>Click the link below to verify your email address:</p>
<p><a href="{verification_url}">{verification_url}</a></p>
<p>This link expires in 24 hours.</p>"#
        ),
    };

    let resp = client
        .post("https://api.resend.com/emails")
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

    tracing::info!("verification email sent to {to_email}");
    Ok(())
}

/// Send a password reset email via Resend API.
pub async fn send_password_reset_email(
    api_key: &str,
    to_email: &str,
    reset_url: &str,
) -> Result<()> {
    let client = reqwest::Client::new();

    let body = SendEmailRequest {
        from: "Quizinart <noreply@quizinart.numachang.com>".to_string(),
        to: vec![to_email.to_string()],
        subject: "Reset your Quizinart password".to_string(),
        html: format!(
            r#"<h2>Password Reset</h2>
<p>Click the link below to reset your password:</p>
<p><a href="{reset_url}">{reset_url}</a></p>
<p>This link expires in 1 hour.</p>
<p>If you did not request this, you can safely ignore this email.</p>"#
        ),
    };

    let resp = client
        .post("https://api.resend.com/emails")
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

    tracing::info!("password reset email sent to {to_email}");
    Ok(())
}
