use axum::{
    extract::{Path, State},
    http::{header::SET_COOKIE, HeaderMap},
    response::IntoResponse,
    Json,
};
use maud::{html, Markup};
use rust_i18n::t;

use super::StartSessionBody;
use crate::{
    extractors::{AuthGuard, Locale},
    names,
    rejections::{AppError, ResultExt},
    utils, views,
    views::quiz as quiz_views,
    AppState,
};

pub(crate) async fn start_session(
    AuthGuard(user): AuthGuard,
    State(state): State<AppState>,
    Path(quiz_id): Path<i32>,
    Locale(locale): Locale,
    Json(body): Json<StartSessionBody>,
) -> Result<axum::response::Response, AppError> {
    let question_count = body
        .question_count
        .clamp(names::MIN_QUESTION_COUNT, names::MAX_QUESTION_COUNT);

    let selection_mode = if names::SELECTION_MODES.contains(&body.selection_mode.as_str()) {
        body.selection_mode.as_str()
    } else {
        names::DEFAULT_SELECTION_MODE
    };

    let session_token = match state
        .db
        .create_session(&body.name, quiz_id, question_count, selection_mode, user.id)
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

            return Ok(error_html.into_response());
        }
        Err(e) => {
            tracing::error!("could not create session for quiz={quiz_id}: {e}");
            return Err(AppError::Internal("could not create session"));
        }
    };

    let session = state
        .db
        .get_session(&session_token)
        .await
        .reject("could not get session")?;

    let quiz_name = state
        .db
        .quiz_name(quiz_id)
        .await
        .reject("could not get quiz name")?;

    let question_idx = state
        .db
        .current_question_index(session.id)
        .await
        .reject("could not get current question index")?;

    let page = views::titled(
        &quiz_name,
        super::question::question(&state.db, session.id, quiz_id, question_idx, false, &locale)
            .await?,
    );
    let cookie = utils::cookie(
        names::QUIZ_SESSION_COOKIE_NAME,
        &session_token,
        state.secure_cookies,
    );
    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.parse().unwrap());

    Ok((headers, page).into_response())
}

pub(crate) async fn resume_session(
    AuthGuard(_user): AuthGuard,
    State(state): State<AppState>,
    Path((session_id, token)): Path<(i32, String)>,
    Locale(locale): Locale,
) -> Result<impl IntoResponse, AppError> {
    tracing::info!("Resuming session {} with token {}", session_id, token);

    let session = state
        .db
        .get_session(&token)
        .await
        .reject("could not get session")?;

    let quiz_name = state
        .db
        .quiz_name(session.quiz_id)
        .await
        .reject("could not get quiz name")?;

    let question_idx = state
        .db
        .current_question_index(session.id)
        .await
        .reject("could not get current question index")?;

    let is_resuming = question_idx > 0;
    let page = views::titled(
        &quiz_name,
        super::question::question(
            &state.db,
            session.id,
            session.quiz_id,
            question_idx,
            is_resuming,
            &locale,
        )
        .await?,
    );
    let cookie = utils::cookie(
        names::QUIZ_SESSION_COOKIE_NAME,
        &token,
        state.secure_cookies,
    );
    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.parse().unwrap());

    Ok((headers, page))
}

pub(crate) async fn retry_incorrect(
    AuthGuard(user): AuthGuard,
    State(state): State<AppState>,
    Path(session_id): Path<i32>,
    Locale(locale): Locale,
) -> Result<axum::response::Response, AppError> {
    let session = state
        .db
        .get_session_by_id(session_id)
        .await
        .reject("could not get session")?;

    let incorrect_ids = state
        .db
        .get_incorrect_questions(session_id)
        .await
        .reject("could not get incorrect questions")?;

    if incorrect_ids.is_empty() {
        let page = views::titled(
            "Results",
            html! {
                p { (t!("result.no_incorrect", locale = &locale)) }
                button hx-get=(names::results_url(session_id))
                       hx-push-url="true"
                       hx-target="main" {
                    (t!("result.back_to_results", locale = &locale))
                }
            },
        );
        return Ok(page.into_response());
    }

    let suffix = &ulid::Ulid::new().to_string()[..6];
    let retry_name = format!("{}-retry-{}", session.name, suffix.to_lowercase());

    let token = state
        .db
        .create_session_with_questions(
            &retry_name,
            session.quiz_id,
            &incorrect_ids,
            "incorrect",
            user.id,
        )
        .await
        .reject("could not create retry session")?;

    let new_session = state
        .db
        .get_session(&token)
        .await
        .reject("could not get new session")?;

    let quiz_name = state
        .db
        .quiz_name(session.quiz_id)
        .await
        .reject("could not get quiz name")?;

    let page = views::titled(
        &quiz_name,
        super::question::question(
            &state.db,
            new_session.id,
            session.quiz_id,
            0,
            false,
            &locale,
        )
        .await?,
    );
    let cookie = utils::cookie(
        names::QUIZ_SESSION_COOKIE_NAME,
        &token,
        state.secure_cookies,
    );
    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.parse().unwrap());

    Ok((headers, page).into_response())
}

