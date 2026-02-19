mod common;

use std::collections::HashSet;

use common::create_test_db;
use quizinart::db::Db;
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
    let db = create_test_db().await;
    assert!(db.migration_applied("V1").await.unwrap());
    assert!(db.migration_applied("V2").await.unwrap());
    assert!(db.migration_applied("V3").await.unwrap());
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

    let quiz_id = db
        .load_quiz("Test Quiz".to_string(), sample_questions())
        .await
        .unwrap();
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

    let quiz_id = db
        .load_quiz("Quiz".to_string(), minimal_questions())
        .await
        .unwrap();
    let token = db
        .create_session("session-1", quiz_id, 5, "random")
        .await
        .unwrap();
    assert!(!token.is_empty());

    let session = db.get_session(&token).await.unwrap();
    assert_eq!(session.name, "session-1");
    assert_eq!(session.quiz_id, quiz_id);
}

#[tokio::test]
async fn test_duplicate_session_name() {
    let db = create_test_db().await;

    let quiz_id = db
        .load_quiz("Quiz".to_string(), minimal_questions())
        .await
        .unwrap();
    db.create_session("dupe", quiz_id, 5, "random")
        .await
        .unwrap();

    // Same name, same quiz -> should fail
    let result = db.create_session("dupe", quiz_id, 5, "random").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already in use"));
}

#[tokio::test]
async fn test_session_count() {
    let db = create_test_db().await;

    let quiz_id = db
        .load_quiz("Quiz".to_string(), minimal_questions())
        .await
        .unwrap();
    assert_eq!(db.sessions_count(quiz_id).await.unwrap(), 0);

    db.create_session("s1", quiz_id, 5, "random").await.unwrap();
    assert_eq!(db.sessions_count(quiz_id).await.unwrap(), 1);

    db.create_session("s2", quiz_id, 5, "random").await.unwrap();
    assert_eq!(db.sessions_count(quiz_id).await.unwrap(), 2);
}

#[tokio::test]
async fn test_delete_session() {
    let db = create_test_db().await;

    let quiz_id = db
        .load_quiz("Quiz".to_string(), minimal_questions())
        .await
        .unwrap();
    let token = db
        .create_session("to-delete", quiz_id, 5, "random")
        .await
        .unwrap();
    let session = db.get_session(&token).await.unwrap();

    assert_eq!(db.sessions_count(quiz_id).await.unwrap(), 1);

    db.delete_session(session.id).await.unwrap();
    assert_eq!(db.sessions_count(quiz_id).await.unwrap(), 0);

    // Session should no longer be retrievable
    assert!(db.get_session(&token).await.is_err());
}

#[tokio::test]
async fn test_rename_session() {
    let db = create_test_db().await;

    let quiz_id = db
        .load_quiz("Quiz".to_string(), minimal_questions())
        .await
        .unwrap();
    let token = db
        .create_session("old-name", quiz_id, 5, "random")
        .await
        .unwrap();
    let session = db.get_session(&token).await.unwrap();

    db.rename_session(session.id, "new-name", quiz_id)
        .await
        .unwrap();

    let renamed = db.get_session_by_id(session.id).await.unwrap();
    assert_eq!(renamed.name, "new-name");
}

#[tokio::test]
async fn test_rename_session_duplicate() {
    let db = create_test_db().await;

    let quiz_id = db
        .load_quiz("Quiz".to_string(), minimal_questions())
        .await
        .unwrap();
    db.create_session("existing", quiz_id, 5, "random")
        .await
        .unwrap();
    let token2 = db
        .create_session("to-rename", quiz_id, 5, "random")
        .await
        .unwrap();
    let session2 = db.get_session(&token2).await.unwrap();

    // Renaming to an existing name should fail
    let result = db.rename_session(session2.id, "existing", quiz_id).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already in use"));
}

// --- Question selection tests ---

