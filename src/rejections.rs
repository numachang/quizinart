use axum::{http::StatusCode, response::IntoResponse};
use maud::html;

use crate::{names, views};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Internal(&'static str),

    #[error("unauthorized")]
    Unauthorized,

    #[error("forbidden")]
    Forbidden,

    #[error("{0}")]
    Input(&'static str),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_SERVER_ERROR"),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED"),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "FORBIDDEN"),
            AppError::Input(_) => (StatusCode::BAD_REQUEST, "INPUT_ERROR"),
        };
        let body = views::page(
            "Error",
            html! {
                h1 { (message) }
            },
            names::DEFAULT_LOCALE,
        );
        (status, body).into_response()
    }
}

/// Extension trait for ergonomic error conversion.
///
/// Usage: `db.quiz_name(id).await.reject("could not get quiz name")?`
pub trait ResultExt<T> {
    fn reject(self, context: &'static str) -> Result<T, AppError>;
    fn reject_input(self, context: &'static str) -> Result<T, AppError>;
}

impl<T, E: Into<color_eyre::eyre::Error>> ResultExt<T> for Result<T, E> {
    fn reject(self, context: &'static str) -> Result<T, AppError> {
        self.map_err(|e| {
            tracing::error!("{context}: {}", e.into());
            AppError::Internal(context)
        })
    }

    fn reject_input(self, context: &'static str) -> Result<T, AppError> {
        self.map_err(|e| {
            tracing::error!("{context}: {}", e.into());
            AppError::Input(context)
        })
    }
}
