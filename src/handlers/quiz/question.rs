use axum::{
    extract::{Path, Query, State},
    http::{header::SET_COOKIE, HeaderMap},
};
use axum_extra::extract::CookieJar;
use maud::Markup;

use super::{NavigateQuestionQuery, SubmitAnswerBody};
use crate::{
    extractors::{AuthGuard, IsHtmx, Locale},
    names,
    rejections::{AppError, ResultExt},
    utils, views,
    views::quiz as quiz_views,
    AppState,
};

pub(crate) async fn quiz_page(
    AuthGuard(user): AuthGuard,
    IsHtmx(is_htmx): IsHtmx,
    State(state): State<AppState>,
    Path(quiz_id): Path<i32>,
    jar: CookieJar,
    Locale(locale): Locale,
) -> Result<Markup, AppError> {
    let token = jar
        .get(names::QUIZ_SESSION_COOKIE_NAME)
        .map(|c| c.value().to_string());

    let content = match token {
        Some(token) => {
            let res = state.db.get_session(&token).await;

            match res {
                Ok(session) if session.quiz_id == quiz_id => {
                    let question_idx = state
                        .db
                        .current_question_index(session.id)
                        .await
                        .reject("could not get current question index")?;
                    question(
                        &state.db,
                        session.id,
                        session.quiz_id,
                        question_idx,
                        false,
                        &locale,
                    )
                    .await?
                }
                Ok(_) => {
                    // Session belongs to a different quiz; show start page for this quiz
                    super::session::page(&state.db, quiz_id, &locale).await?
                }
                Err(e) => {
                    tracing::error!("could not get session for {token}: {e}");
                    super::session::page(&state.db, quiz_id, &locale).await?
                }
            }
        }
        None => super::session::page(&state.db, quiz_id, &locale).await?,
    };

    if is_htmx {
        Ok(views::titled("Quiz", content))
    } else {
        Ok(views::page_with_user(
            "Quiz",
            content,
            &locale,
            Some(&user.display_name),
        ))
    }
}

pub(crate) async fn submit_answer_raw(
    AuthGuard(_user): AuthGuard,
    State(state): State<AppState>,
    jar: CookieJar,
    Locale(locale): Locale,
    body_bytes: bytes::Bytes,
) -> Result<axum::response::Response, AppError> {
    let token = jar
        .get(names::QUIZ_SESSION_COOKIE_NAME)
        .map(|c| c.value().to_string())
        .ok_or(AppError::Input("session cookie not found"))?;

    let body_str =
        String::from_utf8(body_bytes.to_vec()).reject_input("failed to parse body as UTF-8")?;

    let mut option: Option<String> = None;
    let mut options: Vec<String> = Vec::new();

    for pair in body_str.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            let decoded_value = urlencoding::decode(value)
                .map_err(|e| {
                    tracing::error!("failed to decode URL value: {e}");
                    AppError::Input("failed to decode URL value")
                })?
                .to_string();

            match key {
                "option" => option = Some(decoded_value),
                "options" => options.push(decoded_value),
                _ => {}
            }
        }
    }

    tracing::info!("Received body: option={:?}, options={:?}", option, options);

    let body = SubmitAnswerBody { option, options };
    submit_answer(state, token, body, &locale).await
}

async fn submit_answer(
    state: AppState,
    token: String,
    body: SubmitAnswerBody,
    locale: &str,
) -> Result<axum::response::Response, AppError> {
    let session = state
        .db
        .get_session(&token)
        .await
        .reject("could not get session")?;

    let selected_ids: Vec<i32> = if !body.options.is_empty() {
        body.options
            .iter()
            .filter_map(|s| s.parse::<i32>().ok())
            .collect()
    } else if let Some(option) = body.option {
        vec![option
            .parse::<i32>()
            .reject_input("failed to parse option id")?]
    } else {
        tracing::error!("no options provided");
        return Err(AppError::Input("no options provided"));
    };

    let question_idx = state
        .db
        .current_question_index(session.id)
        .await
        .reject("could not get current question index")?;

    let question_id = state
        .db
        .get_question_by_idx(session.id, question_idx)
        .await
        .reject("could not get question id")?;

    let question_data = state
        .db
        .get_question(question_id)
        .await
        .reject("could not get question")?;

    let is_correct = {
        let correct_ids = state
            .db
            .get_correct_option_ids(question_id)
            .await
            .reject("could not get correct option ids")?;

        if question_data.is_multiple_choice {
            let mut selected_sorted = selected_ids.clone();
            selected_sorted.sort();
            let mut correct_sorted = correct_ids.clone();
            correct_sorted.sort();

            tracing::info!(
                "Multiple choice validation: selected={:?}, correct={:?}, match={}",
                selected_sorted,
                correct_sorted,
                selected_sorted == correct_sorted
            );

            selected_sorted == correct_sorted
        } else {
            correct_ids.contains(&selected_ids[0])
        }
    };

    for option_id in &selected_ids {
        state
            .db
            .create_answer(session.id, question_id, *option_id, is_correct)
            .await
            .reject("could not create answer")?;
    }

    state
        .db
        .update_question_result(session.id, question_id, is_correct)
        .await
        .reject("could not update question result")?;

    let questions_count = state
        .db
        .questions_count_for_session(session.id)
        .await
        .reject("could not get question count")?;

    let is_final = question_idx + 1 == questions_count;

    let page = answer(
        &state.db,
        session.id,
        session.quiz_id,
        question_idx,
        selected_ids,
        None,
        None,
        locale,
    )
    .await?;

    use axum::response::IntoResponse;
    if is_final {
        let cookie = utils::cookie(names::QUIZ_SESSION_COOKIE_NAME, "", state.secure_cookies);
        let mut headers = HeaderMap::new();
        headers.insert(SET_COOKIE, cookie.parse().unwrap());
        Ok((headers, page).into_response())
    } else {
        Ok(page.into_response())
    }
}

