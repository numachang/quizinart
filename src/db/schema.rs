// Database schema initialization

use color_eyre::Result;

pub async fn create_schema(conn: &libsql::Connection) -> Result<()> {
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS admin (
            id INTEGER PRIMARY KEY,
            password TEXT NOT NULL
        )
        "#,
        (),
    )
    .await?;

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS admin_sessions (
            id TEXT PRIMARY KEY
        )
        "#,
        (),
    )
    .await?;

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS quizzes (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL
        )
        "#,
        (),
    )
    .await?;

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS questions (
            id INTEGER PRIMARY KEY,
            question TEXT NOT NULL,
            category TEXT,
            is_multiple_choice BOOLEAN DEFAULT 0,
            quiz_id INTEGER NOT NULL,
            FOREIGN KEY(quiz_id) REFERENCES quizzes(id) ON DELETE CASCADE
        )
        "#,
        (),
    )
    .await?;

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS options (
            id INTEGER PRIMARY KEY,
            option TEXT NOT NULL,
            is_answer BOOLEAN NOT NULL,
            explanation TEXT,
            question_id INTEGER NOT NULL,
            FOREIGN KEY(question_id) REFERENCES questions(id) ON DELETE CASCADE
        )
        "#,
        (),
    )
    .await?;

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS quiz_sessions (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            session_token TEXT NOT NULL,
            shuffle_seed INTEGER,
            question_count INTEGER,
            selection_mode TEXT DEFAULT 'unanswered',
            quiz_id INTEGER NOT NULL,
            FOREIGN KEY(quiz_id) REFERENCES quizzes(id) ON DELETE CASCADE
        )
        "#,
        (),
    )
    .await?;

    conn.execute(
        r#"
        CREATE UNIQUE INDEX IF NOT EXISTS idx_quiz_sessions_unique_name_quiz
        ON quiz_sessions(name, quiz_id)
        "#,
        (),
    )
    .await?;

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS session_questions (
            id INTEGER PRIMARY KEY,
            session_id INTEGER NOT NULL,
            question_id INTEGER NOT NULL,
            question_number INTEGER NOT NULL,
            is_correct BOOLEAN DEFAULT NULL,
            FOREIGN KEY(session_id) REFERENCES quiz_sessions(id) ON DELETE CASCADE,
            FOREIGN KEY(question_id) REFERENCES questions(id) ON DELETE CASCADE,
            UNIQUE(session_id, question_number)
        )
        "#,
        (),
    )
    .await?;

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS user_answers (
            id INTEGER PRIMARY KEY,
            is_correct BOOLEAN NOT NULL,
            option_id INTEGER NOT NULL,
            question_id INTEGER NOT NULL,
            session_id INTEGER NOT NULL,
            FOREIGN KEY(option_id) REFERENCES options(id) ON DELETE CASCADE,
            FOREIGN KEY(question_id) REFERENCES questions(id) ON DELETE CASCADE,
            FOREIGN KEY(session_id) REFERENCES quiz_sessions(id) ON DELETE CASCADE
        )
        "#,
        (),
    )
    .await?;

    // VIEW: セッション統計（回答数・正解数・完了判定を一元化）
    conn.execute(
        r#"
        CREATE VIEW IF NOT EXISTS session_stats AS
        SELECT
            s.id AS session_id,
            s.name,
            s.session_token,
            s.quiz_id,
            s.question_count,
            s.selection_mode,
            (SELECT COUNT(*) FROM session_questions WHERE session_id = s.id) AS total_questions,
            (SELECT COUNT(*) FROM session_questions WHERE session_id = s.id AND is_correct IS NOT NULL) AS answered_questions,
            (SELECT COUNT(*) FROM session_questions WHERE session_id = s.id AND is_correct = 1) AS correct_answers,
            CASE WHEN
                (SELECT COUNT(*) FROM session_questions WHERE session_id = s.id AND is_correct IS NOT NULL)
                >= (SELECT COUNT(*) FROM session_questions WHERE session_id = s.id)
                AND (SELECT COUNT(*) FROM session_questions WHERE session_id = s.id) > 0
            THEN 1 ELSE 0 END AS is_complete
        FROM quiz_sessions s
        "#,
        (),
    )
    .await?;

    // VIEW: 問題統計（出題回数・間違い回数・正答率）
    conn.execute(
        r#"
        CREATE VIEW IF NOT EXISTS question_stats AS
        SELECT
            q.id AS question_id,
            q.quiz_id,
            COUNT(DISTINCT sq.session_id) AS times_asked,
            COUNT(DISTINCT CASE WHEN sq.is_correct = 0 THEN sq.session_id END) AS times_incorrect,
            CASE
                WHEN COUNT(DISTINCT sq.session_id) = 0 THEN NULL
                ELSE ROUND(
                    CAST(COUNT(DISTINCT sq.session_id) - COUNT(DISTINCT CASE WHEN sq.is_correct = 0 THEN sq.session_id END) AS REAL)
                    / COUNT(DISTINCT sq.session_id), 2
                )
            END AS accuracy
        FROM questions q
        LEFT JOIN session_questions sq ON sq.question_id = q.id AND sq.is_correct IS NOT NULL
        GROUP BY q.id, q.quiz_id
        "#,
        (),
    )
    .await?;

    Ok(())
}
