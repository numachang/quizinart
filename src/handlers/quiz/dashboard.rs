use axum::extract::{Path, State};
use maud::Markup;

use crate::{
    extractors::{AuthGuard, IsHtmx, Locale},
    rejections::{AppError, ResultExt},
    views,
    views::quiz as quiz_views,
    AppState,
};

pub(crate) async fn quiz_dashboard(
    _guard: AuthGuard,
    IsHtmx(is_htmx): IsHtmx,
    State(state): State<AppState>,
    Path(quiz_id): Path<i32>,
    Locale(locale): Locale,
) -> Result<Markup, AppError> {
    Ok(if is_htmx {
        views::titled(
            "Quiz Dashboard",
            dashboard(&state.db, quiz_id, &locale).await?,
        )
    } else {
        views::page(
            "Quiz Dashboard",
            dashboard(&state.db, quiz_id, &locale).await?,
            &locale,
        )
    })
}

pub(crate) async fn session_result(
    State(state): State<AppState>,
    IsHtmx(is_htmx): IsHtmx,
    Path(session_id): Path<i32>,
    Locale(locale): Locale,
) -> Result<Markup, AppError> {
    let session = state
        .db
        .get_session_by_id(session_id)
        .await
        .reject("could not get session")?;

    let questions_count = state
        .db
        .questions_count_for_session(session.id)
        .await
        .reject("could not get question count")?;

    let current_idx = state
        .db
        .current_question_index(session.id)
        .await
        .reject("could not get current question index")?;

    let is_complete = current_idx >= questions_count;
    let answered_count = current_idx;

    let correct_answers = state
        .db
        .correct_answers(session.id)
        .await
        .reject("could not get correct answer count")?;

    let answers = state
        .db
        .get_answers(session.id)
        .await
        .reject("could not get answers")?;

    let quiz_name = state
        .db
        .quiz_name(session.quiz_id)
        .await
        .reject("could not get quiz name")?;

    let category_stats = state
        .db
        .get_category_stats(session.id)
        .await
        .reject("could not get category stats")?;

    let page = quiz_views::session_result(
        quiz_views::SessionResultData {
            session_name: session.name,
            session_id,
            quiz_id: session.quiz_id,
            quiz_name,
            selection_mode: session
                .selection_mode
                .unwrap_or_else(|| "random".to_string()),
            questions_count,
            answered_count,
            is_complete,
            correct_answers,
            answers,
            category_stats,
        },
        &locale,
    );

    Ok(if is_htmx {
        views::titled("Results", page)
    } else {
        views::page("Results", page, &locale)
    })
}

pub async fn dashboard(db: &crate::db::Db, quiz_id: i32, locale: &str) -> Result<Markup, AppError> {
    let quiz_name = db
        .quiz_name(quiz_id)
        .await
        .reject("could not get quiz name")?;

    let sessions_count = db
        .sessions_count(quiz_id)
        .await
        .reject("could not get sessions count")?;

    let sessions = db
        .get_sessions_report(quiz_id)
        .await
        .reject("could not get sessions report")?;

    let overall = db
        .get_quiz_overall_stats(quiz_id)
        .await
        .reject("could not get quiz overall stats")?;

    let cat_stats = db
        .get_quiz_category_stats(quiz_id)
        .await
        .reject("could not get quiz category stats")?;

    let trends = db
        .get_session_category_trends(quiz_id)
        .await
        .reject("could not get session category trends")?;

    Ok(quiz_views::dashboard(
        quiz_views::DashboardData {
            quiz_name,
            quiz_id,
            sessions_count,
            sessions,
            overall,
            cat_stats,
            trends,
        },
        locale,
    ))
}
