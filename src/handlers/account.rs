use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;

use crate::{
    extractors::{AuthGuard, IsHtmx, Locale},
    rejections::{AppError, ResultExt},
    views, AppState,
};

use crate::views::account as account_views;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/account", get(account_page))
        .route("/change-password", post(change_password_post))
}

async fn account_page(
    AuthGuard(user): AuthGuard,
    IsHtmx(is_htmx): IsHtmx,
    Locale(locale): Locale,
) -> maud::Markup {
    views::render(
        is_htmx,
        "Account",
        account_views::account_page(&user, account_views::ChangePasswordState::NoError, &locale),
        &locale,
        Some(&user.display_name),
    )
}

#[derive(Deserialize)]
struct ChangePasswordPost {
    current_password: String,
    new_password: String,
}

async fn change_password_post(
    AuthGuard(user): AuthGuard,
    State(state): State<AppState>,
    Locale(locale): Locale,
    Json(body): Json<ChangePasswordPost>,
) -> Result<axum::response::Response, AppError> {
    use crate::services::auth::ChangePasswordOutcome;

    let outcome = state
        .auth
        .change_password(user.id, user.is_demo, &body.current_password, &body.new_password)
        .await
        .reject("could not change password")?;

    let pw_state = match outcome {
        ChangePasswordOutcome::Success => account_views::ChangePasswordState::Success,
        ChangePasswordOutcome::EmptyFields => account_views::ChangePasswordState::EmptyFields,
        ChangePasswordOutcome::WeakPassword => account_views::ChangePasswordState::WeakPassword,
        ChangePasswordOutcome::IncorrectPassword => {
            account_views::ChangePasswordState::IncorrectPassword
        }
        ChangePasswordOutcome::DemoUser => account_views::ChangePasswordState::DemoUser,
    };

    Ok(views::titled(
        "Account",
        account_views::account_page(&user, pw_state, &locale),
    )
    .into_response())
}
