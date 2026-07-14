use axum::{
    extract::{Query, State},
    http::{
        header::{LOCATION, SET_COOKIE},
        HeaderMap, HeaderValue, StatusCode,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use axum_extra::extract::CookieJar;
use serde::Deserialize;
use ulid::Ulid;

use crate::{
    extractors::Locale,
    names,
    rejections::{AppError, ResultExt},
    utils, views, AppState,
};

use crate::views::homepage as homepage_views;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(names::GOOGLE_AUTH_URL, get(google_redirect))
        .route(names::GOOGLE_CALLBACK_URL, get(google_callback))
}

/// Start the Google OAuth flow: set an anti-CSRF state cookie and redirect
/// to Google's consent screen.
async fn google_redirect(
    State(state): State<AppState>,
) -> Result<axum::response::Response, AppError> {
    let Some(google) = &state.google else {
        return Err(AppError::Input("google login not configured"));
    };

    let oauth_state = Ulid::new().to_string();
    let state_cookie = utils::oauth_state_cookie(&oauth_state, state.secure_cookies)
        .reject("could not build oauth state cookie")?;
    let location = HeaderValue::from_str(&google.authorize_url(&oauth_state))
        .reject("could not build authorize url header")?;

    Ok((
        StatusCode::SEE_OTHER,
        [(SET_COOKIE, state_cookie), (LOCATION, location)],
        "",
    )
        .into_response())
}

#[derive(Deserialize)]
struct CallbackQuery {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

/// Handle the redirect back from Google: verify state, exchange the code,
/// then log the user in (creating the account on first sign-in).
async fn google_callback(
    State(state): State<AppState>,
    jar: CookieJar,
    Locale(locale): Locale,
    Query(query): Query<CallbackQuery>,
) -> Result<axum::response::Response, AppError> {
    use crate::services::auth::GoogleLoginOutcome;

    let Some(google) = &state.google else {
        return Err(AppError::Input("google login not configured"));
    };

    let clear_state = utils::clear_cookie(names::OAUTH_STATE_COOKIE_NAME, state.secure_cookies)
        .reject("could not build clear-state cookie")?;

    let cookie_state = jar
        .get(names::OAUTH_STATE_COOKIE_NAME)
        .map(|c| c.value().to_string());
    let state_matches =
        matches!((&query.state, &cookie_state), (Some(q), Some(c)) if q == c && !q.is_empty());

    let code = match (&query.error, state_matches, query.code) {
        (None, true, Some(code)) => code,
        (error, _, _) => {
            tracing::warn!(
                "google callback rejected: error={error:?}, state_matches={state_matches}"
            );
            return Ok(google_error_page(clear_state, &locale));
        }
    };

    let user = match google.fetch_user(&code).await {
        Ok(user) => user,
        Err(e) => {
            tracing::error!("google code exchange failed: {e}");
            return Ok(google_error_page(clear_state, &locale));
        }
    };

    let outcome = state
        .auth
        .login_with_google(&user)
        .await
        .reject("google login failed")?;

    match outcome {
        GoogleLoginOutcome::Success(session_token) => {
            let session_cookie = utils::cookie(
                names::USER_SESSION_COOKIE_NAME,
                &session_token,
                state.secure_cookies,
            )
            .reject("could not build session cookie")?;

            let mut headers = HeaderMap::new();
            headers.append(SET_COOKIE, clear_state);
            headers.append(SET_COOKIE, session_cookie);
            headers.insert(LOCATION, HeaderValue::from_static("/"));

            Ok((StatusCode::SEE_OTHER, headers, "").into_response())
        }
        GoogleLoginOutcome::EmailNotVerified => Ok(google_error_page(clear_state, &locale)),
    }
}

/// Show the login page with a Google error message, clearing the state cookie.
fn google_error_page(clear_state: HeaderValue, locale: &str) -> axum::response::Response {
    let page = views::page(
        "Log In",
        homepage_views::login(homepage_views::LoginState::GoogleError, true, locale),
        locale,
    );

    ([(SET_COOKIE, clear_state)], page).into_response()
}
