rust_i18n::i18n!("locales", fallback = "en");

pub mod db;
pub mod handlers;
pub mod models;
pub mod names;
pub mod rejections;
pub mod statics;
pub mod utils;
pub mod views;

use db::Db;
use futures::{future::OptionFuture, FutureExt};
use warp::Filter;

pub fn routes(
    conn: db::Db,
) -> impl warp::Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    handlers::homepage::route(conn.clone()).or(handlers::quiz::route(conn.clone()))
}

pub fn with_state<T: Clone + Send>(
    db: T,
) -> impl Filter<Extract = (T,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

pub trait FutureOptionExt<T> {
    fn to_future(self) -> OptionFuture<T>;
}

impl<T> FutureOptionExt<T> for Option<T> {
    fn to_future(self) -> OptionFuture<T> {
        OptionFuture::from(self)
    }
}

pub fn is_authorized(
    db: Db,
) -> impl Filter<Extract = ((),), Error = warp::reject::Rejection> + Clone {
    warp::any()
        .and(with_state(db.clone()))
        .and(warp::cookie::optional::<String>(
            names::ADMIN_SESSION_COOKIE_NAME,
        ))
        .and_then(authorized)
}

async fn authorized(db: Db, session: Option<String>) -> Result<(), warp::Rejection> {
    let session_exists = session
        .map(|s| db.admin_session_exists(s).map(|res| res.ok()))
        .to_future()
        .await
        .flatten()
        .unwrap_or_default();

    if session_exists {
        Ok(())
    } else {
        Err(warp::reject::custom(rejections::AppError::Unauthorized))
    }
}

pub fn is_htmx() -> impl Filter<Extract = (bool,), Error = warp::reject::Rejection> + Clone {
    warp::any()
        .and(warp::header::optional::<String>("HX-Request"))
        .map(|hx_req: Option<String>| hx_req.is_some_and(|x| x == "true"))
}

pub fn with_locale() -> impl Filter<Extract = (String,), Error = std::convert::Infallible> + Clone {
    warp::cookie::optional::<String>(names::LOCALE_COOKIE_NAME).map(|lang: Option<String>| {
        match lang.as_deref() {
            Some("ja") => "ja".to_string(),
            Some("zh-CN") => "zh-CN".to_string(),
            Some("zh-TW") => "zh-TW".to_string(),
            _ => "en".to_string(),
        }
    })
}
