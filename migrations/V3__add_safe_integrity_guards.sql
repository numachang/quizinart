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

-- Forward-looking integrity guards (do not alter existing rows)
CREATE TRIGGER IF NOT EXISTS trg_quiz_sessions_unique_token
BEFORE INSERT ON quiz_sessions
FOR EACH ROW
WHEN EXISTS (
    SELECT 1 FROM quiz_sessions WHERE session_token = NEW.session_token
)
BEGIN
    SELECT RAISE(ABORT, 'duplicate session_token');
END;

CREATE TRIGGER IF NOT EXISTS trg_session_questions_unique_pair
BEFORE INSERT ON session_questions
FOR EACH ROW
WHEN EXISTS (
    SELECT 1
    FROM session_questions
    WHERE session_id = NEW.session_id
      AND question_id = NEW.question_id
)
BEGIN
    SELECT RAISE(ABORT, 'duplicate session_questions(session_id, question_id)');
END;

CREATE TRIGGER IF NOT EXISTS trg_user_answers_unique_triplet
BEFORE INSERT ON user_answers
FOR EACH ROW
WHEN EXISTS (
    SELECT 1
    FROM user_answers
    WHERE session_id = NEW.session_id
      AND question_id = NEW.question_id
      AND option_id = NEW.option_id
)
BEGIN
    SELECT RAISE(ABORT, 'duplicate user_answers(session_id, question_id, option_id)');
END;
