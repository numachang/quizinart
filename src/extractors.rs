use std::convert::Infallible;

use axum::{extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::CookieJar;

use crate::{db::models::AuthUser, names, rejections::AppError, AppState};

/// Extracts whether the request is an HTMX request by checking the `HX-Request` header.
pub struct IsHtmx(pub bool);

impl<S: Send + Sync> FromRequestParts<S> for IsHtmx {
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let is_htmx = parts
            .headers
            .get("HX-Request")
            .and_then(|v: &axum::http::HeaderValue| v.to_str().ok())
            .is_some_and(|v| v == "true");
        Ok(IsHtmx(is_htmx))
    }
}

/// Extracts the locale from the `lang` cookie, defaulting to `"en"`.
pub struct Locale(pub String);

impl<S: Send + Sync> FromRequestParts<S> for Locale {
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_headers(&parts.headers);
        let lang = jar.get(names::LOCALE_COOKIE_NAME).map(|c| c.value());
        let locale = match lang {
            Some("ja") => "ja",
            Some("zh-CN") => "zh-CN",
            Some("zh-TW") => "zh-TW",
            _ => "en",
        };
        Ok(Locale(locale.to_string()))
    }
}

/// Guard extractor that verifies the user session cookie against the database.
/// Carries the authenticated user's info for use in handlers.
/// Falls back to legacy admin_session cookie for migration compatibility.
pub struct AuthGuard(pub AuthUser);

impl FromRequestParts<AppState> for AuthGuard {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_headers(&parts.headers);

        // Try new user_session cookie first
        if let Some(session_id) = jar
            .get(names::USER_SESSION_COOKIE_NAME)
            .map(|c| c.value().to_string())
        {
            if let Ok(Some(user)) = state.db.get_user_by_session(&session_id).await {
                return Ok(AuthGuard(user));
            }
        }

        // Fallback: legacy admin_session cookie → map to default migration user
        if let Some(admin_session) = jar
            .get(names::ADMIN_SESSION_COOKIE_NAME)
            .map(|c| c.value().to_string())
        {
            let exists = state
                .db
                .admin_session_exists(admin_session)
                .await
                .unwrap_or(false);
            if exists {
                if let Ok(Some(user)) = state.db.find_user_by_email("admin@local").await {
                    return Ok(AuthGuard(user));
                }
            }
        }

        Err(AppError::Unauthorized)
    }
}
