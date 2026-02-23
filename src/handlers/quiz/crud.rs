use std::collections::HashMap;

use axum::{
    extract::{Form, Multipart, State},
    http::HeaderMap,
    response::IntoResponse,
    routing::{delete, patch, post},
    Router,
};
use rust_i18n::t;
use serde::Deserialize;

use crate::{
    extractors::{AuthGuard, Locale},
    models, names,
    rejections::{AppError, ResultExt},
    views, AppState,
};

use crate::views::homepage as homepage_views;

use super::dashboard;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/create-quiz", post(create_quiz))
        .route("/delete-quiz/{id}", delete(delete_quiz))
        .route("/rename-quiz/{id}", patch(rename_quiz))
}

async fn create_quiz(
    AuthGuard(user): AuthGuard,
    State(state): State<AppState>,
    Locale(locale): Locale,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let mut field_names: HashMap<String, String> = HashMap::new();

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        tracing::error!("failed to read multipart field: {e}");
        AppError::Input("failed to read multipart field")
    })? {
        let name = field.name().unwrap_or_default().to_string();
        let text = field.text().await.map_err(|e| {
            tracing::error!("failed to read field data: {e}");
            AppError::Input("failed to read field data")
        })?;
        field_names.insert(name, text);
    }

    let quiz_name = field_names
        .remove("quiz_name")
        .ok_or(AppError::Input("missing quiz_name field"))?;

    let quiz_file = field_names
        .remove("quiz_file")
        .ok_or(AppError::Input("missing quiz_file field"))?;

    let questions = serde_json::from_str::<models::Questions>(&quiz_file)
        .reject_input("failed to decode quiz file")?;

    let public_id = state
        .db
        .load_quiz(quiz_name, questions, user.id)
        .await
        .reject_input("failed to load quiz")?;

    let quiz_id = state
        .db
        .resolve_quiz_id(&public_id)
        .await
        .reject("failed to resolve quiz")?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "HX-Replace-Url",
        names::quiz_dashboard_url(&public_id).parse().unwrap(),
    );

    Ok((
        headers,
        views::titled(
            "Quiz Dashboard",
            dashboard::dashboard(&state.db, quiz_id, &public_id, user.id, &locale).await?,
        ),
    ))
}

async fn delete_quiz(
    AuthGuard(user): AuthGuard,
    State(state): State<AppState>,
    Locale(locale): Locale,
    axum::extract::Path(public_id): axum::extract::Path<String>,
) -> Result<maud::Markup, AppError> {
    let has_others = state
        .db
        .quiz_has_other_users(&public_id, user.id)
        .await
        .reject("failed to check quiz users")?;

    if has_others {
        let quizzes = state
            .db
            .quizzes(user.id)
            .await
            .reject("failed to get quizzes")?;
        let msg = t!("homepage.delete_blocked", locale = locale);
        return Ok(views::titled(
            "My Quizzes",
            homepage_views::quiz_list_with_error(quizzes, &locale, Some(&msg)),
        ));
    }

    state
        .db
        .delete_quiz(&public_id, user.id)
        .await
        .reject("failed to delete quiz")?;

    let quizzes = state
        .db
        .quizzes(user.id)
        .await
        .reject("failed to get quizzes")?;
    Ok(views::titled(
        "My Quizzes",
        homepage_views::quiz_list(quizzes, &locale),
    ))
}

#[derive(Deserialize)]
struct RenameQuizBody {
    name: String,
}

async fn rename_quiz(
    AuthGuard(user): AuthGuard,
    State(state): State<AppState>,
    Locale(locale): Locale,
    axum::extract::Path(public_id): axum::extract::Path<String>,
    Form(body): Form<RenameQuizBody>,
) -> Result<maud::Markup, AppError> {
    state
        .db
        .rename_quiz(&public_id, &body.name, user.id)
        .await
        .reject("failed to rename quiz")?;

    let quizzes = state
        .db
        .quizzes(user.id)
        .await
        .reject("failed to get quizzes")?;
    Ok(views::titled(
        "My Quizzes",
        homepage_views::quiz_list(quizzes, &locale),
    ))
}
