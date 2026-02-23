use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::IntoResponse,
};
use maud::Markup;
use rust_i18n::t;

use crate::{
    extractors::{AuthGuard, IsHtmx, Locale},
    names,
    rejections::{AppError, ResultExt},
    views,
    views::quiz as quiz_views,
    AppState,
};

pub(crate) async fn toggle_share(
    AuthGuard(user): AuthGuard,
    State(state): State<AppState>,
    Path(public_id): Path<String>,
    Locale(locale): Locale,
) -> Result<Markup, AppError> {
    let is_owner = state
        .db
        .verify_quiz_owner(&public_id, user.id)
        .await
        .reject("could not verify quiz owner")?;

    if !is_owner {
        return Err(AppError::Unauthorized);
    }

    let is_shared = state
        .db
        .toggle_share(&public_id, user.id)
        .await
        .reject("could not toggle share")?;

    Ok(quiz_views::share_toggle_icon(
        &public_id, is_shared, &locale,
    ))
}

pub(crate) async fn shared_quiz_page(
    AuthGuard(user): AuthGuard,
    IsHtmx(is_htmx): IsHtmx,
    State(state): State<AppState>,
    Path(public_id): Path<String>,
    Locale(locale): Locale,
) -> Result<Markup, AppError> {
    let quiz_info = state
        .db
        .get_shared_quiz(&public_id)
        .await
        .reject("could not get shared quiz")?;

    match quiz_info {
        Some(info) if info.is_shared => {
            let already_in_library = state
                .db
                .user_has_quiz(user.id, info.id)
                .await
                .reject("could not check library")?;

            Ok(views::render(
                is_htmx,
                &info.name,
                quiz_views::shared_quiz_page(&info, already_in_library, &locale),
                &locale,
                Some(&user.display_name),
            ))
        }
        _ => {
            let title = t!("share.not_available_title", locale = &locale);
            Ok(views::render(
                is_htmx,
                &title,
                quiz_views::shared_quiz_not_available(&locale),
                &locale,
                Some(&user.display_name),
            ))
        }
    }
}

pub(crate) async fn add_to_library(
    AuthGuard(user): AuthGuard,
    State(state): State<AppState>,
    Path(public_id): Path<String>,
    Locale(locale): Locale,
) -> Result<impl IntoResponse, AppError> {
    let quiz_info = state
        .db
        .get_shared_quiz(&public_id)
        .await
        .reject("could not get shared quiz")?;

    match quiz_info {
        Some(info) if info.is_shared => {
            state
                .db
                .add_quiz_to_library(user.id, info.id)
                .await
                .reject("could not add to library")?;

            let mut headers = HeaderMap::new();
            headers.insert(
                "HX-Push-Url",
                names::quiz_dashboard_url(&public_id).parse().unwrap(),
            );

            Ok((
                headers,
                views::titled(
                    "Quiz Dashboard",
                    super::dashboard::dashboard(&state.db, info.id, &public_id, &locale).await?,
                ),
            ))
        }
        _ => Err(AppError::Input("quiz not available for sharing")),
    }
}