pub(crate) async fn navigate_question(
    AuthGuard(_user): AuthGuard,
    State(state): State<AppState>,
    IsHtmx(_is_htmx): IsHtmx,
    Path(session_id): Path<i32>,
    Query(query): Query<NavigateQuestionQuery>,
    Locale(locale): Locale,
) -> Result<Markup, AppError> {
    let session = state
        .db
        .get_session_by_id(session_id)
        .await
        .reject("could not get session")?;

    let quiz_name = state
        .db
        .quiz_name(session.quiz_id)
        .await
        .reject("could not get quiz name")?;

    let question_id = state
        .db
        .get_question_by_idx(session_id, query.question_idx)
        .await
        .reject("could not get question id")?;

    let is_answered = state
        .db
        .is_question_answered(session_id, question_id)
        .await
        .reject("could not check if question is answered")?;

    let page = if is_answered {
        let selected_answers = state
            .db
            .get_selected_answers(session_id, question_id)
            .await
            .reject("could not get selected answers")?;

        answer(
            &state.db,
            session_id,
            session.quiz_id,
            query.question_idx,
            selected_answers,
            query.from.clone(),
            query.current_idx,
            &locale,
        )
        .await?
    } else {
        question(
            &state.db,
            session_id,
            session.quiz_id,
            query.question_idx,
            false,
            &locale,
        )
        .await?
    };

    Ok(views::titled(&quiz_name, page))
}

pub(crate) async fn toggle_bookmark(
    AuthGuard(_user): AuthGuard,
    State(state): State<AppState>,
    Path((session_id, question_id)): Path<(i32, i32)>,
    Locale(locale): Locale,
) -> Result<Markup, AppError> {
    let new_state = state
        .db
        .toggle_bookmark(session_id, question_id)
        .await
        .reject("could not toggle bookmark")?;

    Ok(quiz_views::bookmark_button(
        session_id,
        question_id,
        new_state,
        &locale,
    ))
}

// --- Helper functions: DB queries + view delegation ---

pub async fn question(
    db: &crate::db::Db,
    session_id: i32,
    quiz_id: i32,
    question_idx: i32,
    is_resuming: bool,
    locale: &str,
) -> Result<Markup, AppError> {
    let quiz_name = db
        .quiz_name(quiz_id)
        .await
        .reject("could not get quiz name")?;

    let question_id = db
        .get_question_by_idx(session_id, question_idx)
        .await
        .reject("could not get question id")?;

    let question_data = db
        .get_question(question_id)
        .await
        .reject("could not get question")?;

    let questions_count = db
        .questions_count_for_session(session_id)
        .await
        .reject("could not get question count")?;

    let is_answered = db
        .is_question_answered(session_id, question_id)
        .await
        .reject("could not check if question is answered")?;

    let selected_answers = db
        .get_selected_answers(session_id, question_id)
        .await
        .reject("could not get selected answers")?;

    let is_bookmarked = db
        .is_question_bookmarked(session_id, question_id)
        .await
        .reject("could not check bookmark status")?;

    Ok(quiz_views::question(
        quiz_views::QuestionData {
            quiz_name,
            question: question_data,
            question_idx,
            questions_count,
            is_answered,
            selected_answers,
            is_resuming,
            session_id,
            question_id,
            is_bookmarked,
        },
        locale,
    ))
}

#[allow(clippy::too_many_arguments)]
pub async fn answer(
    db: &crate::db::Db,
    session_id: i32,
    quiz_id: i32,
    question_idx: i32,
    selected: Vec<i32>,
    from_context: Option<String>,
    current_idx: Option<i32>,
    locale: &str,
) -> Result<Markup, AppError> {
    let quiz_name = db
        .quiz_name(quiz_id)
        .await
        .reject("could not get quiz name")?;

    let question_id = db
        .get_question_by_idx(session_id, question_idx)
        .await
        .reject("could not get question id")?;

    let question_data = db
        .get_question(question_id)
        .await
        .reject("could not get question")?;

    let questions_count = db
        .questions_count_for_session(session_id)
        .await
        .reject("could not get question count")?;

    let is_bookmarked = db
        .is_question_bookmarked(session_id, question_id)
        .await
        .reject("could not check bookmark status")?;

    Ok(quiz_views::answer(
        quiz_views::AnswerData {
            quiz_name,
            question: question_data,
            question_idx,
            questions_count,
            session_id,
            quiz_id,
            selected,
            from_context,
            current_idx,
            question_id,
            is_bookmarked,
        },
        locale,
    ))
}
