use axum::extract::State;
use maud::Markup;
use rust_i18n::t;

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
    Ok(views::render(
        is_htmx,
        &title,
        views::marketplace::marketplace_page(&shared_quizzes, &user_quiz_ids, &locale),
        &locale,
        Some(&user.display_name),
    ))
}
