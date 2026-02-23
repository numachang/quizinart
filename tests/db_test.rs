mod common;

use std::collections::HashSet;

use common::create_test_db;
use quizinart::db::Db;
use quizinart::models::{Question, QuestionOption};

/// Helper: create a test user and return their id
async fn create_test_user(db: &Db) -> i32 {
    db.create_user("test@example.com", "password123", "Test User")
        .await
        .expect("create test user")
}

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

fn make_questions(n: usize) -> Vec<Question> {
    (0..n)
        .map(|i| Question {
            question: format!("Question {}", i + 1),
            category: Some(format!("Category {}", i % 3)),
            is_multiple_choice: false,
            options: vec![
                QuestionOption {
                    text: format!("Correct {}", i + 1),
                    is_answer: true,
                    explanation: None,
                },
                QuestionOption {
                    text: format!("Wrong {}", i + 1),
                    is_answer: false,
                    explanation: None,
                },
            ],
        })
        .collect()
}

/// Helper: load a quiz and return (public_id, internal quiz_id)
async fn load_quiz_with_id(
    db: &Db,
    name: &str,
    questions: Vec<Question>,
    user_id: i32,
) -> (String, i32) {
    let public_id = db
        .load_quiz(name.to_string(), questions, user_id)
        .await
        .expect("load quiz");
    let quiz_id = db
        .resolve_quiz_id(&public_id)
        .await
        .expect("resolve quiz id");
    (public_id, quiz_id)
}

