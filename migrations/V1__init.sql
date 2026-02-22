CREATE TABLE IF NOT EXISTS admin (
    id INTEGER PRIMARY KEY,
    password TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS admin_sessions (
    id TEXT PRIMARY KEY
);

CREATE TABLE IF NOT EXISTS quizzes (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS questions (
    id SERIAL PRIMARY KEY,
    question TEXT NOT NULL,
    category TEXT,
    is_multiple_choice BOOLEAN DEFAULT FALSE,
    quiz_id INTEGER NOT NULL,
    FOREIGN KEY(quiz_id) REFERENCES quizzes(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS options (
    id SERIAL PRIMARY KEY,
    option TEXT NOT NULL,
    is_answer BOOLEAN NOT NULL,
    explanation TEXT,
    question_id INTEGER NOT NULL,
    FOREIGN KEY(question_id) REFERENCES questions(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS quiz_sessions (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    session_token TEXT NOT NULL,
    shuffle_seed INTEGER,
    question_count INTEGER,
    selection_mode TEXT DEFAULT 'unanswered',
    quiz_id INTEGER NOT NULL,
    FOREIGN KEY(quiz_id) REFERENCES quizzes(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_quiz_sessions_unique_name_quiz
ON quiz_sessions(name, quiz_id);

CREATE TABLE IF NOT EXISTS session_questions (
    id SERIAL PRIMARY KEY,
    session_id INTEGER NOT NULL,
    question_id INTEGER NOT NULL,
    question_number INTEGER NOT NULL,
    is_correct BOOLEAN DEFAULT NULL,
    FOREIGN KEY(session_id) REFERENCES quiz_sessions(id) ON DELETE CASCADE,
    FOREIGN KEY(question_id) REFERENCES questions(id) ON DELETE CASCADE,
    UNIQUE(session_id, question_number)
);

CREATE TABLE IF NOT EXISTS user_answers (
    id SERIAL PRIMARY KEY,
    is_correct BOOLEAN NOT NULL,
    option_id INTEGER NOT NULL,
    question_id INTEGER NOT NULL,
    session_id INTEGER NOT NULL,
    FOREIGN KEY(option_id) REFERENCES options(id) ON DELETE CASCADE,
    FOREIGN KEY(question_id) REFERENCES questions(id) ON DELETE CASCADE,
    FOREIGN KEY(session_id) REFERENCES quiz_sessions(id) ON DELETE CASCADE
);

CREATE OR REPLACE VIEW session_stats AS
SELECT
    s.id AS session_id,
    s.name,
    s.session_token,
    s.quiz_id,
    s.question_count,
    s.selection_mode,
    (SELECT COUNT(*)::INTEGER FROM session_questions WHERE session_id = s.id) AS total_questions,
    (SELECT COUNT(*)::INTEGER FROM session_questions WHERE session_id = s.id AND is_correct IS NOT NULL) AS answered_questions,
    (SELECT COUNT(*)::INTEGER FROM session_questions WHERE session_id = s.id AND is_correct IS TRUE) AS correct_answers,
    CASE WHEN
        (SELECT COUNT(*) FROM session_questions WHERE session_id = s.id AND is_correct IS NOT NULL)
        >= (SELECT COUNT(*) FROM session_questions WHERE session_id = s.id)
        AND (SELECT COUNT(*) FROM session_questions WHERE session_id = s.id) > 0
    THEN TRUE ELSE FALSE END AS is_complete
FROM quiz_sessions s;

CREATE OR REPLACE VIEW question_stats AS
SELECT
    q.id AS question_id,
    q.quiz_id,
    COUNT(DISTINCT sq.session_id)::INTEGER AS times_asked,
    COUNT(DISTINCT CASE WHEN sq.is_correct IS FALSE THEN sq.session_id END)::INTEGER AS times_incorrect,
    CASE
        WHEN COUNT(DISTINCT sq.session_id) = 0 THEN NULL
        ELSE ROUND(
            CAST(COUNT(DISTINCT sq.session_id) - COUNT(DISTINCT CASE WHEN sq.is_correct IS FALSE THEN sq.session_id END) AS NUMERIC)
            / COUNT(DISTINCT sq.session_id), 2
        )::FLOAT8
    END AS accuracy
FROM questions q
LEFT JOIN session_questions sq ON sq.question_id = q.id AND sq.is_correct IS NOT NULL
GROUP BY q.id, q.quiz_id;
