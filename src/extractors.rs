use std::convert::Infallible;

use axum::{extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::CookieJar;

use crate::{names, rejections::AppError, AppState};

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

/// Guard extractor that verifies the admin session cookie against the database.
/// Rejects with `AppError::Unauthorized` if the session is invalid or missing.
pub struct AuthGuard;

impl FromRequestParts<AppState> for AuthGuard {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_headers(&parts.headers);
        let session = jar
            .get(names::ADMIN_SESSION_COOKIE_NAME)
            .map(|c| c.value().to_string());

        let exists = match session {
            Some(s) => state.db.admin_session_exists(s).await.unwrap_or(false),
            None => false,
        };

        if exists {
            Ok(AuthGuard)
        } else {
            Err(AppError::Unauthorized)
        }
    }
}