#[tokio::test]
async fn test_random_mode_no_duplicates() {
    let db = create_test_db().await;
    let quiz_id = db
        .load_quiz("Quiz".to_string(), make_questions(10))
        .await
        .unwrap();

    let token = db
        .create_session("random-session", quiz_id, 5, "random")
        .await
        .unwrap();
    let session = db.get_session(&token).await.unwrap();
    let ids = get_session_question_ids(&db, session.id).await;

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
    let quiz_id = db
        .load_quiz("Quiz".to_string(), make_questions(3))
        .await
        .unwrap();

    // Request more questions than exist
    let token = db
        .create_session("random-big", quiz_id, 10, "random")
        .await
        .unwrap();
    let session = db.get_session(&token).await.unwrap();
    let ids = get_session_question_ids(&db, session.id).await;

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
    let quiz_id = db
        .load_quiz("Quiz".to_string(), make_questions(10))
        .await
        .unwrap();

    // Session 1: pick 4 unanswered questions
    let token1 = db
        .create_session("s1", quiz_id, 4, "unanswered")
        .await
        .unwrap();
    let s1 = db.get_session(&token1).await.unwrap();
    let ids1 = get_session_question_ids(&db, s1.id).await;

    assert_eq!(ids1.len(), 4);
    let unique1: HashSet<i32> = ids1.iter().cloned().collect();
    assert_eq!(
        unique1.len(),
        ids1.len(),
        "Session 1 has duplicates: {:?}",
        ids1
    );

    // Session 2: pick 4 more unanswered questions — should NOT overlap with session 1
    let token2 = db
        .create_session("s2", quiz_id, 4, "unanswered")
        .await
        .unwrap();
    let s2 = db.get_session(&token2).await.unwrap();
    let ids2 = get_session_question_ids(&db, s2.id).await;

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
    let quiz_id = db
        .load_quiz("Quiz".to_string(), make_questions(5))
        .await
        .unwrap();

    // Session 1: exhaust all 5 questions
    let token1 = db
        .create_session("s1", quiz_id, 5, "unanswered")
        .await
        .unwrap();
    let s1 = db.get_session(&token1).await.unwrap();
    let ids1 = get_session_question_ids(&db, s1.id).await;
    assert_eq!(ids1.len(), 5);

    // Session 2: no unanswered left — fallback fills from already-asked
    let token2 = db
        .create_session("s2", quiz_id, 3, "unanswered")
        .await
        .unwrap();
    let s2 = db.get_session(&token2).await.unwrap();
    let ids2 = get_session_question_ids(&db, s2.id).await;

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
    let quiz_id = db
        .load_quiz("Quiz".to_string(), make_questions(6))
        .await
        .unwrap();

    // Session 1: use 4 out of 6
    let token1 = db
        .create_session("s1", quiz_id, 4, "unanswered")
        .await
        .unwrap();
    let s1 = db.get_session(&token1).await.unwrap();
    let ids1 = get_session_question_ids(&db, s1.id).await;
    assert_eq!(ids1.len(), 4);

    // Session 2: request 4, only 2 unanswered remain → 2 unanswered + 2 fill
    let token2 = db
        .create_session("s2", quiz_id, 4, "unanswered")
        .await
        .unwrap();
    let s2 = db.get_session(&token2).await.unwrap();
    let ids2 = get_session_question_ids(&db, s2.id).await;

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
    let quiz_id = db
        .load_quiz("Quiz".to_string(), make_questions(5))
        .await
        .unwrap();

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
        .create_session_with_questions("dedupe", quiz_id, &requested, "incorrect")
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
    let quiz_id = db
        .load_quiz("Quiz".to_string(), minimal_questions())
        .await
        .unwrap();
    let token = db
        .create_session("bm-test", quiz_id, 5, "random")
        .await
        .unwrap();
    let session = db.get_session(&token).await.unwrap();
    let question_id = db.get_question_by_idx(session.id, 0).await.unwrap();

    let is_bm = db
        .is_question_bookmarked(session.id, question_id)
        .await
        .unwrap();
    assert!(!is_bm, "New questions should not be bookmarked by default");
}

#[tokio::test]
async fn test_bookmark_toggle() {
    let db = create_test_db().await;
    let quiz_id = db
        .load_quiz("Quiz".to_string(), minimal_questions())
        .await
        .unwrap();
    let token = db
        .create_session("bm-toggle", quiz_id, 5, "random")
        .await
        .unwrap();
    let session = db.get_session(&token).await.unwrap();
    let question_id = db.get_question_by_idx(session.id, 0).await.unwrap();

    // Toggle on
    let new_state = db.toggle_bookmark(session.id, question_id).await.unwrap();
    assert!(new_state, "First toggle should set bookmark to true");

    // Toggle off
    let new_state = db.toggle_bookmark(session.id, question_id).await.unwrap();
    assert!(!new_state, "Second toggle should set bookmark to false");

    // Toggle on again
    let new_state = db.toggle_bookmark(session.id, question_id).await.unwrap();
    assert!(new_state, "Third toggle should set bookmark to true");
}

#[tokio::test]
async fn test_get_bookmarked_questions() {
    let db = create_test_db().await;
    let quiz_id = db
        .load_quiz("Quiz".to_string(), make_questions(5))
        .await
        .unwrap();
    let token = db
        .create_session("bm-list", quiz_id, 5, "random")
        .await
        .unwrap();
    let session = db.get_session(&token).await.unwrap();
    let ids = get_session_question_ids(&db, session.id).await;

    // No bookmarks initially
    let bookmarked = db.get_bookmarked_questions(session.id).await.unwrap();
    assert!(bookmarked.is_empty(), "No bookmarks initially");

    // Bookmark 2 questions
    db.toggle_bookmark(session.id, ids[0]).await.unwrap();
    db.toggle_bookmark(session.id, ids[2]).await.unwrap();

    let bookmarked = db.get_bookmarked_questions(session.id).await.unwrap();
    assert_eq!(bookmarked.len(), 2);
    let bookmarked_set: HashSet<i32> = bookmarked.into_iter().collect();
    assert!(bookmarked_set.contains(&ids[0]));
    assert!(bookmarked_set.contains(&ids[2]));

    // Un-bookmark one
    db.toggle_bookmark(session.id, ids[0]).await.unwrap();
    let bookmarked = db.get_bookmarked_questions(session.id).await.unwrap();
    assert_eq!(bookmarked.len(), 1);
    assert_eq!(bookmarked[0], ids[2]);
}
