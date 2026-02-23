use std::convert::Infallible;

use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
};
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

/// Extracts the locale from the `lang` cookie, falling back to the browser's
/// `Accept-Language` header, then to `"en"`.
pub struct Locale(pub String);

impl<S: Send + Sync> FromRequestParts<S> for Locale {
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_headers(&parts.headers);
        let locale = jar
            .get(names::LOCALE_COOKIE_NAME)
            .and_then(|c| match_supported_locale(c.value()))
            .or_else(|| {
                parts
                    .headers
                    .get(header::ACCEPT_LANGUAGE)
                    .and_then(|v| v.to_str().ok())
                    .and_then(locale_from_accept_language)
            })
            .unwrap_or(names::DEFAULT_LOCALE);
        Ok(Locale(locale.to_string()))
    }
}

/// Match a language tag against supported locales, returning the locale string.
fn match_supported_locale(lang: &str) -> Option<&'static str> {
    match lang {
        "ja" => return Some("ja"),
        "en" => return Some("en"),
        "zh-CN" => return Some("zh-CN"),
        "zh-TW" => return Some("zh-TW"),
        _ => {}
    }
    if lang.starts_with("ja-") || lang.starts_with("en-") {
        return Some(if lang.starts_with("ja") { "ja" } else { "en" });
    }
    if lang == "zh" || lang.starts_with("zh-Hans") {
        return Some("zh-CN");
    }
    if lang.starts_with("zh-Hant") {
        return Some("zh-TW");
    }
    None
}

/// Parse an `Accept-Language` header and return the best matching supported locale.
fn locale_from_accept_language(header: &str) -> Option<&'static str> {
    let mut entries: Vec<(&str, f32)> = header
        .split(',')
        .map(|entry| {
            let entry = entry.trim();
            if let Some((lang, params)) = entry.split_once(';') {
                let q = params
                    .split(';')
                    .find_map(|p| p.trim().strip_prefix("q="))
                    .and_then(|v| v.trim().parse::<f32>().ok())
                    .unwrap_or(1.0);
                (lang.trim(), q)
            } else {
                (entry, 1.0)
            }
        })
        .collect();
    entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    entries
        .iter()
        .find_map(|(lang, _)| match_supported_locale(lang))
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
