#![deny(clippy::unwrap_used)]

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
use tower_governor::{
    governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor, GovernorLayer,
};

#[derive(Clone)]
pub struct AppState {
    pub db: db::Db,
    pub auth: services::auth::AuthService,
    pub secure_cookies: bool,
}

pub fn router(state: AppState, disable_rate_limit: bool) -> Router {
    let auth_routes = if disable_rate_limit {
        handlers::homepage::auth_post_routes()
    } else {
        let governor_conf = GovernorConfigBuilder::default()
            .per_second(30)
            .burst_size(5)
            .key_extractor(SmartIpKeyExtractor)
            .use_headers()
            .finish()
            .expect("valid governor config");

        handlers::homepage::auth_post_routes().layer(GovernorLayer::new(governor_conf))
    };

    Router::new()
        .merge(handlers::homepage::routes())
        .merge(auth_routes)
        .merge(handlers::quiz::routes())
        .merge(handlers::account::routes())
        .merge(handlers::admin::routes())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            refresh_session_cookie,
        ))
        .layer(middleware::from_fn(csrf_check))
        .route("/health", axum::routing::get(health))
        .nest("/static", statics::routes())
        .layer(middleware::from_fn(security_headers))
        .layer(tower_http::compression::CompressionLayer::new())
        .with_state(state)
}

async fn health(State(state): State<AppState>) -> axum::response::Response {
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    match state.db.health_check().await {
        Ok(()) => StatusCode::OK.into_response(),
        Err(_) => StatusCode::SERVICE_UNAVAILABLE.into_response(),
    }
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
            if let Ok(header_value) = utils::cookie(
                names::USER_SESSION_COOKIE_NAME,
                &value,
                state.secure_cookies,
            ) {
                response
                    .headers_mut()
                    .append(axum::http::header::SET_COOKIE, header_value);
            }
        }
    }

    response
}

async fn security_headers(
    req: axum::http::Request<axum::body::Body>,
    next: middleware::Next,
) -> axum::response::Response {
    use axum::http::HeaderValue;

    let mut response = next.run(req).await;
    let headers = response.headers_mut();

    headers.insert(
        "X-Content-Type-Options",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert("X-Frame-Options", HeaderValue::from_static("DENY"));
    headers.insert(
        "Referrer-Policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(
        "Content-Security-Policy",
        HeaderValue::from_static(
            "default-src 'self'; \
             script-src 'self'; \
             style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; \
             font-src 'self' https://fonts.gstatic.com; \
             img-src 'self' data:; \
             connect-src 'self'",
        ),
    );

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
