-- Non-destructive performance indexes
CREATE INDEX IF NOT EXISTS idx_quiz_sessions_session_token
ON quiz_sessions(session_token);

CREATE INDEX IF NOT EXISTS idx_questions_quiz_id
ON questions(quiz_id);

CREATE INDEX IF NOT EXISTS idx_options_question_id
ON options(question_id);

CREATE INDEX IF NOT EXISTS idx_session_questions_session_id_question_id
ON session_questions(session_id, question_id);

CREATE INDEX IF NOT EXISTS idx_session_questions_session_id_is_correct
ON session_questions(session_id, is_correct);

CREATE INDEX IF NOT EXISTS idx_session_questions_question_id_is_correct
ON session_questions(question_id, is_correct);

CREATE INDEX IF NOT EXISTS idx_user_answers_session_question
ON user_answers(session_id, question_id);

CREATE INDEX IF NOT EXISTS idx_user_answers_session_question_option
ON user_answers(session_id, question_id, option_id);

-- Integrity guards: replace SQLite triggers with PostgreSQL unique constraints
ALTER TABLE quiz_sessions ADD CONSTRAINT uq_quiz_sessions_session_token UNIQUE (session_token);
ALTER TABLE session_questions ADD CONSTRAINT uq_session_questions_session_question UNIQUE (session_id, question_id);
ALTER TABLE user_answers ADD CONSTRAINT uq_user_answers_triplet UNIQUE (session_id, question_id, option_id);
