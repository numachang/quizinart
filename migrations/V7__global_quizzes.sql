-- 1. Add public_id column (nullable initially for startup backfill via Rust ULID)
ALTER TABLE quizzes ADD COLUMN public_id TEXT UNIQUE;

-- 2. Rename user_id → owner_id
ALTER TABLE quizzes RENAME COLUMN user_id TO owner_id;

-- 3. Add is_shared flag (for future sharing support)
ALTER TABLE quizzes ADD COLUMN is_shared BOOLEAN NOT NULL DEFAULT FALSE;

-- 4. Create user_quizzes join table (lightweight: only holds user_id + quiz_id)
CREATE TABLE IF NOT EXISTS user_quizzes (
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    quiz_id INTEGER NOT NULL REFERENCES quizzes(id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, quiz_id)
);

-- 5. Populate user_quizzes from existing ownership data
INSERT INTO user_quizzes (user_id, quiz_id)
SELECT owner_id, id FROM quizzes WHERE owner_id IS NOT NULL;

-- 6. Update indexes
DROP INDEX IF EXISTS idx_quizzes_user_id;
CREATE INDEX IF NOT EXISTS idx_quizzes_owner_id ON quizzes(owner_id);
CREATE INDEX IF NOT EXISTS idx_user_quizzes_user_id ON user_quizzes(user_id);
CREATE INDEX IF NOT EXISTS idx_quizzes_public_id ON quizzes(public_id);
