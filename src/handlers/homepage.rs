use std::collections::HashMap;

use crate::{
    db::Db,
    handlers::quiz,
    is_authorized, models, names,
    rejections::{InputError, InternalServerError},
    utils, views, with_state, FutureOptionExt,
};

use crate::views::homepage as homepage_views;
use futures::FutureExt;
use maud::html;
use serde::Deserialize;
use warp::{
    filters::multipart::FormData,
    http::{header::SET_COOKIE, Response},
    reply::Reply,
    Filter,
};

pub fn route(
    conn: Db,
) -> impl warp::Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let homepage = warp::path::end()
        .and(warp::get())
        .and(with_state(conn.clone()))
        .and(warp::cookie::optional::<String>(names::ADMIN_SESSION_COOKIE_NAME))
        .and_then(homepage);

    let get_started_post = warp::path("start")
        .and(warp::post())
        .and(with_state(conn.clone()))
        .and(warp::body::json::<GetStartedPost>())
        .and_then(get_started_post);

    let login_post = warp::path("login")
        .and(warp::post())
        .and(with_state(conn.clone()))
        .and(warp::body::json::<LoginPost>())
        .and_then(login_post);

    let create_quiz = warp::path("create-quiz")
        .and(warp::post())
        .and(is_authorized(conn.clone()))
        .and(with_state(conn.clone()))
        .and(warp::multipart::form())
        .and_then(create_quiz);

    let delete_quiz = is_authorized(conn.clone())
        .and(with_state(conn.clone()))
        .and(warp::delete())
        .and(warp::path!("delete-quiz" / i32))
        .and_then(delete_quiz);

    homepage
        .or(get_started_post)
        .or(login_post)
        .or(create_quiz)
        .or(delete_quiz)
}

async fn homepage(
    db: Db,
    session: Option<String>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let session_exists = session
        .map(|s| db.admin_session_exists(s).map(|res| res.ok()))
        .to_future()
        .await
        .flatten()
        .unwrap_or_default();

    if session_exists {
        Ok(views::page("Dashboard", { let quizzes = db.quizzes().await.map_err(|e| { tracing::error!("could not get quizzes: {e}"); warp::reject::custom(InternalServerError) })?; homepage_views::dashboard(quizzes) }))
    } else {
        let admin_password = db.admin_password().await.map_err(|e| {
            tracing::error!("could not get admin password: {e}");
            warp::reject::custom(InternalServerError)
        })?;

        match admin_password {
            Some(_) => Ok(views::page("Welcome Back", homepage_views::login(homepage_views::LoginState::NoError))),
            None => Ok(views::page("Get Started", homepage_views::get_started())),
        }
    }
}

#[derive(Deserialize)]
struct GetStartedPost {
    admin_password: String,
}

async fn get_started_post(
    db: Db,
    body: GetStartedPost,
) -> Result<impl warp::Reply, warp::Rejection> {
    db.set_admin_password(body.admin_password)
        .await
        .map_err(|e| {
            tracing::error!("could not set admin password: {e}");
            warp::reject::custom(InternalServerError)
        })?;

    let session = db.create_admin_session().await.map_err(|e| {
        tracing::error!("could not create a new session: {e}");
        warp::reject::custom(InternalServerError)
    })?;

    let cookie = utils::cookie(names::ADMIN_SESSION_COOKIE_NAME, &session);
    let resp = Response::builder()
        .header(SET_COOKIE, cookie)
        .body(views::titled("Dashboard", { let quizzes = db.quizzes().await.map_err(|e| { tracing::error!("could not get quizzes: {e}"); warp::reject::custom(InternalServerError) })?; homepage_views::dashboard(quizzes) }).into_string())
        .unwrap();

    Ok(resp)
}

#[derive(Deserialize)]
struct LoginPost {
    admin_password: String,
}

async fn login_post(db: Db, body: LoginPost) -> Result<impl warp::Reply, warp::Rejection> {
    let admin_password = db.admin_password().await.map_err(|e| {
        tracing::error!("could not get admin password: {e}");
        warp::reject::custom(InternalServerError)
    })?;

    if admin_password == Some(body.admin_password) {
        let session = db.create_admin_session().await.map_err(|e| {
            tracing::error!("could not create a new session: {e}");
            warp::reject::custom(InternalServerError)
        })?;

        let cookie = utils::cookie(names::ADMIN_SESSION_COOKIE_NAME, &session);
        let resp = Response::builder()
            .header(SET_COOKIE, cookie)
            .body(views::titled("Dashboard", { let quizzes = db.quizzes().await.map_err(|e| { tracing::error!("could not get quizzes: {e}"); warp::reject::custom(InternalServerError) })?; homepage_views::dashboard(quizzes) }).into_string())
            .unwrap();

        Ok(resp.into_response())
    } else {
        Ok(views::titled("Welcome Back", homepage_views::login(homepage_views::LoginState::IncorrectPassword)).into_response())
    }
}

async fn create_quiz(
    _: (),
    db: Db,
    form: FormData,
) -> Result<impl warp::Reply, warp::Rejection> {
    use bytes::BufMut;
    use futures::TryStreamExt;

    let mut field_names: HashMap<_, _> = form
        .and_then(|mut field| async move {
            let mut bytes: Vec<u8> = Vec::new();

            while let Some(content) = field.data().await {
                let content = content.unwrap();
                bytes.put(content);
            }
            Ok((
                field.name().to_string(),
                String::from_utf8_lossy(&bytes).to_string(),
            ))
        })
        .try_collect()
        .await
        .map_err(|e| {
            tracing::error!("failed to decode form data: {e}");
            warp::reject::custom(InputError)
        })?;

    let quiz_name = field_names
        .remove("quiz_name")
        .ok_or_else(|| warp::reject::custom(InputError))?;

    let quiz_file = field_names
        .remove("quiz_file")
        .ok_or_else(|| warp::reject::custom(InputError))?;

    let questions = serde_json::from_str::<models::Questions>(&quiz_file).map_err(|e| {
        tracing::error!("failed to decode quiz file: {e}");
        warp::reject::custom(InputError)
    })?;

    let quiz_id = db.load_quiz(quiz_name, questions).await.map_err(|e| {
        tracing::error!("failed to decode quiz file: {e}");
        warp::reject::custom(InputError)
    })?;

    let resp = Response::builder()
        .header("HX-Replace-Url", names::quiz_dashboard_url(quiz_id))
        .body(
            views::titled("Quiz Dashboard", quiz::dashboard(&db, quiz_id).await?).into_string(),
        )
        .unwrap();

    Ok(resp)
}

async fn delete_quiz(_: (), db: Db, quiz_id: i32) -> Result<impl warp::Reply, warp::Rejection> {
    db.delete_quiz(quiz_id).await.map_err(|e| {
        tracing::error!("failed to delete a quiz: {e}");
        warp::reject::custom(InternalServerError)
    })?;

    Ok(html!())
}

