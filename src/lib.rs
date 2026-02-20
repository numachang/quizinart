rust_i18n::i18n!("locales", fallback = "en");

pub mod db;
pub mod email;
pub mod extractors;
pub mod handlers;
pub mod models;
pub mod names;
pub mod rejections;
pub mod statics;
pub mod utils;
pub mod views;

use axum::{middleware, Router};

#[derive(Clone)]
pub struct AppState {
    pub db: db::Db,
    pub secure_cookies: bool,
    pub resend_api_key: String,
    pub base_url: String,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .merge(handlers::homepage::routes())
        .merge(handlers::quiz::routes())
        .merge(handlers::account::routes())
        .layer(middleware::from_fn(csrf_check))
        .nest("/static", statics::routes())
        .with_state(state)
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

        if !has_hx_request {
            return (StatusCode::FORBIDDEN, "CSRF check failed").into_response();
        }
    }

    next.run(req).await
}
