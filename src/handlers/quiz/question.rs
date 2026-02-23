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
    Path(public_id): Path<String>,
    jar: CookieJar,
    Locale(locale): Locale,
) -> Result<Markup, AppError> {
    let quiz_id = state
        .db
        .resolve_quiz_id(&public_id)
        .await
        .reject("quiz not found")?;

    if !state.db.user_has_quiz(user.id, quiz_id).await.reject("could not check access")? {
        return Err(AppError::Forbidden);
    }

    let token = jar
        .get(names::QUIZ_SESSION_COOKIE_NAME)
        .map(|c| c.value().to_string());

    let content = match token {
        Some(token) if !token.is_empty() => {
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
                    super::session::page(&state.db, quiz_id, &public_id, &locale).await?
                }
                Err(e) => {
                    tracing::error!("could not get session for {token}: {e}");
                    super::session::page(&state.db, quiz_id, &public_id, &locale).await?
                }
            }
        }
        _ => super::session::page(&state.db, quiz_id, &public_id, &locale).await?,
    };

    Ok(views::render(
        is_htmx,
        "Quiz",
        content,
        &locale,
        Some(&user.display_name),
    ))
}

pub(crate) async fn submit_answer_raw(
    AuthGuard(user): AuthGuard,
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
    submit_answer(state, token, body, user.id, &locale).await
}

async fn submit_answer(
    state: AppState,
    token: String,
    body: SubmitAnswerBody,
    user_id: i32,
    locale: &str,
) -> Result<axum::response::Response, AppError> {
    let session = state
        .db
        .get_session(&token)
        .await
        .reject("could not get session")?;

    if !state.db.verify_session_owner(session.id, user_id).await.reject("could not verify session owner")? {
        return Err(AppError::Forbidden);
    }

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

    // Parallel: current_question_index + questions_count (both only need session.id)
    let (question_idx, questions_count) = tokio::try_join!(
        state.db.current_question_index(session.id),
        state.db.questions_count_for_session(session.id),
    )
    .reject("could not get question state")?;

    let question_id = state
        .db
        .get_question_by_idx(session.id, question_idx)
        .await
        .reject("could not get question id")?;

    // Parallel: get_question + get_correct_option_ids (both only need question_id)
    let (question_data, correct_ids) = tokio::try_join!(
        state.db.get_question(question_id),
        state.db.get_correct_option_ids(question_id),
    )
    .reject("could not get question data")?;

    let is_correct = if question_data.is_multiple_choice {
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
    };

    // Parallel: create_answers_batch + update_question_result (independent writes)
    tokio::try_join!(
        state
            .db
            .create_answers_batch(session.id, question_id, &selected_ids, is_correct),
        state
            .db
            .update_question_result(session.id, question_id, is_correct),
    )
    .reject("could not save answer")?;

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
        let cookie = utils::clear_cookie(names::QUIZ_SESSION_COOKIE_NAME, state.secure_cookies);
        let mut headers = HeaderMap::new();
        headers.insert(SET_COOKIE, cookie.parse().unwrap());
        Ok((headers, page).into_response())
    } else {
        Ok(page.into_response())
    }
}

pub(crate) async fn navigate_question(
    AuthGuard(user): AuthGuard,
    State(state): State<AppState>,
    IsHtmx(_is_htmx): IsHtmx,
    Path(session_id): Path<i32>,
    Query(query): Query<NavigateQuestionQuery>,
    Locale(locale): Locale,
) -> Result<Markup, AppError> {
    if !state.db.verify_session_owner(session_id, user.id).await.reject("could not verify session owner")? {
        return Err(AppError::Forbidden);
    }

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
    AuthGuard(user): AuthGuard,
    State(state): State<AppState>,
    Path((session_id, question_id)): Path<(i32, i32)>,
    Locale(locale): Locale,
) -> Result<Markup, AppError> {
    if !state.db.verify_session_owner(session_id, user.id).await.reject("could not verify session owner")? {
        return Err(AppError::Forbidden);
    }

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
// Phase 1 optimization: merged queries (7 → 2 for question, 5 → 2 for answer)

pub async fn question(
    db: &crate::db::Db,
    session_id: i32,
    quiz_id: i32,
    question_idx: i32,
    is_resuming: bool,
    locale: &str,
) -> Result<Markup, AppError> {
    let ctx = db
        .get_question_context(session_id, quiz_id, question_idx)
        .await
        .reject("could not get question context")?;

    let options_with_sel = db
        .get_options_with_selection(session_id, ctx.question_id)
        .await
        .reject("could not get options")?;

    let selected_answers: Vec<i32> = options_with_sel
        .iter()
        .filter(|o| o.is_selected)
        .map(|o| o.id)
        .collect();

    let options: Vec<crate::db::QuestionOptionModel> = options_with_sel
        .into_iter()
        .map(|o| crate::db::QuestionOptionModel {
            id: o.id,
            is_answer: o.is_answer,
            option: o.option,
            explanation: o.explanation,
        })
        .collect();

    Ok(quiz_views::question(
        quiz_views::QuestionData {
            quiz_name: ctx.quiz_name,
            question: crate::db::QuestionModel {
                question: ctx.question,
                is_multiple_choice: ctx.is_multiple_choice,
                options,
            },
            question_idx,
            questions_count: ctx.questions_count,
            is_answered: ctx.is_answered,
            selected_answers,
            is_resuming,
            session_id,
            question_id: ctx.question_id,
            is_bookmarked: ctx.is_bookmarked,
            quiz_id: ctx.quiz_public_id,
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
    let ctx = db
        .get_question_context(session_id, quiz_id, question_idx)
        .await
        .reject("could not get question context")?;

    let options = db
        .get_options(ctx.question_id)
        .await
        .reject("could not get options")?;

    Ok(quiz_views::answer(
        quiz_views::AnswerData {
            quiz_name: ctx.quiz_name,
            question: crate::db::QuestionModel {
                question: ctx.question,
                is_multiple_choice: ctx.is_multiple_choice,
                options,
            },
            question_idx,
            questions_count: ctx.questions_count,
            session_id,
            quiz_id: ctx.quiz_public_id,
            selected,
            from_context,
            current_idx,
            question_id: ctx.question_id,
            is_bookmarked: ctx.is_bookmarked,
        },
        locale,
    ))
}