async fn get_session_question_ids(db: &Db, session_id: i32) -> Vec<i32> {
    let count = db.questions_count_for_session(session_id).await.unwrap();
    let mut ids = Vec::new();
    for idx in 0..count {
        ids.push(db.get_question_by_idx(session_id, idx).await.unwrap());
    }
    ids
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
    // Verifies that Db::new() succeeds, which means all sqlx migrations were applied
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
async fn test_admin_password_can_be_updated() {
    let db = create_test_db().await;

    db.set_admin_password("secret-1".to_string()).await.unwrap();
    db.set_admin_password("secret-2".to_string()).await.unwrap();

    let pw = db.admin_password().await.unwrap();
    assert_eq!(pw, Some("secret-2".to_string()));
}

#[tokio::test]
async fn test_quiz_crud() {
    let db = create_test_db().await;
    let user_id = create_test_user(&db).await;

    let (public_id, quiz_id) =
        load_quiz_with_id(&db, "Test Quiz", sample_questions(), user_id).await;
    assert!(!public_id.is_empty());

    let quizzes = db.quizzes(user_id).await.unwrap();
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
    let user_id = create_test_user(&db).await;

    let (_public_id, quiz_id) = load_quiz_with_id(&db, "Quiz", minimal_questions(), user_id).await;
    let (token, session_id) = db
        .create_session("session-1", quiz_id, 5, "random", user_id)
        .await
        .unwrap();
    assert!(!token.is_empty());
    assert!(session_id > 0);

    let session = db.get_session(&token).await.unwrap();
    assert_eq!(session.name, "session-1");
    assert_eq!(session.quiz_id, quiz_id);
}

#[tokio::test]
async fn test_duplicate_session_name() {
    let db = create_test_db().await;
    let user_id = create_test_user(&db).await;

    let (_public_id, quiz_id) = load_quiz_with_id(&db, "Quiz", minimal_questions(), user_id).await;
    db.create_session("dupe", quiz_id, 5, "random", user_id)
        .await
        .unwrap();

    // Same name, same quiz -> should fail
    let result = db
        .create_session("dupe", quiz_id, 5, "random", user_id)
        .await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already in use"));
}

#[tokio::test]
async fn test_session_count() {
    let db = create_test_db().await;
    let user_id = create_test_user(&db).await;

    let (_public_id, quiz_id) = load_quiz_with_id(&db, "Quiz", minimal_questions(), user_id).await;
    assert_eq!(db.sessions_count(quiz_id).await.unwrap(), 0);

    db.create_session("s1", quiz_id, 5, "random", user_id)
        .await
        .unwrap();
    assert_eq!(db.sessions_count(quiz_id).await.unwrap(), 1);

    db.create_session("s2", quiz_id, 5, "random", user_id)
        .await
        .unwrap();
    assert_eq!(db.sessions_count(quiz_id).await.unwrap(), 2);
}

#[tokio::test]
async fn test_delete_session() {
    let db = create_test_db().await;
    let user_id = create_test_user(&db).await;

    let (_public_id, quiz_id) = load_quiz_with_id(&db, "Quiz", minimal_questions(), user_id).await;
    let (token, session_id) = db
        .create_session("to-delete", quiz_id, 5, "random", user_id)
        .await
        .unwrap();

    assert_eq!(db.sessions_count(quiz_id).await.unwrap(), 1);

    db.delete_session(session_id).await.unwrap();
    assert_eq!(db.sessions_count(quiz_id).await.unwrap(), 0);

    // Session should no longer be retrievable
    assert!(db.get_session(&token).await.is_err());
}

#[tokio::test]
async fn test_rename_session() {
    let db = create_test_db().await;
    let user_id = create_test_user(&db).await;

    let (_public_id, quiz_id) = load_quiz_with_id(&db, "Quiz", minimal_questions(), user_id).await;
    let (_, session_id) = db
        .create_session("old-name", quiz_id, 5, "random", user_id)
        .await
        .unwrap();

    db.rename_session(session_id, "new-name", quiz_id)
        .await
        .unwrap();

    let renamed = db.get_session_by_id(session_id).await.unwrap();
    assert_eq!(renamed.name, "new-name");
}

#[tokio::test]
async fn test_rename_session_duplicate() {
    let db = create_test_db().await;
    let user_id = create_test_user(&db).await;

    let (_public_id, quiz_id) = load_quiz_with_id(&db, "Quiz", minimal_questions(), user_id).await;
    db.create_session("existing", quiz_id, 5, "random", user_id)
        .await
        .unwrap();
    let (_, session_id2) = db
        .create_session("to-rename", quiz_id, 5, "random", user_id)
        .await
        .unwrap();

    // Renaming to an existing name should fail
    let result = db.rename_session(session_id2, "existing", quiz_id).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already in use"));
}

// --- Question selection tests ---

#[tokio::test]
async fn test_random_mode_no_duplicates() {
    let db = create_test_db().await;
    let user_id = create_test_user(&db).await;
    let (_public_id, quiz_id) = load_quiz_with_id(&db, "Quiz", make_questions(10), user_id).await;

    let (_, session_id) = db
        .create_session("random-session", quiz_id, 5, "random", user_id)
        .await
        .unwrap();
    let ids = get_session_question_ids(&db, session_id).await;

    assert_eq!(ids.len(), 5, "Should select exactly 5 questions");

    let unique: HashSet<i32> = ids.iter().cloned().collect();
    assert_eq!(
        unique.len(),
        ids.len(),
        "Random mode produced duplicate questions: {:?}",
        ids
    );
}

#[tokio::test]
async fn test_random_mode_cap_at_total() {
    let db = create_test_db().await;
    let user_id = create_test_user(&db).await;
    let (_public_id, quiz_id) = load_quiz_with_id(&db, "Quiz", make_questions(3), user_id).await;

    // Request more questions than exist
    let (_, session_id) = db
        .create_session("random-big", quiz_id, 10, "random", user_id)
        .await
        .unwrap();
    let ids = get_session_question_ids(&db, session_id).await;

    assert_eq!(
        ids.len(),
        3,
        "Should cap at total available questions (3), got {}",
        ids.len()
    );

    let unique: HashSet<i32> = ids.iter().cloned().collect();
    assert_eq!(unique.len(), ids.len(), "Duplicates in capped selection");
}

#[tokio::test]
async fn test_unanswered_mode_no_duplicates() {
    let db = create_test_db().await;
    let user_id = create_test_user(&db).await;
    let (_public_id, quiz_id) = load_quiz_with_id(&db, "Quiz", make_questions(10), user_id).await;

    // Session 1: pick 4 unanswered questions
    let (_, s1_id) = db
        .create_session("s1", quiz_id, 4, "unanswered", user_id)
        .await
        .unwrap();
    let ids1 = get_session_question_ids(&db, s1_id).await;

    assert_eq!(ids1.len(), 4);
    let unique1: HashSet<i32> = ids1.iter().cloned().collect();
    assert_eq!(
        unique1.len(),
        ids1.len(),
        "Session 1 has duplicates: {:?}",
        ids1
    );

    // Session 2: pick 4 more unanswered questions — should NOT overlap with session 1
    let (_, s2_id) = db
        .create_session("s2", quiz_id, 4, "unanswered", user_id)
        .await
        .unwrap();
    let ids2 = get_session_question_ids(&db, s2_id).await;

    assert_eq!(ids2.len(), 4);
    let unique2: HashSet<i32> = ids2.iter().cloned().collect();
    assert_eq!(
        unique2.len(),
        ids2.len(),
        "Session 2 has duplicates: {:?}",
        ids2
    );

    // No overlap between sessions (10 total, 4+4 = 8 asked, 2 still unanswered)
    let overlap: HashSet<&i32> = unique1.intersection(&unique2).collect();
    assert!(
        overlap.is_empty(),
        "Unanswered sessions should not overlap when enough questions exist. Overlap: {:?}",
        overlap
    );
}

#[tokio::test]
async fn test_unanswered_mode_fallback_no_duplicates() {
    let db = create_test_db().await;
    let user_id = create_test_user(&db).await;
    let (_public_id, quiz_id) = load_quiz_with_id(&db, "Quiz", make_questions(5), user_id).await;

    // Session 1: exhaust all 5 questions
    let (_, s1_id) = db
        .create_session("s1", quiz_id, 5, "unanswered", user_id)
        .await
        .unwrap();
    let ids1 = get_session_question_ids(&db, s1_id).await;
    assert_eq!(ids1.len(), 5);

    // Session 2: no unanswered left — fallback fills from already-asked
    let (_, s2_id) = db
        .create_session("s2", quiz_id, 3, "unanswered", user_id)
        .await
        .unwrap();
    let ids2 = get_session_question_ids(&db, s2_id).await;

    assert_eq!(
        ids2.len(),
        3,
        "Fallback should still give 3 questions, got {}",
        ids2.len()
    );
    let unique2: HashSet<i32> = ids2.iter().cloned().collect();
    assert_eq!(
        unique2.len(),
        ids2.len(),
        "Fallback session has duplicate questions: {:?}",
        ids2
    );
}

#[tokio::test]
async fn test_unanswered_mode_partial_fallback() {
    let db = create_test_db().await;
    let user_id = create_test_user(&db).await;
    let (_public_id, quiz_id) = load_quiz_with_id(&db, "Quiz", make_questions(6), user_id).await;

    // Session 1: use 4 out of 6
    let (_, s1_id) = db
        .create_session("s1", quiz_id, 4, "unanswered", user_id)
        .await
        .unwrap();
    let ids1 = get_session_question_ids(&db, s1_id).await;
    assert_eq!(ids1.len(), 4);

    // Session 2: request 4, only 2 unanswered remain → 2 unanswered + 2 fill
    let (_, s2_id) = db
        .create_session("s2", quiz_id, 4, "unanswered", user_id)
        .await
        .unwrap();
    let ids2 = get_session_question_ids(&db, s2_id).await;

    assert_eq!(ids2.len(), 4, "Should get 4 questions (2 new + 2 fill)");
    let unique2: HashSet<i32> = ids2.iter().cloned().collect();
    assert_eq!(
        unique2.len(),
        ids2.len(),
        "Partial fallback produced duplicates: {:?}",
        ids2
    );

    // The 2 remaining unanswered questions should be included
    let all_question_ids: HashSet<i32> = {
        let mut all = HashSet::new();
        for &id in &ids1 {
            all.insert(id);
        }
        for &id in &ids2 {
            all.insert(id);
        }
        all
    };

    // With 6 questions total and 4+4 selections, we should cover at least 6
    // (session 2 must include the 2 unanswered ones that session 1 missed)
    assert_eq!(
        all_question_ids.len(),
        6,
        "Both sessions combined should cover all 6 questions"
    );
}

#[tokio::test]
async fn test_create_session_with_questions_deduplicates_question_ids() {
    let db = create_test_db().await;
    let user_id = create_test_user(&db).await;
    let (_public_id, quiz_id) = load_quiz_with_id(&db, "Quiz", make_questions(5), user_id).await;

    let mut all_questions = Vec::new();
    for idx in 0..5 {
        all_questions.push(db.question_id_from_idx(quiz_id, idx).await.unwrap());
    }

    let requested = vec![
        all_questions[0],
        all_questions[1],
        all_questions[0],
        all_questions[2],
    ];

    let token = db
        .create_session_with_questions("dedupe", quiz_id, &requested, "incorrect", user_id)
        .await
        .unwrap();
    let session = db.get_session(&token).await.unwrap();

    let ids = get_session_question_ids(&db, session.id).await;
    assert_eq!(ids.len(), 3);

    let unique: HashSet<i32> = ids.iter().copied().collect();
    assert_eq!(unique.len(), ids.len());
}

// --- Bookmark tests ---

#[tokio::test]
async fn test_bookmark_default_false() {
    let db = create_test_db().await;
    let user_id = create_test_user(&db).await;
    let (_public_id, quiz_id) = load_quiz_with_id(&db, "Quiz", minimal_questions(), user_id).await;
    let (_, session_id) = db
        .create_session("bm-test", quiz_id, 5, "random", user_id)
        .await
        .unwrap();
    let question_id = db.get_question_by_idx(session_id, 0).await.unwrap();

    let is_bm = db
        .is_question_bookmarked(session_id, question_id)
        .await
        .unwrap();
    assert!(!is_bm, "New questions should not be bookmarked by default");
}

#[tokio::test]
async fn test_bookmark_toggle() {
    let db = create_test_db().await;
    let user_id = create_test_user(&db).await;
    let (_public_id, quiz_id) = load_quiz_with_id(&db, "Quiz", minimal_questions(), user_id).await;
    let (_, session_id) = db
        .create_session("bm-toggle", quiz_id, 5, "random", user_id)
        .await
        .unwrap();
    let question_id = db.get_question_by_idx(session_id, 0).await.unwrap();

    // Toggle on
    let new_state = db.toggle_bookmark(session_id, question_id).await.unwrap();
    assert!(new_state, "First toggle should set bookmark to true");

    // Toggle off
    let new_state = db.toggle_bookmark(session_id, question_id).await.unwrap();
    assert!(!new_state, "Second toggle should set bookmark to false");

    // Toggle on again
    let new_state = db.toggle_bookmark(session_id, question_id).await.unwrap();
    assert!(new_state, "Third toggle should set bookmark to true");
}

#[tokio::test]
async fn test_get_bookmarked_questions() {
    let db = create_test_db().await;
    let user_id = create_test_user(&db).await;
    let (_public_id, quiz_id) = load_quiz_with_id(&db, "Quiz", make_questions(5), user_id).await;
    let (_, session_id) = db
        .create_session("bm-list", quiz_id, 5, "random", user_id)
        .await
        .unwrap();
    let ids = get_session_question_ids(&db, session_id).await;

    // No bookmarks initially
    let bookmarked = db.get_bookmarked_questions(session_id).await.unwrap();
    assert!(bookmarked.is_empty(), "No bookmarks initially");

    // Bookmark 2 questions
    db.toggle_bookmark(session_id, ids[0]).await.unwrap();
    db.toggle_bookmark(session_id, ids[2]).await.unwrap();

    let bookmarked = db.get_bookmarked_questions(session_id).await.unwrap();
    assert_eq!(bookmarked.len(), 2);
    let bookmarked_set: HashSet<i32> = bookmarked.into_iter().collect();
    assert!(bookmarked_set.contains(&ids[0]));
    assert!(bookmarked_set.contains(&ids[2]));

    // Un-bookmark one
    db.toggle_bookmark(session_id, ids[0]).await.unwrap();
    let bookmarked = db.get_bookmarked_questions(session_id).await.unwrap();
    assert_eq!(bookmarked.len(), 1);
    assert_eq!(bookmarked[0], ids[2]);
}

// --- User tests ---

#[tokio::test]
async fn test_user_crud() {
    let db = create_test_db().await;

    let user_id = db
        .create_user("test@example.com", "password123", "Test User")
        .await
        .unwrap();
    assert!(user_id > 0);

    let found = db.find_user_by_email("test@example.com").await.unwrap();
    assert!(found.is_some());
    let user = found.unwrap();
    assert_eq!(user.id, user_id);
    assert_eq!(user.email, "test@example.com");
    assert_eq!(user.display_name, "Test User");

    assert!(db.email_exists("test@example.com").await.unwrap());
    assert!(!db.email_exists("other@example.com").await.unwrap());
}

#[tokio::test]
async fn test_user_password_verification() {
    let db = create_test_db().await;

    db.create_user("test@example.com", "correct-password", "Test")
        .await
        .unwrap();

    assert!(db
        .verify_user_password("test@example.com", "correct-password")
        .await
        .unwrap());
    assert!(!db
        .verify_user_password("test@example.com", "wrong-password")
        .await
        .unwrap());
    assert!(!db
        .verify_user_password("nonexistent@example.com", "any")
        .await
        .unwrap());
}

#[tokio::test]
async fn test_user_session() {
    let db = create_test_db().await;

    let user_id = db
        .create_user("test@example.com", "password", "Test")
        .await
        .unwrap();
    let session = db.create_user_session(user_id).await.unwrap();
    assert!(!session.is_empty());

    let found = db.get_user_by_session(&session).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, user_id);

    db.delete_user_session(&session).await.unwrap();
    let found = db.get_user_by_session(&session).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn test_quiz_isolation_between_users() {
    let db = create_test_db().await;

    let user1 = db
        .create_user("user1@example.com", "pw1", "User 1")
        .await
        .unwrap();
    let user2 = db
        .create_user("user2@example.com", "pw2", "User 2")
        .await
        .unwrap();

    load_quiz_with_id(&db, "User1 Quiz", minimal_questions(), user1).await;
    load_quiz_with_id(&db, "User2 Quiz", minimal_questions(), user2).await;

    let quizzes1 = db.quizzes(user1).await.unwrap();
    assert_eq!(quizzes1.len(), 1);
    assert_eq!(quizzes1[0].name, "User1 Quiz");

    let quizzes2 = db.quizzes(user2).await.unwrap();
    assert_eq!(quizzes2.len(), 1);
    assert_eq!(quizzes2[0].name, "User2 Quiz");
}

// --- Email verification tests ---

#[tokio::test]
async fn test_unverified_user_and_verify_token() {
    let db = create_test_db().await;

    let (user_id, token) = db
        .create_unverified_user("unverified@example.com", "password", "Unverified")
        .await
        .expect("create unverified user");
    assert!(user_id > 0);
    assert!(!token.is_empty());

    // Should not be verified
    assert!(!db
        .is_email_verified("unverified@example.com")
        .await
        .unwrap());

    // Verify with correct token
    assert!(db.verify_email_token(&token).await.unwrap());

    // Now should be verified
    assert!(db
        .is_email_verified("unverified@example.com")
        .await
        .unwrap());

    // Token should be consumed
    assert!(!db.verify_email_token(&token).await.unwrap());
}

#[tokio::test]
async fn test_verify_invalid_token() {
    let db = create_test_db().await;
    assert!(!db.verify_email_token("nonexistent-token").await.unwrap());
}

#[tokio::test]
async fn test_regenerate_verification_token() {
    let db = create_test_db().await;

    let (_user_id, original_token) = db
        .create_unverified_user("regen@example.com", "password", "Regen")
        .await
        .expect("create unverified user");

    let new_token = db
        .regenerate_verification_token("regen@example.com")
        .await
        .unwrap();
    assert!(new_token.is_some());
    let new_token = new_token.unwrap();
    assert_ne!(new_token, original_token);

    // Old token should no longer work
    assert!(!db.verify_email_token(&original_token).await.unwrap());

    // New token should work
    assert!(db.verify_email_token(&new_token).await.unwrap());
}

#[tokio::test]
async fn test_existing_user_is_verified_by_default() {
    let db = create_test_db().await;
    db.create_user("existing@example.com", "password", "Existing")
        .await
        .unwrap();

    // create_user uses DEFAULT TRUE, so should be verified
    assert!(db.is_email_verified("existing@example.com").await.unwrap());
}

// --- Password reset tests ---

#[tokio::test]
async fn test_create_password_reset_token_verified_user() {
    let db = create_test_db().await;
    db.create_user("reset@example.com", "password", "Reset User")
        .await
        .unwrap();

    let token = db
        .create_password_reset_token("reset@example.com")
        .await
        .unwrap();
    assert!(token.is_some());
}

#[tokio::test]
async fn test_create_password_reset_token_nonexistent_email() {
    let db = create_test_db().await;

    let token = db
        .create_password_reset_token("nobody@example.com")
        .await
        .unwrap();
    assert!(token.is_none());
}

#[tokio::test]
async fn test_create_password_reset_token_unverified_user() {
    let db = create_test_db().await;
    db.create_unverified_user("unverified@example.com", "password", "Unverified")
        .await
        .unwrap();

    // Unverified users should not get reset tokens
    let token = db
        .create_password_reset_token("unverified@example.com")
        .await
        .unwrap();
    assert!(token.is_none());
}

#[tokio::test]
async fn test_validate_and_reset_password_with_token() {
    let db = create_test_db().await;
    db.create_user("reset@example.com", "old-password", "Reset")
        .await
        .unwrap();

    let token = db
        .create_password_reset_token("reset@example.com")
        .await
        .unwrap()
        .unwrap();

    // Validate token
    let email = db.validate_password_reset_token(&token).await.unwrap();
    assert_eq!(email.as_deref(), Some("reset@example.com"));

    // Reset password
    assert!(db
        .reset_password_with_token(&token, "new-password")
        .await
        .unwrap());

    // Token should be consumed
    let email = db.validate_password_reset_token(&token).await.unwrap();
    assert!(email.is_none());

    // Old password should not work
    assert!(!db
        .verify_user_password("reset@example.com", "old-password")
        .await
        .unwrap());

    // New password should work
    assert!(db
        .verify_user_password("reset@example.com", "new-password")
        .await
        .unwrap());
}

#[tokio::test]
async fn test_reset_password_invalid_token() {
    let db = create_test_db().await;
    assert!(!db
        .reset_password_with_token("invalid-token", "new-pass")
        .await
        .unwrap());
}

#[tokio::test]
async fn test_change_password_success() {
    let db = create_test_db().await;
    let user_id = db
        .create_user("change@example.com", "current-pass", "Change")
        .await
        .unwrap();

    assert!(db
        .change_password(user_id, "current-pass", "new-pass")
        .await
        .unwrap());

    // Login with new password
    assert!(db
        .verify_user_password("change@example.com", "new-pass")
        .await
        .unwrap());
    assert!(!db
        .verify_user_password("change@example.com", "current-pass")
        .await
        .unwrap());
}

#[tokio::test]
async fn test_change_password_wrong_current() {
    let db = create_test_db().await;
    let user_id = db
        .create_user("change@example.com", "correct-pass", "Change")
        .await
        .unwrap();

    assert!(!db
        .change_password(user_id, "wrong-pass", "new-pass")
        .await
        .unwrap());

    // Original password should still work
    assert!(db
        .verify_user_password("change@example.com", "correct-pass")
        .await
        .unwrap());
}
