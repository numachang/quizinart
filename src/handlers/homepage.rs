use std::collections::HashMap;

use crate::{
    db::Db,
    handlers::quiz,
    is_authorized, models, names,
    rejections::{AppError, ResultExt},
    utils, views, with_locale, with_state, FutureOptionExt,
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
        .and(warp::cookie::optional::<String>(
            names::ADMIN_SESSION_COOKIE_NAME,
        ))
        .and(with_locale())
        .and_then(homepage);

    let get_started_post = warp::path("start")
        .and(warp::post())
        .and(with_state(conn.clone()))
        .and(warp::body::json::<GetStartedPost>())
        .and(with_locale())
        .and_then(get_started_post);

    let login_post = warp::path("login")
        .and(warp::post())
        .and(with_state(conn.clone()))
        .and(warp::body::json::<LoginPost>())
        .and(with_locale())
        .and_then(login_post);

    let create_quiz = warp::path("create-quiz")
        .and(warp::post())
        .and(is_authorized(conn.clone()))
        .and(with_state(conn.clone()))
        .and(warp::multipart::form())
        .and(with_locale())
        .and_then(create_quiz);

    let delete_quiz = is_authorized(conn.clone())
        .and(with_state(conn.clone()))
        .and(warp::delete())
        .and(warp::path!("delete-quiz" / i32))
        .and_then(delete_quiz);

    let set_locale = warp::path("set-locale")
        .and(warp::post())
        .and(warp::body::json::<SetLocaleBody>())
        .and_then(set_locale);

    homepage
        .or(get_started_post)
        .or(login_post)
        .or(create_quiz)
        .or(delete_quiz)
        .or(set_locale)
}

async fn homepage(
    db: Db,
    session: Option<String>,
    locale: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    let session_exists = session
        .map(|s| db.admin_session_exists(s).map(|res| res.ok()))
        .to_future()
        .await
        .flatten()
        .unwrap_or_default();

    if session_exists {
        let quizzes = db.quizzes().await.reject("could not get quizzes")?;
        Ok(views::page(
            "Dashboard",
            homepage_views::dashboard(quizzes, &locale),
            &locale,
        ))
    } else {
        let admin_password = db
            .admin_password()
            .await
            .reject("could not get admin password")?;

        match admin_password {
            Some(_) => Ok(views::page(
                "Welcome Back",
                homepage_views::login(homepage_views::LoginState::NoError, &locale),
                &locale,
            )),
            None => Ok(views::page(
                "Get Started",
                homepage_views::get_started(&locale),
                &locale,
            )),
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
    locale: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    db.set_admin_password(body.admin_password)
        .await
        .reject("could not set admin password")?;

    let session = db
        .create_admin_session()
        .await
        .reject("could not create admin session")?;

    let cookie = utils::cookie(names::ADMIN_SESSION_COOKIE_NAME, &session);
    let quizzes = db.quizzes().await.reject("could not get quizzes")?;
    let resp = Response::builder()
        .header(SET_COOKIE, cookie)
        .body(views::titled("Dashboard", homepage_views::dashboard(quizzes, &locale)).into_string())
        .unwrap();

    Ok(resp)
}

#[derive(Deserialize)]
struct LoginPost {
    admin_password: String,
}

async fn login_post(
    db: Db,
    body: LoginPost,
    locale: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    let admin_password = db
        .admin_password()
        .await
        .reject("could not get admin password")?;

    if admin_password == Some(body.admin_password) {
        let session = db
            .create_admin_session()
            .await
            .reject("could not create admin session")?;

        let cookie = utils::cookie(names::ADMIN_SESSION_COOKIE_NAME, &session);
        let quizzes = db.quizzes().await.reject("could not get quizzes")?;
        let resp = Response::builder()
            .header(SET_COOKIE, cookie)
            .body(
                views::titled("Dashboard", homepage_views::dashboard(quizzes, &locale))
                    .into_string(),
            )
            .unwrap();

        Ok(resp.into_response())
    } else {
        Ok(views::titled(
            "Welcome Back",
            homepage_views::login(homepage_views::LoginState::IncorrectPassword, &locale),
        )
        .into_response())
    }
}

async fn create_quiz(
    _: (),
    db: Db,
    form: FormData,
    locale: String,
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
            warp::reject::custom(AppError::Input("failed to decode form data"))
        })?;

    let quiz_name = field_names
        .remove("quiz_name")
        .ok_or_else(|| warp::reject::custom(AppError::Input("missing quiz_name field")))?;

    let quiz_file = field_names
        .remove("quiz_file")
        .ok_or_else(|| warp::reject::custom(AppError::Input("missing quiz_file field")))?;

    let questions = serde_json::from_str::<models::Questions>(&quiz_file)
        .reject_input("failed to decode quiz file")?;

    let quiz_id = db
        .load_quiz(quiz_name, questions)
        .await
        .reject_input("failed to load quiz")?;

    let resp = Response::builder()
        .header("HX-Replace-Url", names::quiz_dashboard_url(quiz_id))
        .body(
            views::titled(
                "Quiz Dashboard",
                quiz::dashboard(&db, quiz_id, &locale).await?,
            )
            .into_string(),
        )
        .unwrap();

    Ok(resp)
}

async fn delete_quiz(_: (), db: Db, quiz_id: i32) -> Result<impl warp::Reply, warp::Rejection> {
    db.delete_quiz(quiz_id)
        .await
        .reject("failed to delete quiz")?;

    Ok(html!())
}

#[derive(Deserialize)]
struct SetLocaleBody {
    locale: String,
}

async fn set_locale(body: SetLocaleBody) -> Result<impl warp::Reply, warp::Rejection> {
    let locale = match body.locale.as_str() {
        "ja" => "ja",
        "zh-CN" => "zh-CN",
        "zh-TW" => "zh-TW",
        _ => "en",
    };
    let cookie = utils::cookie(names::LOCALE_COOKIE_NAME, locale);
    Ok(Response::builder()
        .header(SET_COOKIE, cookie)
        .header("HX-Refresh", "true")
        .body("")
        .unwrap())
}
