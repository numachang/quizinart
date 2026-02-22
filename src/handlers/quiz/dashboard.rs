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
    AuthGuard(user): AuthGuard,
    IsHtmx(is_htmx): IsHtmx,
    State(state): State<AppState>,
    Path(public_id): Path<String>,
    Locale(locale): Locale,
) -> Result<Markup, AppError> {
    let quiz_id = state
        .db
        .resolve_quiz_id(&public_id)
        .await
        .reject("quiz not found")?;

    Ok(views::render(
        is_htmx,
        "Quiz Dashboard",
        dashboard(&state.db, quiz_id, &public_id, &locale).await?,
        &locale,
        Some(&user.display_name),
    ))
}

pub(crate) async fn quiz_session_history(
    AuthGuard(user): AuthGuard,
    IsHtmx(is_htmx): IsHtmx,
    State(state): State<AppState>,
    Path(public_id): Path<String>,
    Locale(locale): Locale,
) -> Result<Markup, AppError> {
    let quiz_id = state
        .db
        .resolve_quiz_id(&public_id)
        .await
        .reject("quiz not found")?;

    Ok(views::render(
        is_htmx,
        "Session History",
        session_history(&state.db, quiz_id, &public_id, &locale).await?,
        &locale,
        Some(&user.display_name),
    ))
}

pub(crate) async fn session_result(
    AuthGuard(user): AuthGuard,
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

    let (
        questions_count,
        current_idx,
        correct_answers,
        answers,
        quiz_name,
        category_stats,
        quiz_public_id,
    ) = tokio::try_join!(
        state.db.questions_count_for_session(session.id),
        state.db.current_question_index(session.id),
        state.db.correct_answers(session.id),
        state.db.get_answers(session.id),
        state.db.quiz_name(session.quiz_id),
        state.db.get_category_stats(session.id),
        state.db.quiz_public_id(session.quiz_id),
    )
    .reject("could not get session result data")?;

    let is_complete = current_idx >= questions_count;
    let answered_count = current_idx;

    let page = quiz_views::session_result(
        quiz_views::SessionResultData {
            session_name: session.name,
            session_id,
            quiz_id: quiz_public_id,
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

    Ok(views::render(
        is_htmx,
        "Results",
        page,
        &locale,
        Some(&user.display_name),
    ))
}

pub async fn dashboard(
    db: &crate::db::Db,
    quiz_id: i32,
    quiz_public_id: &str,
    locale: &str,
) -> Result<Markup, AppError> {
    let (quiz_name, sessions_count, overall, cat_stats, daily_accuracy) = tokio::try_join!(
        db.quiz_name(quiz_id),
        db.sessions_count(quiz_id),
        db.get_quiz_overall_stats(quiz_id),
        db.get_quiz_category_stats(quiz_id),
        db.get_daily_accuracy(quiz_id),
    )
    .reject("could not get dashboard data")?;

    Ok(quiz_views::dashboard(
        quiz_views::DashboardData {
            quiz_name,
            quiz_id: quiz_public_id.to_string(),
            sessions_count,
            overall,
            cat_stats,
            daily_accuracy,
        },
        locale,
    ))
}

pub async fn session_history(
    db: &crate::db::Db,
    quiz_id: i32,
    quiz_public_id: &str,
    locale: &str,
) -> Result<Markup, AppError> {
    let (quiz_name, sessions) =
        tokio::try_join!(db.quiz_name(quiz_id), db.get_sessions_report(quiz_id),)
            .reject("could not get session history data")?;

    Ok(quiz_views::session_history(
        quiz_views::SessionHistoryData {
            quiz_name,
            quiz_id: quiz_public_id.to_string(),
            sessions,
        },
        locale,
    ))
}
