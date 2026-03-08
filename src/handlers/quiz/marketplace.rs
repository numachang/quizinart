use axum::extract::{Query, State};
use maud::Markup;
use rust_i18n::t;
use serde::Deserialize;

use crate::{
    extractors::{AuthGuard, IsHtmx, Locale},
    rejections::{AppError, ResultExt},
    views, AppState,
};

pub(crate) async fn marketplace_page(
    AuthGuard(user): AuthGuard,
    IsHtmx(is_htmx): IsHtmx,
    State(state): State<AppState>,
    Locale(locale): Locale,
) -> Result<Markup, AppError> {
    let (shared_quizzes, user_quiz_ids) = tokio::try_join!(
        state.db.list_shared_quizzes(),
        state.db.user_quiz_ids(user.id),
    )
    .reject("could not load marketplace data")?;

    let title = t!("marketplace.title", locale = &locale);
    let nav_user = views::NavUser {
        display_name: &user.display_name,
        is_admin: user.is_admin,
    };
    Ok(views::render(
        is_htmx,
        &title,
        views::marketplace::marketplace_page(&shared_quizzes, &user_quiz_ids, &locale),
        &locale,
        Some(&nav_user),
    ))
}

#[derive(Deserialize)]
pub(crate) struct SearchQuery {
    #[serde(default)]
    q: String,
}

pub(crate) async fn marketplace_search(
    AuthGuard(user): AuthGuard,
    State(state): State<AppState>,
    Locale(locale): Locale,
    Query(params): Query<SearchQuery>,
) -> Result<Markup, AppError> {
    let (shared_quizzes, user_quiz_ids) = tokio::try_join!(
        state.db.search_shared_quizzes(&params.q),
        state.db.user_quiz_ids(user.id),
    )
    .reject("could not search marketplace")?;

    Ok(views::marketplace::marketplace_results(
        &shared_quizzes,
        &user_quiz_ids,
        &locale,
    ))
}
