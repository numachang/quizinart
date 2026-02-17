use std::convert::Infallible;

use maud::{html, Markup};
use warp::{
    http::StatusCode,
    reject::{Reject, Rejection},
    reply::Reply,
};

use crate::views;

macro_rules! rejects {
    ($($name:ident),*) => {
        $(
            #[derive(Debug)]
            pub struct $name;

            impl Reject for $name {}
        )*
    };
}

rejects!(InternalServerError, Unauthorized, InputError);

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
    } else if let Some(InternalServerError) = err.find() {
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "INTERNAL_SERVER_ERROR";
    } else if let Some(Unauthorized) = err.find() {
        code = StatusCode::UNAUTHORIZED;
        message = "UNAUTHORIZED";
    } else if let Some(InputError) = err.find() {
        code = StatusCode::BAD_REQUEST;
        message = "INPUT_ERROR";
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
    )
}
