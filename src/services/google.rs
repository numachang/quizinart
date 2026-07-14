use color_eyre::Result;
use serde::Deserialize;

use crate::names;

const GOOGLE_AUTH_ENDPOINT: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_ENDPOINT: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_USERINFO_ENDPOINT: &str = "https://openidconnect.googleapis.com/v1/userinfo";

/// Profile of a Google account, obtained after a successful code exchange.
pub struct GoogleUser {
    pub email: String,
    pub email_verified: bool,
    pub display_name: String,
}

/// Google OAuth 2.0 client for the server-side authorization code flow.
#[derive(Clone)]
pub struct GoogleOAuth {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct UserInfoResponse {
    email: String,
    #[serde(default)]
    email_verified: bool,
    #[serde(default)]
    name: Option<String>,
}

impl GoogleOAuth {
    pub fn new(client_id: String, client_secret: String, base_url: &str) -> Self {
        Self {
            client_id,
            client_secret,
            redirect_uri: format!(
                "{}{}",
                base_url.trim_end_matches('/'),
                names::GOOGLE_CALLBACK_URL
            ),
        }
    }

    /// Build the Google consent screen URL carrying the anti-CSRF state token.
    pub fn authorize_url(&self, state: &str) -> String {
        format!(
            "{GOOGLE_AUTH_ENDPOINT}?client_id={}&redirect_uri={}&response_type=code&scope=openid%20email%20profile&state={}",
            urlencoding::encode(&self.client_id),
            urlencoding::encode(&self.redirect_uri),
            urlencoding::encode(state),
        )
    }

    /// Exchange an authorization code for tokens and fetch the user's profile.
    pub async fn fetch_user(&self, code: &str) -> Result<GoogleUser> {
        let client = reqwest::Client::new();

        let token_resp = client
            .post(GOOGLE_TOKEN_ENDPOINT)
            .form(&[
                ("code", code),
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
                ("redirect_uri", self.redirect_uri.as_str()),
                ("grant_type", "authorization_code"),
            ])
            .send()
            .await?;

        if !token_resp.status().is_success() {
            let status = token_resp.status();
            let text = token_resp.text().await.unwrap_or_default();
            tracing::error!("Google token endpoint error: {status} - {text}");
            color_eyre::eyre::bail!("Google token endpoint returned {status}");
        }

        let token: TokenResponse = token_resp.json().await?;

        let info_resp = client
            .get(GOOGLE_USERINFO_ENDPOINT)
            .bearer_auth(&token.access_token)
            .send()
            .await?;

        if !info_resp.status().is_success() {
            let status = info_resp.status();
            let text = info_resp.text().await.unwrap_or_default();
            tracing::error!("Google userinfo endpoint error: {status} - {text}");
            color_eyre::eyre::bail!("Google userinfo endpoint returned {status}");
        }

        let info: UserInfoResponse = info_resp.json().await?;

        Ok(GoogleUser {
            display_name: info.name.unwrap_or_else(|| info.email.clone()),
            email: info.email,
            email_verified: info.email_verified,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authorize_url_contains_redirect_and_state() {
        let google = GoogleOAuth::new(
            "client-123".to_string(),
            "secret".to_string(),
            "https://example.com/",
        );
        let url = google.authorize_url("state-abc");

        assert!(url.starts_with(GOOGLE_AUTH_ENDPOINT));
        assert!(url.contains("client_id=client-123"));
        assert!(url.contains(
            &urlencoding::encode("https://example.com/auth/google/callback").into_owned()
        ));
        assert!(url.contains("state=state-abc"));
        assert!(!url.contains("secret"));
    }
}
