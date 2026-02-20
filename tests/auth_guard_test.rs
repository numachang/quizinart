mod common;

use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
};
use quizinart::{names, router, AppState};
use tower::ServiceExt;

async fn app() -> axum::Router {
    let db = common::create_test_db().await;
    router(AppState { db })
}

#[tokio::test]
async fn protected_quiz_routes_reject_direct_access_without_session_cookie() {
    let app = app().await;

    let cases = [
        (Method::GET, "/quiz/1", Body::empty()),
        (Method::GET, "/quiz/1/dashboard", Body::empty()),
        (Method::GET, "/results/1", Body::empty()),
        (Method::GET, "/question/1?question_idx=0", Body::empty()),
        (
            Method::POST,
            "/start-session/1",
            Body::from(r#"{"name":"u"}"#),
        ),
        (Method::POST, "/submit-answer", Body::from("option=1")),
    ];

    for (method, uri, body) in cases {
        let mut req = Request::builder().method(method).uri(uri);
        req = req.header("content-type", "application/json");
        let resp = app
            .clone()
            .oneshot(req.body(body).expect("request build should succeed"))
            .await
            .expect("router should respond");

        assert_eq!(
            resp.status(),
            StatusCode::UNAUTHORIZED,
            "expected UNAUTHORIZED for {uri}",
        );
    }
}

#[tokio::test]
async fn protected_quiz_routes_accept_requests_with_valid_user_session() {
    let db = common::create_test_db().await;
    let user_id = db
        .create_user("test@example.com", "password123", "Test User")
        .await
        .expect("create user");
    let session = db
        .create_user_session(user_id)
        .await
        .expect("create user session");

    let app = router(AppState { db });

    let req = Request::builder()
        .method(Method::GET)
        .uri("/quiz/1")
        .header(
            "cookie",
            format!("{}={}", names::USER_SESSION_COOKIE_NAME, session),
        )
        .body(Body::empty())
        .expect("request build should succeed");

    let resp = app.oneshot(req).await.expect("router should respond");

    assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn protected_routes_accept_legacy_admin_session_with_migration_user() {
    let db = common::create_test_db().await;

    // Set up legacy admin + create migration user (admin@local)
    db.set_admin_password("secret".to_string())
        .await
        .expect("set admin password");
    let admin_session = db
        .create_admin_session()
        .await
        .expect("create admin session");

    // Create the migration user that AuthGuard falls back to
    db.create_user("admin@local", "secret", "Admin")
        .await
        .expect("create migration user");

    let app = router(AppState { db });

    let req = Request::builder()
        .method(Method::GET)
        .uri("/quiz/1")
        .header(
            "cookie",
            format!("{}={}", names::ADMIN_SESSION_COOKIE_NAME, admin_session),
        )
        .body(Body::empty())
        .expect("request build should succeed");

    let resp = app.oneshot(req).await.expect("router should respond");

    assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
}
