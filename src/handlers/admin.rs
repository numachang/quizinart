use axum::{extract::State, routing::get, Router};
use maud::Markup;

use crate::{
    extractors::{AuthGuard, IsHtmx, Locale},
    rejections::{AppError, ResultExt},
    views,
    views::admin as admin_views,
    AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new().route("/admin", get(admin_dashboard))
}

async fn admin_dashboard(
    AuthGuard(user): AuthGuard,
    IsHtmx(is_htmx): IsHtmx,
    State(state): State<AppState>,
    Locale(locale): Locale,
) -> Result<Markup, AppError> {
    if !user.is_admin {
        return Err(AppError::Forbidden);
    }

    let users = state
        .db
        .get_all_users_with_stats()
        .await
        .reject("could not get admin stats")?;

    Ok(views::render(
        is_htmx,
        "Admin Dashboard",
        admin_views::dashboard(&users, &locale),
        &locale,
        Some(&user.display_name),
    ))
}