pub(crate) async fn retry_bookmarked(
    AuthGuard(user): AuthGuard,
    State(state): State<AppState>,
    Path(session_id): Path<i32>,
    Locale(locale): Locale,
) -> Result<axum::response::Response, AppError> {
    let session = state
        .db
        .get_session_by_id(session_id)
        .await
        .reject("could not get session")?;

    let bookmarked_ids = state
        .db
        .get_bookmarked_questions(session_id)
        .await
        .reject("could not get bookmarked questions")?;

    if bookmarked_ids.is_empty() {
        let page = views::titled(
            "Results",
            html! {
                p { (t!("result.no_bookmarked", locale = &locale)) }
                button hx-get=(names::results_url(session_id))
                       hx-push-url="true"
                       hx-target="main" {
                    (t!("result.back_to_results", locale = &locale))
                }
            },
        );
        return Ok(page.into_response());
    }

    let suffix = &ulid::Ulid::new().to_string()[..6];
    let retry_name = format!("{}-bm-{}", session.name, suffix.to_lowercase());

    let token = state
        .db
        .create_session_with_questions(
            &retry_name,
            session.quiz_id,
            &bookmarked_ids,
            "bookmarked",
            user.id,
        )
        .await
        .reject("could not create bookmarked retry session")?;

    let new_session = state
        .db
        .get_session(&token)
        .await
        .reject("could not get new session")?;

    let quiz_name = state
        .db
        .quiz_name(session.quiz_id)
        .await
        .reject("could not get quiz name")?;

    let page = views::titled(
        &quiz_name,
        super::question::question(
            &state.db,
            new_session.id,
            session.quiz_id,
            0,
            false,
            &locale,
        )
        .await?,
    );
    let cookie = utils::cookie(
        names::QUIZ_SESSION_COOKIE_NAME,
        &token,
        state.secure_cookies,
    );
    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.parse().unwrap());

    Ok((headers, page).into_response())
}

pub(crate) async fn delete_session(
    AuthGuard(_user): AuthGuard,
    State(state): State<AppState>,
    Path(session_id): Path<i32>,
    Locale(locale): Locale,
) -> Result<Markup, AppError> {
    let session = state
        .db
        .get_session_by_id(session_id)
        .await
        .reject("could not get session")?;
    let quiz_id = session.quiz_id;

    state
        .db
        .delete_session(session_id)
        .await
        .reject("could not delete session")?;

    Ok(views::titled(
        "Quiz Dashboard",
        super::dashboard::dashboard(&state.db, quiz_id, &locale).await?,
    ))
}

pub(crate) async fn rename_session(
    AuthGuard(_user): AuthGuard,
    State(state): State<AppState>,
    Path(session_id): Path<i32>,
    Locale(locale): Locale,
    Json(body): Json<super::RenameSessionBody>,
) -> Result<Markup, AppError> {
    let session = state
        .db
        .get_session_by_id(session_id)
        .await
        .reject("could not get session")?;
    let quiz_id = session.quiz_id;

    if let Err(e) = state
        .db
        .rename_session(session_id, &body.name, quiz_id)
        .await
    {
        tracing::warn!("could not rename session {session_id}: {e}");
    }

    Ok(views::titled(
        "Quiz Dashboard",
        super::dashboard::dashboard(&state.db, quiz_id, &locale).await?,
    ))
}

pub(crate) async fn page(
    db: &crate::db::Db,
    quiz_id: i32,
    locale: &str,
) -> Result<Markup, AppError> {
    let quiz_name = db
        .quiz_name(quiz_id)
        .await
        .reject("could not get quiz name")?;

    let total_questions = db
        .questions_count(quiz_id)
        .await
        .reject("could not get question count")?;

    Ok(quiz_views::start_page(
        quiz_views::StartPageData {
            quiz_name,
            total_questions,
            quiz_id,
        },
        locale,
    ))
}
