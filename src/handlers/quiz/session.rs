use maud::{html, Markup};
use rust_i18n::t;
use warp::{
    http::{header::SET_COOKIE, Response},
    reject::Rejection,
};

use crate::{
    db::Db,
    names,
    rejections::{AppError, ResultExt},
    utils, views,
    views::quiz as quiz_views,
};
use super::StartSessionBody;

pub(crate) async fn start_session(
    db: Db,
    quiz_id: i32,
    body: StartSessionBody,
    locale: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    let question_count = body.question_count.clamp(names::MIN_QUESTION_COUNT, names::MAX_QUESTION_COUNT);

    let selection_mode = if names::SELECTION_MODES.contains(&body.selection_mode.as_str()) {
        body.selection_mode.as_str()
    } else {
        names::DEFAULT_SELECTION_MODE
    };

    let session_token = match db
        .create_session(&body.name, quiz_id, question_count, selection_mode)
        .await
    {
        Ok(token) => {
            tracing::info!("Created new session for '{}'", body.name);
            token
        }
        Err(e) if e.to_string().contains("already in use") => {
            tracing::warn!("Duplicate session name attempted: {}", body.name);

            let error_html = views::titled(
                "Error",
                quiz_views::session_name_error_page(&body.name, quiz_id, &locale),
            );

            return Ok(Response::builder()
                .status(200)
                .body(error_html.into_string())
                .unwrap());
        }
        Err(e) => {
            tracing::error!("could not create session for quiz={quiz_id}: {e}");
            return Err(warp::reject::custom(AppError::Internal("could not create session")));
        }
    };

    let session = db.get_session(&session_token).await
        .reject("could not get session")?;

    let quiz_name = db.quiz_name(quiz_id).await
        .reject("could not get quiz name")?;

    let question_idx = db.current_question_index(session.id).await
        .reject("could not get current question index")?;

    let page = views::titled(
        &quiz_name,
        super::question::question(&db, session.id, quiz_id, question_idx, false, &locale).await?,
    );
    let cookie = utils::cookie(names::QUIZ_SESSION_COOKIE_NAME, &session_token);
    let resp = Response::builder()
        .header(SET_COOKIE, cookie)
        .body(page.into_string())
        .unwrap();

    Ok(resp)
}

pub(crate) async fn resume_session(
    db: Db,
    session_id: i32,
    token: String,
    locale: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    tracing::info!(
        "Resuming session {} with token {}",
        session_id,
        token
    );

    let session = db.get_session(&token).await
        .reject("could not get session")?;

    let quiz_name = db.quiz_name(session.quiz_id).await
        .reject("could not get quiz name")?;

    let question_idx = db.current_question_index(session.id).await
        .reject("could not get current question index")?;

    let is_resuming = question_idx > 0;
    let page = views::titled(
        &quiz_name,
        super::question::question(&db, session.id, session.quiz_id, question_idx, is_resuming, &locale).await?,
    );
    let cookie_header = utils::cookie(names::QUIZ_SESSION_COOKIE_NAME, &token);

    let resp = Response::builder()
        .header(SET_COOKIE, cookie_header)
        .body(page.into_string())
        .unwrap();

    Ok(resp)
}

pub(crate) async fn retry_incorrect(
    db: Db,
    session_id: i32,
    locale: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    let session = db.get_session_by_id(session_id).await
        .reject("could not get session")?;

    let incorrect_ids = db.get_incorrect_questions(session_id).await
        .reject("could not get incorrect questions")?;

    if incorrect_ids.is_empty() {
        let page = views::titled("Results", html! {
            p { (t!("result.no_incorrect", locale = &locale)) }
            button hx-get=(names::results_url(session_id))
                   hx-push-url="true"
                   hx-target="main" {
                (t!("result.back_to_results", locale = &locale))
            }
        });
        return Ok(Response::builder()
            .body(page.into_string())
            .unwrap());
    }

    let suffix = &ulid::Ulid::new().to_string()[..6];
    let retry_name = format!("{}-retry-{}", session.name, suffix.to_lowercase());

    let token = db
        .create_session_with_questions(&retry_name, session.quiz_id, &incorrect_ids, "incorrect")
        .await
        .reject("could not create retry session")?;

    let new_session = db.get_session(&token).await
        .reject("could not get new session")?;

    let quiz_name = db.quiz_name(session.quiz_id).await
        .reject("could not get quiz name")?;

    let page = views::titled(
        &quiz_name,
        super::question::question(&db, new_session.id, session.quiz_id, 0, false, &locale).await?,
    );
    let cookie = utils::cookie(names::QUIZ_SESSION_COOKIE_NAME, &token);

    Ok(Response::builder()
        .header(SET_COOKIE, cookie)
        .body(page.into_string())
        .unwrap())
}

pub(crate) async fn page(db: &Db, quiz_id: i32, locale: &str) -> Result<Markup, Rejection> {
    let quiz_name = db.quiz_name(quiz_id).await
        .reject("could not get quiz name")?;

    let total_questions = db.questions_count(quiz_id).await
        .reject("could not get question count")?;

    Ok(quiz_views::start_page(quiz_views::StartPageData {
        quiz_name,
        total_questions,
        quiz_id,
    }, locale))
}
