rust_i18n::i18n!("locales", fallback = "en");

pub mod db;
pub mod email;
pub mod extractors;
pub mod handlers;
pub mod models;
pub mod names;
pub mod rejections;
pub mod services;
pub mod statics;
pub mod utils;
pub mod views;

use axum::{extract::State, middleware, Router};
use axum_extra::extract::CookieJar;

#[derive(Clone)]
pub struct AppState {
    pub db: db::Db,
    pub auth: services::auth::AuthService,
    pub secure_cookies: bool,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .merge(handlers::homepage::routes())
        .merge(handlers::quiz::routes())
        .merge(handlers::account::routes())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            refresh_session_cookie,
        ))
        .layer(middleware::from_fn(csrf_check))
        .nest("/static", statics::routes())
        .with_state(state)
}

/// Sliding expiration: refresh the user_session cookie Max-Age on every successful response.
async fn refresh_session_cookie(
    State(state): State<AppState>,
    req: axum::http::Request<axum::body::Body>,
    next: middleware::Next,
) -> axum::response::Response {
    let jar = CookieJar::from_headers(req.headers());
    let session_value = jar
        .get(names::USER_SESSION_COOKIE_NAME)
        .map(|c| c.value().to_string());

    let mut response = next.run(req).await;

    if response.status().is_success() {
        if let Some(value) = session_value {
            let cookie_str = utils::cookie(
                names::USER_SESSION_COOKIE_NAME,
                &value,
                state.secure_cookies,
            );
            if let Ok(header_value) = cookie_str.parse() {
                response
                    .headers_mut()
                    .append(axum::http::header::SET_COOKIE, header_value);
            }
        }
    }

    response
}

async fn csrf_check(
    req: axum::http::Request<axum::body::Body>,
    next: middleware::Next,
) -> axum::response::Response {
    use axum::http::{Method, StatusCode};
    use axum::response::IntoResponse;

    let state_changing = [Method::POST, Method::PUT, Method::PATCH, Method::DELETE];

    if state_changing.contains(req.method()) {
        let has_hx_request = req
            .headers()
            .get("HX-Request")
            .and_then(|v| v.to_str().ok())
            .is_some_and(|v| v == "true");

        // Also allow same-origin HTML form submissions (Origin matches Host)
        let has_same_origin = req
            .headers()
            .get("Origin")
            .and_then(|o| o.to_str().ok())
            .zip(req.headers().get("Host").and_then(|h| h.to_str().ok()))
            .is_some_and(|(origin, host)| origin.ends_with(host));

        if !has_hx_request && !has_same_origin {
            return (StatusCode::FORBIDDEN, "CSRF check failed").into_response();
        }
    }

    next.run(req).await
}
