mod common;

use common::create_test_db;
use quizinart::models::{Question, QuestionOption};

fn sample_questions() -> Vec<Question> {
    vec![Question {
        question: "What is 1+1?".to_string(),
        category: Some("Math".to_string()),
        is_multiple_choice: false,
        options: vec![
            QuestionOption {
                text: "1".to_string(),
                is_answer: false,
                explanation: None,
            },
            QuestionOption {
                text: "2".to_string(),
                is_answer: true,
                explanation: Some("Basic arithmetic".to_string()),
            },
        ],
    }]
}

fn minimal_questions() -> Vec<Question> {
    vec![Question {
        question: "Q1".to_string(),
        category: None,
        is_multiple_choice: false,
        options: vec![
            QuestionOption {
                text: "A".to_string(),
                is_answer: true,
                explanation: None,
            },
            QuestionOption {
                text: "B".to_string(),
                is_answer: false,
                explanation: None,
            },
        ],
    }]
}

#[tokio::test]
async fn test_db_connection() {
    let _db = create_test_db().await;
}

#[tokio::test]
async fn test_admin_password() {
    let db = create_test_db().await;

    // Initially no password
    let pw = db.admin_password().await.unwrap();
    assert!(pw.is_none());

    // Set password
    db.set_admin_password("secret".to_string()).await.unwrap();
    let pw = db.admin_password().await.unwrap();
    assert_eq!(pw, Some("secret".to_string()));
}

#[tokio::test]
async fn test_quiz_crud() {
    let db = create_test_db().await;

    let quiz_id = db.load_quiz("Test Quiz".to_string(), sample_questions()).await.unwrap();
    assert!(quiz_id > 0);

    let quizzes = db.quizzes().await.unwrap();
    assert_eq!(quizzes.len(), 1);
    assert_eq!(quizzes[0].name, "Test Quiz");
    assert_eq!(quizzes[0].count, 1);

    let name = db.quiz_name(quiz_id).await.unwrap();
    assert_eq!(name, "Test Quiz");

    let count = db.questions_count(quiz_id).await.unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn test_session_creation() {
    let db = create_test_db().await;

    let quiz_id = db.load_quiz("Quiz".to_string(), minimal_questions()).await.unwrap();
    let token = db.create_session("session-1", quiz_id, 5, "random").await.unwrap();
    assert!(!token.is_empty());

    let session = db.get_session(&token).await.unwrap();
    assert_eq!(session.name, "session-1");
    assert_eq!(session.quiz_id, quiz_id);
}

#[tokio::test]
async fn test_duplicate_session_name() {
    let db = create_test_db().await;

    let quiz_id = db.load_quiz("Quiz".to_string(), minimal_questions()).await.unwrap();
    db.create_session("dupe", quiz_id, 5, "random").await.unwrap();

    // Same name, same quiz -> should fail
    let result = db.create_session("dupe", quiz_id, 5, "random").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already in use"));
}

#[tokio::test]
async fn test_session_count() {
    let db = create_test_db().await;

    let quiz_id = db.load_quiz("Quiz".to_string(), minimal_questions()).await.unwrap();
    assert_eq!(db.sessions_count(quiz_id).await.unwrap(), 0);

    db.create_session("s1", quiz_id, 5, "random").await.unwrap();
    assert_eq!(db.sessions_count(quiz_id).await.unwrap(), 1);

    db.create_session("s2", quiz_id, 5, "random").await.unwrap();
    assert_eq!(db.sessions_count(quiz_id).await.unwrap(), 2);
}
