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
async fn protected_quiz_routes_reject_direct_access_without_admin_cookie() {
    let app = app().await;

    let cases = [
        (Method::GET, "/quiz/1", Body::empty()),
        (Method::GET, "/quiz/1/dashboard", Body::empty()),
        (Method::GET, "/results/1", Body::empty()),
        (Method::GET, "/question/1?question_idx=0", Body::empty()),
        (
            Method::POST,
            "/start-session/1",
            Body::from(r#"{\"name\":\"u\"}"#),
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
async fn protected_quiz_routes_accept_requests_with_valid_admin_cookie() {
    let db = common::create_test_db().await;
    db.set_admin_password("secret".to_string())
        .await
        .expect("set admin password");
    let session = db
        .create_admin_session()
        .await
        .expect("create admin session");

    let app = router(AppState { db });

    let req = Request::builder()
        .method(Method::GET)
        .uri("/quiz/1")
        .header(
            "cookie",
            format!("{}={}", names::ADMIN_SESSION_COOKIE_NAME, session),
        )
        .body(Body::empty())
        .expect("request build should succeed");

    let resp = app.oneshot(req).await.expect("router should respond");

    assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
}
