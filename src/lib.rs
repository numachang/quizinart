rust_i18n::i18n!("locales", fallback = "en");

pub mod db;
pub mod extractors;
pub mod handlers;
pub mod models;
pub mod names;
pub mod rejections;
pub mod statics;
pub mod utils;
pub mod views;

use axum::Router;

#[derive(Clone)]
pub struct AppState {
    pub db: db::Db,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .merge(handlers::homepage::routes())
        .merge(handlers::quiz::routes())
        .nest("/static", statics::routes())
        .with_state(state)
}
