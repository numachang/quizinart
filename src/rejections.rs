use std::convert::Infallible;

use maud::{html, Markup};
use warp::{
    http::StatusCode,
    reject::{Reject, Rejection},
    reply::Reply,
};

use crate::{names, views};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Internal(&'static str),

    #[error("unauthorized")]
    Unauthorized,

    #[error("{0}")]
    Input(&'static str),
}

impl Reject for AppError {}

/// Extension trait for ergonomic rejection conversion.
///
/// Usage: `db.quiz_name(id).await.reject("could not get quiz name")?`
pub trait ResultExt<T> {
    fn reject(self, context: &'static str) -> Result<T, Rejection>;
    fn reject_input(self, context: &'static str) -> Result<T, Rejection>;
}

impl<T, E: Into<color_eyre::eyre::Error>> ResultExt<T> for Result<T, E> {
    fn reject(self, context: &'static str) -> Result<T, Rejection> {
        self.map_err(|e| {
            tracing::error!("{context}: {}", e.into());
            warp::reject::custom(AppError::Internal(context))
        })
    }

    fn reject_input(self, context: &'static str) -> Result<T, Rejection> {
        self.map_err(|e| {
            tracing::error!("{context}: {}", e.into());
            warp::reject::custom(AppError::Input(context))
        })
    }
}

pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "NOT_FOUND";
    } else if err
        .find::<warp::filters::body::BodyDeserializeError>()
        .is_some()
    {
        code = StatusCode::BAD_REQUEST;
        message = "BAD_REQUEST";
    } else if let Some(app_err) = err.find::<AppError>() {
        match app_err {
            AppError::Internal(_) => {
                code = StatusCode::INTERNAL_SERVER_ERROR;
                message = "INTERNAL_SERVER_ERROR";
            }
            AppError::Unauthorized => {
                code = StatusCode::UNAUTHORIZED;
                message = "UNAUTHORIZED";
            }
            AppError::Input(_) => {
                code = StatusCode::BAD_REQUEST;
                message = "INPUT_ERROR";
            }
        }
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "METHOD_NOT_ALLOWED";
    } else if err
        .find::<warp::reject::InvalidHeader>()
        .is_some_and(|e| e.name() == warp::http::header::COOKIE)
    {
        code = StatusCode::BAD_REQUEST;
        message = "COOKIE_NOT_AVAILABLE";
    } else {
        tracing::error!("unhandled rejection: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "UNHANDLED_REJECTION";
    }

    Ok(warp::reply::with_status(error_page(message), code))
}

fn error_page(message: &str) -> Markup {
    views::page(
        "Error",
        html! {
            h1 { (message) }
        },
        names::DEFAULT_LOCALE,
    )
}
